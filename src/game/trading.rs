use crate::data::commodities::{CommodityType, MarketPrices, TradingSystem};
use crate::data::systems::{EconomyType, StarSystem, SystemManager};
use crate::game::commander::Standing;
use crate::game::{Commander, Ship, TradeLog};
use crate::utils::display::{show_error, show_info, show_success};
use colored::*;
use std::collections::HashMap;

const ILLEGAL_TRADE_REPUTATION: i32 = -2;
const MARKET_REFRESH_JUMPS: u32 = 20;
const IMPACT_PER_TON: f64 = 0.015;
const MAX_IMPACT: f64 = 0.40;
const RECOVERY_PER_JUMP: f64 = 0.10;
const MARKER_JUMPS: u32 = 16;

/// Trading command handler
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TradingHandler {
    trading_system: TradingSystem,
    /// Each system's market, kept between visits
    #[serde(default)]
    markets: HashMap<String, SystemMarket>,
}

/// What a system's market remembers: its price roll and how the player has pushed it since
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct SystemMarket {
    pub seed: u64,
    pub rolled_at: u32,
    /// The clock when the pushes below were last brought up to date
    #[serde(default)]
    pub updated_at: u32,
    /// Tons bought since the roll that stock has not yet recovered from, per commodity
    #[serde(default)]
    pub purchased: HashMap<CommodityType, f64>,
    /// How far buying has raised each buy price, as a fraction of the undisturbed price
    #[serde(default)]
    pub bought_impact: HashMap<CommodityType, f64>,
    /// How far selling has lowered each sell price, as a fraction of the undisturbed price
    #[serde(default)]
    pub sold_impact: HashMap<CommodityType, f64>,
    /// The clock when each commodity was last bought here
    #[serde(default)]
    pub bought_at: HashMap<CommodityType, u32>,
    /// The clock when each commodity was last sold here
    #[serde(default)]
    pub sold_at: HashMap<CommodityType, u32>,
}

/// Which of the player's recent trades are still showing in a commodity's price
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Pushed {
    pub sold: bool,
    pub bought: bool,
}

impl Pushed {
    pub fn markers(&self) -> Vec<&'static str> {
        let mut markers = Vec::new();
        if self.sold {
            markers.push("▼ sold here");
        }
        if self.bought {
            markers.push("▲ bought here");
        }
        markers
    }
}

impl SystemMarket {
    fn rolled(clock: u32) -> Self {
        Self {
            seed: rand::random(),
            rolled_at: clock,
            updated_at: clock,
            ..Self::default()
        }
    }

    /// Close a share of every gap for each jump since the last update
    fn recover(&mut self, clock: u32) {
        let jumps = clock.saturating_sub(self.updated_at);
        if jumps == 0 {
            return;
        }
        let keep = (1.0 - RECOVERY_PER_JUMP).powi(jumps as i32);
        for gap in self
            .purchased
            .values_mut()
            .chain(self.bought_impact.values_mut())
            .chain(self.sold_impact.values_mut())
        {
            *gap *= keep;
        }
        self.updated_at = clock;
    }

    fn record_buy(&mut self, commodity: &CommodityType, tons: u32) {
        self.bought_at.insert(commodity.clone(), self.updated_at);
        *self.purchased.entry(commodity.clone()).or_insert(0.0) += f64::from(tons);
        let impact = self.bought_impact.entry(commodity.clone()).or_insert(0.0);
        *impact = (*impact + IMPACT_PER_TON * f64::from(tons)).min(MAX_IMPACT);
    }

    fn record_sell(&mut self, commodity: &CommodityType, tons: u32) {
        self.sold_at.insert(commodity.clone(), self.updated_at);
        let impact = self.sold_impact.entry(commodity.clone()).or_insert(0.0);
        *impact = (*impact + IMPACT_PER_TON * f64::from(tons)).min(MAX_IMPACT);
    }

    fn pushed(&self, commodity: &CommodityType, clock: u32) -> Pushed {
        let recent = |times: &HashMap<CommodityType, u32>| {
            times
                .get(commodity)
                .is_some_and(|at| clock.saturating_sub(*at) < MARKER_JUMPS)
        };
        Pushed {
            sold: recent(&self.sold_at),
            bought: recent(&self.bought_at),
        }
    }

    /// Apply what the player has done here to freshly generated prices and stock
    fn apply_to(&self, market: &mut MarketPrices) {
        for (commodity, impact) in &self.bought_impact {
            if let Some(price) = market.buy_prices.get_mut(commodity) {
                *price = (f64::from(*price) * (1.0 + impact)).round() as u32;
            }
        }
        for (commodity, impact) in &self.sold_impact {
            if let Some(price) = market.sell_prices.get_mut(commodity) {
                *price = ((f64::from(*price) * (1.0 - impact)).round() as u32).max(1);
            }
        }
        for (commodity, bought) in &self.purchased {
            if let Some(stock) = market.availability.get_mut(commodity) {
                *stock = stock.saturating_sub(bought.round() as u32);
            }
        }
    }
}

impl TradingHandler {
    pub fn new() -> Self {
        Self {
            trading_system: TradingSystem::new(),
            markets: HashMap::new(),
        }
    }

    /// A system's prices and stock stay fixed for a number of jumps, whoever arrives or leaves,
    /// and re-roll the next time they are looked at after that
    fn market_for(&mut self, location: &str, economy: &EconomyType, clock: u32) -> MarketPrices {
        let stale = self
            .markets
            .get(location)
            .is_none_or(|state| clock.saturating_sub(state.rolled_at) >= MARKET_REFRESH_JUMPS);
        if stale {
            self.markets
                .insert(location.to_string(), SystemMarket::rolled(clock));
        }
        let state = self
            .markets
            .get_mut(location)
            .expect("market was just inserted");
        state.recover(clock);
        let mut market = self
            .trading_system
            .generate_market_prices(economy, state.seed);
        state.apply_to(&mut market);
        market
    }

    /// The commodities at `location` whose prices the player's recent trading is still moving
    pub fn pushed_commodities(&self, location: &str, clock: u32) -> HashMap<CommodityType, Pushed> {
        let Some(state) = self.markets.get(location) else {
            return HashMap::new();
        };
        self.trading_system
            .get_commodities()
            .iter()
            .map(|commodity| (commodity.clone(), state.pushed(commodity, clock)))
            .filter(|(_, pushed)| *pushed != Pushed::default())
            .collect()
    }

    /// The market at `location` as the player sees it, or `None` for an unknown system
    pub fn market_view(
        &mut self,
        system_manager: &SystemManager,
        location: &str,
        standing: Standing,
        clock: u32,
    ) -> Option<MarketPrices> {
        let system = system_manager.get_system(location)?;
        Some(self.priced_market(location, &system.economy, standing, clock))
    }

    /// The market as the player sees it: stable prices adjusted for their standing
    fn priced_market(
        &mut self,
        location: &str,
        economy: &EconomyType,
        standing: Standing,
        clock: u32,
    ) -> MarketPrices {
        let mut market = self.market_for(location, economy, clock);
        for price in market.buy_prices.values_mut() {
            *price = standing.buy_price(*price);
        }
        for price in market.sell_prices.values_mut() {
            *price = standing.sell_price(*price);
        }
        market
    }

    /// Handle the market command - show current system's market prices
    pub fn handle_market(
        &mut self,
        system_manager: &SystemManager,
        current_location: &str,
        standing: Standing,
        clock: u32,
    ) {
        if let Some(system) = system_manager.get_system(current_location) {
            let market = self.priced_market(current_location, &system.economy, standing, clock);
            let pushed = self.pushed_commodities(current_location, clock);
            self.display_market(system, &market, standing, &pushed);
        } else {
            show_error("System not found");
        }
    }

    /// Handle the buy command - purchase commodities
    pub fn handle_buy(
        &mut self,
        args: &[String],
        commander: &mut Commander,
        ship: &mut Ship,
        system_manager: &SystemManager,
        trade_log: &mut TradeLog,
        clock: u32,
    ) -> bool {
        if args.is_empty() {
            show_error("Usage: buy <item> [quantity]");
            return false;
        }

        let commodity_name = &args[0];
        let quantity_str = args.get(1).map(|s| s.as_str()).unwrap_or("1");

        // Parse commodity
        let commodity = match self.parse_commodity(commodity_name) {
            Some(c) => c,
            None => {
                show_error(&format!("Unknown commodity: {}", commodity_name));
                return false;
            }
        };

        // Parse quantity
        let quantity = match quantity_str.parse::<u32>() {
            Ok(q) if q > 0 => q,
            _ => {
                show_error("Quantity must be a positive number");
                return false;
            }
        };

        // Get current system and market
        let current_location = commander.location();
        let system = match system_manager.get_system(current_location) {
            Some(s) => s,
            None => {
                show_error("System not found");
                return false;
            }
        };

        let market = self.priced_market(
            current_location,
            &system.economy,
            commander.standing(),
            clock,
        );

        // Perform the purchase
        self.execute_buy(commodity, quantity, commander, ship, &market, trade_log)
    }

    /// Handle the sell command - sell commodities
    pub fn handle_sell(
        &mut self,
        args: &[String],
        commander: &mut Commander,
        ship: &mut Ship,
        system_manager: &SystemManager,
        trade_log: &mut TradeLog,
        clock: u32,
    ) -> bool {
        if args.is_empty() {
            show_error("Usage: sell <item> [quantity]");
            return false;
        }

        let commodity_name = &args[0];
        let quantity_str = args.get(1).map(|s| s.as_str()).unwrap_or("1");

        // Parse commodity
        let commodity = match self.parse_commodity(commodity_name) {
            Some(c) => c,
            None => {
                show_error(&format!("Unknown commodity: {}", commodity_name));
                return false;
            }
        };

        // Parse quantity
        let quantity = match quantity_str.parse::<u32>() {
            Ok(q) if q > 0 => q,
            _ => {
                show_error("Quantity must be a positive number");
                return false;
            }
        };

        // Get current system and market
        let current_location = commander.location();
        let system = match system_manager.get_system(current_location) {
            Some(s) => s,
            None => {
                show_error("System not found");
                return false;
            }
        };

        let market = self.priced_market(
            current_location,
            &system.economy,
            commander.standing(),
            clock,
        );

        // Perform the sale
        self.execute_sell(commodity, quantity, commander, ship, &market, trade_log)
    }

    /// Handle the cargo command - show current cargo hold contents
    pub fn handle_cargo(&self, ship: &Ship) {
        self.display_cargo(ship);
    }

    /// Parse commodity name to CommodityType
    fn parse_commodity(&self, name: &str) -> Option<CommodityType> {
        CommodityType::from_string(name)
    }

    /// Execute a buy transaction
    fn execute_buy(
        &mut self,
        commodity: CommodityType,
        quantity: u32,
        commander: &mut Commander,
        ship: &mut Ship,
        market: &MarketPrices,
        trade_log: &mut TradeLog,
    ) -> bool {
        // Check if commodity is available
        let availability = match market.get_availability(&commodity) {
            Some(avail) => avail,
            None => {
                show_error("Commodity not available in this system");
                return false;
            }
        };

        if availability < quantity {
            show_error(&format!(
                "Only {} tons available, requested {}",
                availability, quantity
            ));
            return false;
        }

        // Check cargo space
        if !ship.has_cargo_space(quantity) {
            show_error(&format!(
                "Insufficient cargo space. Need {} tons, have {} tons available",
                quantity,
                ship.cargo_available()
            ));
            return false;
        }

        // Get buy price
        let buy_price = match market.get_buy_price(&commodity) {
            Some(price) => price,
            None => {
                show_error("Commodity not available for purchase");
                return false;
            }
        };

        let total_cost = buy_price * quantity;

        // Check if commander has enough credits
        if !commander.has_credits(total_cost) {
            show_error(&format!(
                "Insufficient credits. Need {}cr, have {}cr",
                total_cost,
                commander.credits()
            ));
            return false;
        }

        // Execute the transaction
        let cargo_before = ship.cargo_hold().get_quantity(&commodity);
        commander.remove_credits(total_cost);
        ship.add_cargo(commodity.clone(), quantity);
        if let Some(state) = self.markets.get_mut(commander.location()) {
            state.record_buy(&commodity, quantity);
        }
        trade_log.record_buy(
            commodity.clone(),
            quantity,
            buy_price,
            commander.location(),
            cargo_before,
        );

        show_success(&format!(
            "Purchased {} tons of {} for {}cr",
            quantity,
            commodity.name(),
            total_cost
        ));
        Self::penalise_illegal_trade(&commodity, commander);
        true
    }

    fn penalise_illegal_trade(commodity: &CommodityType, commander: &mut Commander) {
        if !commodity.is_illegal() {
            return;
        }
        let change = commander.change_reputation(ILLEGAL_TRADE_REPUTATION);
        println!(
            "{}",
            format!(
                "Trading {} is illegal. Reputation {:+}.",
                commodity.name(),
                change.applied
            )
            .yellow()
        );
        if let Some(standing) = change.new_standing {
            show_info(&format!("Your standing is now {}.", standing.name()));
        }
    }

    /// Execute a sell transaction
    fn execute_sell(
        &mut self,
        commodity: CommodityType,
        quantity: u32,
        commander: &mut Commander,
        ship: &mut Ship,
        market: &MarketPrices,
        trade_log: &mut TradeLog,
    ) -> bool {
        // Check if we have the commodity
        let current_quantity = ship.cargo_hold().get_quantity(&commodity);
        if current_quantity < quantity {
            show_error(&format!(
                "Insufficient cargo. Have {} tons, trying to sell {}",
                current_quantity, quantity
            ));
            return false;
        }

        // Get sell price
        let sell_price = match market.get_sell_price(&commodity) {
            Some(price) => price,
            None => {
                show_error("Commodity not available for sale");
                return false;
            }
        };

        let total_revenue = sell_price * quantity;

        // Execute the transaction
        ship.remove_cargo(&commodity, quantity);
        commander.add_credits(total_revenue);
        if let Some(state) = self.markets.get_mut(commander.location()) {
            state.record_sell(&commodity, quantity);
        }
        trade_log.record_sell(
            commodity.clone(),
            quantity,
            sell_price,
            commander.location(),
            current_quantity,
        );

        show_success(&format!(
            "Sold {} tons of {} for {}cr",
            quantity,
            commodity.name(),
            total_revenue
        ));
        Self::penalise_illegal_trade(&commodity, commander);
        true
    }

    /// Display market information
    fn display_market(
        &self,
        system: &StarSystem,
        market: &MarketPrices,
        standing: Standing,
        pushed: &HashMap<CommodityType, Pushed>,
    ) {
        println!("\n{} Market", system.name().yellow());
        println!("{}", "=".repeat(system.name().len() + 7));
        if standing != Standing::Neutral {
            println!(
                "Prices adjusted for your standing: {} ({:+}% on trades)",
                standing.name(),
                standing.price_favour_percent()
            );
        }

        for commodity in self.trading_system.get_commodities() {
            let buy_price = market.get_buy_price(commodity).unwrap_or(0);
            let sell_price = market.get_sell_price(commodity).unwrap_or(0);
            let availability = market.get_availability(commodity).unwrap_or(0);

            let commodity_name = format!("{}:", commodity.name());
            let buy_str = format!("Buy: {}cr", buy_price);
            let sell_str = format!("Sell: {}cr", sell_price);
            let avail_str = format!("Available: {} tons", availability);

            let versus_base = commodity.percent_vs_base(buy_price);
            let base_str = format!("{:+}% vs base", versus_base);
            let base_coloured = if versus_base <= -10 {
                base_str.green()
            } else if versus_base >= 10 {
                base_str.red()
            } else {
                base_str.normal()
            };

            let markers = pushed
                .get(commodity)
                .map(|pushed| pushed.markers().join(" "))
                .unwrap_or_default();
            println!(
                "{:<12} {:<12} {:<12} {:<20} {} {}",
                commodity_name,
                buy_str,
                sell_str,
                avail_str,
                base_coloured,
                markers.yellow()
            );
        }
        println!();
    }

    /// Display cargo information
    fn display_cargo(&self, ship: &Ship) {
        let cargo_hold = ship.cargo_hold();
        let current_weight = cargo_hold.current_weight();
        let capacity = ship.cargo_capacity();

        println!("\nCargo Hold ({}/{})", current_weight, capacity);
        println!("{}", "=".repeat(20));

        if cargo_hold.reserved > 0 {
            println!(
                "Contract cargo: {} tons (not for sale)",
                cargo_hold.reserved
            );
        }
        if cargo_hold.is_empty() {
            println!("No goods in the hold");
        } else {
            let mut total_value = 0;

            for item in cargo_hold.get_items() {
                // For display purposes, we'll use a rough estimate of value
                // In a real implementation, you'd track purchase prices
                let estimated_value = item.quantity * item.commodity.base_price();
                total_value += estimated_value;

                println!(
                    "{:<12} {} tons  Value: {}cr",
                    format!("{}:", item.commodity.name()),
                    item.quantity,
                    estimated_value
                );
            }

            println!("{}", "-".repeat(20));
            println!("Total Value: {}cr", total_value);
        }
        println!();
    }
}

impl Default for TradingHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_commodity() {
        let handler = TradingHandler::new();

        assert_eq!(handler.parse_commodity("food"), Some(CommodityType::Food));
        assert_eq!(handler.parse_commodity("FOOD"), Some(CommodityType::Food));
        assert_eq!(
            handler.parse_commodity("machinery"),
            Some(CommodityType::Machinery)
        );
        assert_eq!(handler.parse_commodity("unknown"), None);
    }

    #[test]
    fn prices_are_stable_across_arrivals_within_a_period_and_reroll_after_it() {
        let mut handler = TradingHandler::new();
        let economy = EconomyType::RichIndustrial;

        let first = handler.market_for("LAVE", &economy, 0);
        let seed = handler.markets["LAVE"].seed;
        handler.market_for("DISO", &economy, 1);
        let back = handler.market_for("LAVE", &economy, 2);
        assert_eq!(first.buy_prices, back.buy_prices);
        assert_eq!(first.availability, back.availability);
        assert_eq!(handler.markets["LAVE"].seed, seed);

        handler.market_for("LAVE", &economy, MARKET_REFRESH_JUMPS - 1);
        assert_eq!(handler.markets["LAVE"].seed, seed);

        handler.market_for("LAVE", &economy, MARKET_REFRESH_JUMPS);
        assert_ne!(handler.markets["LAVE"].seed, seed);
        assert_eq!(handler.markets["LAVE"].rolled_at, MARKET_REFRESH_JUMPS);
    }

    #[test]
    fn test_trading_handler_creation() {
        let handler = TradingHandler::new();
        assert_eq!(handler.trading_system.get_commodities().len(), 8);
    }

    fn market(buy: u32, sell: u32, available: u32) -> MarketPrices {
        let mut m = MarketPrices::new();
        m.buy_prices.insert(CommodityType::Food, buy);
        m.sell_prices.insert(CommodityType::Food, sell);
        m.availability.insert(CommodityType::Food, available);
        m
    }

    #[test]
    fn successful_buy_and_sell_are_logged() {
        let mut handler = TradingHandler::new();
        let mut commander = Commander::new("Test".to_string());
        let mut ship = Ship::new_cobra_mk3();
        let mut log = TradeLog::new();

        assert!(handler.execute_buy(
            CommodityType::Food,
            5,
            &mut commander,
            &mut ship,
            &market(10, 8, 20),
            &mut log
        ));
        assert!(handler.execute_sell(
            CommodityType::Food,
            5,
            &mut commander,
            &mut ship,
            &market(12, 15, 20),
            &mut log
        ));

        assert_eq!(log.transactions().len(), 2);
        assert_eq!(log.transactions()[1].profit, Some(25));
        assert_eq!(log.transactions()[0].location, commander.location());
    }

    #[test]
    fn failed_buy_and_sell_are_not_logged() {
        let mut handler = TradingHandler::new();
        let mut commander = Commander::new("Test".to_string());
        let mut ship = Ship::new_cobra_mk3();
        let mut log = TradeLog::new();

        // too expensive, not enough stock, nothing to sell
        assert!(!handler.execute_buy(
            CommodityType::Food,
            5,
            &mut commander,
            &mut ship,
            &market(1_000_000, 8, 20),
            &mut log
        ));
        assert!(!handler.execute_buy(
            CommodityType::Food,
            5,
            &mut commander,
            &mut ship,
            &market(10, 8, 2),
            &mut log
        ));
        assert!(!handler.execute_sell(
            CommodityType::Food,
            5,
            &mut commander,
            &mut ship,
            &market(10, 8, 20),
            &mut log
        ));

        assert!(log.transactions().is_empty());
    }

    #[test]
    fn buying_reduces_stock_until_the_market_refreshes() {
        let mut handler = TradingHandler::new();
        let economy = EconomyType::RichIndustrial;
        let mut commander = Commander::new("Test".to_string());
        let mut ship = Ship::new_cobra_mk3();
        let mut log = TradeLog::new();

        let before = handler.market_for("LAVE", &economy, 0);
        let stock = before.get_availability(&CommodityType::Food).unwrap();
        assert!(handler.execute_buy(
            CommodityType::Food,
            1,
            &mut commander,
            &mut ship,
            &before,
            &mut log
        ));

        let after = handler.market_for("LAVE", &economy, 0);
        assert_eq!(
            after.get_availability(&CommodityType::Food),
            Some(stock - 1)
        );
        assert_eq!(
            after.get_availability(&CommodityType::Textiles),
            before.get_availability(&CommodityType::Textiles)
        );

        assert_eq!(handler.markets["LAVE"].purchased.len(), 1);
        handler.market_for("DISO", &economy, 1);
        let back = handler.market_for("LAVE", &economy, 2);
        assert_eq!(back.get_availability(&CommodityType::Food), Some(stock - 1));

        handler.market_for("LAVE", &economy, MARKET_REFRESH_JUMPS);
        assert!(handler.markets["LAVE"].purchased.is_empty());
    }

    #[test]
    fn cannot_buy_more_than_remaining_stock() {
        let mut handler = TradingHandler::new();
        let mut commander = Commander::new("Test".to_string());
        let mut ship = Ship::new_cobra_mk3();
        let mut log = TradeLog::new();
        commander.add_credits(1_000);

        let mut stock = market(10, 8, 6);
        assert!(handler.execute_buy(
            CommodityType::Food,
            5,
            &mut commander,
            &mut ship,
            &stock,
            &mut log
        ));
        stock.availability.insert(CommodityType::Food, 1);
        assert!(!handler.execute_buy(
            CommodityType::Food,
            2,
            &mut commander,
            &mut ship,
            &stock,
            &mut log
        ));
    }

    #[test]
    fn system_markets_remember_their_state_across_visits() {
        let mut handler = TradingHandler::new();
        let economy = EconomyType::RichIndustrial;
        handler.market_for("LAVE", &economy, 3);
        handler.market_for("DISO", &economy, 4);

        assert_eq!(handler.markets.len(), 2);
        assert_eq!(handler.markets["LAVE"].rolled_at, 3);
        assert_eq!(handler.markets["DISO"].rolled_at, 4);
    }

    #[test]
    fn markets_survive_serialization() {
        let mut handler = TradingHandler::new();
        handler.market_for("LAVE", &EconomyType::RichIndustrial, 7);
        handler
            .markets
            .get_mut("LAVE")
            .unwrap()
            .purchased
            .insert(CommodityType::Food, 4.0);

        let json = serde_json::to_string(&handler).unwrap();
        let restored: TradingHandler = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.markets["LAVE"].seed, handler.markets["LAVE"].seed);
        assert_eq!(restored.markets["LAVE"].rolled_at, 7);
        assert_eq!(
            restored.markets["LAVE"].purchased.get(&CommodityType::Food),
            Some(&4.0)
        );
    }

    #[test]
    fn old_saves_with_the_single_market_fields_load_with_no_markets() {
        let json = r#"{"trading_system":{"commodities":["Food"]},"market_seed":5,"market_location":"LAVE","purchased":{"Food":2}}"#;
        let restored: TradingHandler = serde_json::from_str(json).unwrap();
        assert!(restored.markets.is_empty());
    }

    fn apply(state: &SystemMarket) -> MarketPrices {
        let mut market = market(100, 90, 50);
        state.apply_to(&mut market);
        market
    }

    #[test]
    fn selling_lowers_the_sell_price_by_one_and_a_half_percent_per_ton() {
        let mut state = SystemMarket::rolled(0);
        state.record_sell(&CommodityType::Food, 10);

        let after = apply(&state);
        assert_eq!(after.get_sell_price(&CommodityType::Food), Some(77));
        assert_eq!(after.get_buy_price(&CommodityType::Food), Some(100));
    }

    #[test]
    fn buying_raises_the_buy_price_and_lowers_stock() {
        let mut state = SystemMarket::rolled(0);
        state.record_buy(&CommodityType::Food, 10);

        let after = apply(&state);
        assert_eq!(after.get_buy_price(&CommodityType::Food), Some(115));
        assert_eq!(after.get_sell_price(&CommodityType::Food), Some(90));
        assert_eq!(after.get_availability(&CommodityType::Food), Some(40));
    }

    #[test]
    fn impact_is_capped_at_forty_percent() {
        let mut state = SystemMarket::rolled(0);
        state.record_sell(&CommodityType::Food, 200);
        state.record_buy(&CommodityType::Food, 200);

        let after = apply(&state);
        assert_eq!(after.get_sell_price(&CommodityType::Food), Some(54));
        assert_eq!(after.get_buy_price(&CommodityType::Food), Some(140));
    }

    #[test]
    fn impact_and_stock_recover_ten_percent_of_the_gap_per_jump() {
        let mut state = SystemMarket::rolled(0);
        state.record_sell(&CommodityType::Food, 20);
        state.record_buy(&CommodityType::Food, 20);

        state.recover(1);
        assert!((state.sold_impact[&CommodityType::Food] - 0.27).abs() < 1e-9);
        assert!((state.purchased[&CommodityType::Food] - 18.0).abs() < 1e-9);

        state.recover(16);
        let left = state.sold_impact[&CommodityType::Food] / 0.30;
        assert!((left - 0.9_f64.powi(16)).abs() < 1e-9);
        assert!(left < 0.2);
    }

    #[test]
    fn recovery_happens_at_systems_the_player_is_not_in() {
        let mut handler = TradingHandler::new();
        let economy = EconomyType::RichIndustrial;
        handler.market_for("LAVE", &economy, 0);
        handler
            .markets
            .get_mut("LAVE")
            .unwrap()
            .record_sell(&CommodityType::Food, 20);

        handler.market_for("DISO", &economy, 3);
        handler.market_for("LAVE", &economy, 3);

        let impact = handler.markets["LAVE"].sold_impact[&CommodityType::Food];
        assert!((impact - 0.30 * 0.9_f64.powi(3)).abs() < 1e-9);
    }

    #[test]
    fn a_refresh_clears_the_impact() {
        let mut handler = TradingHandler::new();
        let economy = EconomyType::RichIndustrial;
        handler.market_for("LAVE", &economy, 0);
        handler
            .markets
            .get_mut("LAVE")
            .unwrap()
            .record_sell(&CommodityType::Food, 20);

        handler.market_for("LAVE", &economy, MARKET_REFRESH_JUMPS);

        assert!(handler.markets["LAVE"].sold_impact.is_empty());
    }

    #[test]
    fn old_markets_without_impact_fields_load_untouched() {
        let json = r#"{"seed":3,"rolled_at":2,"purchased":{"Food":4.0}}"#;
        let state: SystemMarket = serde_json::from_str(json).unwrap();
        assert!(state.sold_impact.is_empty());
        assert_eq!(state.updated_at, 0);
    }

    #[test]
    fn markers_show_for_sixteen_jumps_after_trading() {
        let mut state = SystemMarket::rolled(0);
        state.recover(2);
        state.record_sell(&CommodityType::Food, 5);
        state.record_buy(&CommodityType::Textiles, 5);

        let food = state.pushed(&CommodityType::Food, 2);
        assert_eq!(
            food,
            Pushed {
                sold: true,
                bought: false
            }
        );
        assert_eq!(food.markers(), vec!["▼ sold here"]);
        let textiles = state.pushed(&CommodityType::Textiles, 2);
        assert_eq!(textiles.markers(), vec!["▲ bought here"]);
        assert_eq!(state.pushed(&CommodityType::Minerals, 2), Pushed::default());

        assert!(
            state
                .pushed(&CommodityType::Food, 2 + MARKER_JUMPS - 1)
                .sold
        );
        assert!(!state.pushed(&CommodityType::Food, 2 + MARKER_JUMPS).sold);
    }

    #[test]
    fn a_commodity_both_bought_and_sold_shows_both_markers() {
        let mut state = SystemMarket::rolled(0);
        state.record_buy(&CommodityType::Food, 1);
        state.record_sell(&CommodityType::Food, 1);

        assert_eq!(
            state.pushed(&CommodityType::Food, 0).markers(),
            vec!["▼ sold here", "▲ bought here"]
        );
    }

    #[test]
    fn a_refresh_clears_the_markers() {
        let mut handler = TradingHandler::new();
        let economy = EconomyType::RichIndustrial;
        handler.market_for("LAVE", &economy, 0);
        handler
            .markets
            .get_mut("LAVE")
            .unwrap()
            .record_sell(&CommodityType::Food, 3);
        assert!(!handler.pushed_commodities("LAVE", 1).is_empty());

        handler.market_for("LAVE", &economy, MARKET_REFRESH_JUMPS);

        assert!(handler
            .pushed_commodities("LAVE", MARKET_REFRESH_JUMPS)
            .is_empty());
    }
}
