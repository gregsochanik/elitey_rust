use crate::data::commodities::{CommodityType, MarketPrices, TradingSystem};
use crate::data::systems::{EconomyType, StarSystem, SystemManager};
use crate::game::commander::Standing;
use crate::game::{Commander, Ship, TradeLog};
use crate::utils::display::{show_error, show_info, show_success};
use colored::*;
use std::collections::HashMap;

const ILLEGAL_TRADE_REPUTATION: i32 = -2;

/// Trading command handler
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TradingHandler {
    trading_system: TradingSystem,
    /// Each system's market, kept between visits
    #[serde(default)]
    markets: HashMap<String, SystemMarket>,
    /// Where the player last traded, to notice an arrival elsewhere
    #[serde(default)]
    market_location: String,
}

/// What a system's market remembers: its price roll, when it was rolled and what was bought
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct SystemMarket {
    pub seed: u64,
    pub rolled_at: u32,
    /// Tons bought since the roll, per commodity
    pub purchased: HashMap<CommodityType, u32>,
}

impl TradingHandler {
    pub fn new() -> Self {
        Self {
            trading_system: TradingSystem::new(),
            markets: HashMap::new(),
            market_location: String::new(),
        }
    }

    /// Prices stay fixed while the player stays in a system and re-roll on arrival elsewhere
    fn market_for(&mut self, location: &str, economy: &EconomyType, clock: u32) -> MarketPrices {
        if self.market_location != location || !self.markets.contains_key(location) {
            self.market_location = location.to_string();
            self.markets.insert(
                location.to_string(),
                SystemMarket {
                    seed: rand::random(),
                    rolled_at: clock,
                    purchased: HashMap::new(),
                },
            );
        }
        let state = &self.markets[location];
        let mut market = self
            .trading_system
            .generate_market_prices(economy, state.seed);
        for (commodity, bought) in &state.purchased {
            if let Some(stock) = market.availability.get_mut(commodity) {
                *stock = stock.saturating_sub(*bought);
            }
        }
        market
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
            self.display_market(system, &market, standing);
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
        let name_lower = name.to_lowercase();
        match name_lower.as_str() {
            "food" => Some(CommodityType::Food),
            "textiles" => Some(CommodityType::Textiles),
            "machinery" => Some(CommodityType::Machinery),
            "computers" => Some(CommodityType::Computers),
            "minerals" => Some(CommodityType::Minerals),
            "alloys" => Some(CommodityType::Alloys),
            "luxury" => Some(CommodityType::Luxury),
            "narcotics" => Some(CommodityType::Narcotics),
            _ => None,
        }
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
            *state.purchased.entry(commodity.clone()).or_insert(0) += quantity;
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
        &self,
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
    fn display_market(&self, system: &StarSystem, market: &MarketPrices, standing: Standing) {
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

            println!(
                "{:<12} {:<12} {:<12} {:<20} {}",
                commodity_name, buy_str, sell_str, avail_str, base_coloured
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

        if cargo_hold.is_empty() {
            println!("Cargo hold is empty");
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
    fn prices_are_stable_within_a_system_and_reroll_on_arrival() {
        let mut handler = TradingHandler::new();
        let economy = EconomyType::RichIndustrial;

        let first = handler.market_for("LAVE", &economy, 0);
        let second = handler.market_for("LAVE", &economy, 0);
        assert_eq!(first.buy_prices, second.buy_prices);
        assert_eq!(first.availability, second.availability);

        let seed_at_lave = handler.markets["LAVE"].seed;
        handler.market_for("DISO", &economy, 1);
        handler.market_for("LAVE", &economy, 2);
        assert_ne!(handler.markets["LAVE"].seed, seed_at_lave);
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
    fn buying_reduces_stock_until_the_player_leaves() {
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
        handler.market_for("LAVE", &economy, 2);
        assert!(handler.markets["LAVE"].purchased.is_empty());
    }

    #[test]
    fn cannot_buy_more_than_remaining_stock() {
        let mut handler = TradingHandler::new();
        let mut commander = Commander::new("Test".to_string());
        let mut ship = Ship::new_cobra_mk3();
        let mut log = TradeLog::new();
        commander.add_credits(1_000);

        handler.market_location = "LAVE".to_string();
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
            .insert(CommodityType::Food, 4);

        let json = serde_json::to_string(&handler).unwrap();
        let restored: TradingHandler = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.markets["LAVE"].seed, handler.markets["LAVE"].seed);
        assert_eq!(restored.markets["LAVE"].rolled_at, 7);
        assert_eq!(
            restored.markets["LAVE"].purchased.get(&CommodityType::Food),
            Some(&4)
        );
    }

    #[test]
    fn old_saves_with_the_single_market_fields_load_with_no_markets() {
        let json = r#"{"trading_system":{"commodities":["Food"]},"market_seed":5,"market_location":"LAVE","purchased":{"Food":2}}"#;
        let restored: TradingHandler = serde_json::from_str(json).unwrap();
        assert!(restored.markets.is_empty());
    }
}
