use crate::data::commodities::{CommodityType, MarketPrices, TradingSystem};
use crate::data::systems::{EconomyType, SystemManager};
use crate::game::commander::{ReputationChange, Standing};
use crate::game::trade_log::TradeKind;
use crate::game::{Commander, Ship, TradeLog};
use std::collections::HashMap;
use thiserror::Error;

const ILLEGAL_TRADE_REPUTATION: i32 = -2;
const MARKET_REFRESH_JUMPS: u32 = 20;
const IMPACT_PER_TON: f64 = 0.015;
const MAX_IMPACT: f64 = 0.40;
const RECOVERY_PER_JUMP: f64 = 0.10;
const MARKER_JUMPS: u32 = 16;

/// Why a buy or sell was refused
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum TradeError {
    #[error("System not found")]
    UnknownSystem,
    #[error("Commodity not available in this system")]
    NotSoldHere,
    #[error("Commodity not available for sale")]
    NotBoughtHere,
    #[error("Only {available} tons available, requested {requested}")]
    NotEnoughStock { available: u32, requested: u32 },
    #[error("Insufficient cargo space. Need {needed} tons, have {available} tons available")]
    NoCargoSpace { needed: u32, available: u32 },
    #[error("Insufficient credits. Need {needed}cr, have {have}cr")]
    NotEnoughCredits { needed: u32, have: u32 },
    #[error("Insufficient cargo. Have {held} tons, trying to sell {requested}")]
    NotEnoughCargo { held: u32, requested: u32 },
}

/// What a completed buy or sell did
#[derive(Debug, Clone, PartialEq)]
pub struct TradeReceipt {
    pub kind: TradeKind,
    pub commodity: CommodityType,
    pub quantity: u32,
    pub unit_price: u32,
    pub total: u32,
    pub credits_after: u32,
    /// Set when the commodity is illegal and the trade cost reputation
    pub illegal_penalty: Option<ReputationChange>,
}

/// One commodity in the market table
#[derive(Debug, Clone, PartialEq)]
pub struct MarketRow {
    pub commodity: CommodityType,
    pub buy_price: u32,
    pub sell_price: u32,
    pub available: u32,
    pub percent_vs_base: i32,
    pub pushed: Pushed,
    /// Tons of this commodity in the hold
    pub held: u32,
    /// What each held ton cost on average, when it was bought
    pub paid: Option<u32>,
}

/// The market at a system as the player sees it
#[derive(Debug, Clone, PartialEq)]
pub struct MarketView {
    pub system_name: String,
    pub standing: Standing,
    pub rows: Vec<MarketRow>,
}

/// One commodity in the hold
#[derive(Debug, Clone, PartialEq)]
pub struct CargoLine {
    pub commodity: CommodityType,
    pub quantity: u32,
    pub estimated_value: u32,
    /// What each ton cost on average, when it was bought
    pub paid: Option<u32>,
}

/// The cargo hold
#[derive(Debug, Clone, PartialEq)]
pub struct CargoView {
    pub weight: u32,
    pub capacity: u32,
    /// Tons of contract cargo, not for sale
    pub reserved: u32,
    pub items: Vec<CargoLine>,
    pub total_value: u32,
}

/// Trading command handler
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TradingHandler {
    trading_system: TradingSystem,
    /// Every market's price roll is derived from this, the system and the clock
    #[serde(default = "rand::random")]
    world_seed: u64,
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
    fn rolled(seed: u64, clock: u32) -> Self {
        Self {
            seed,
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
        Self::with_seed(rand::random())
    }

    /// A trading handler whose market rolls are fully determined by `world_seed`
    pub fn with_seed(world_seed: u64) -> Self {
        Self {
            trading_system: TradingSystem::new(),
            world_seed,
            markets: HashMap::new(),
        }
    }

    fn market_seed(&self, location: &str, clock: u32) -> u64 {
        let hash = location
            .bytes()
            .fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
                (hash ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3)
            });
        (hash ^ self.world_seed).wrapping_add(u64::from(clock).wrapping_mul(0x9e37_79b9_7f4a_7c15))
    }

    /// A system's prices and stock stay fixed for a number of jumps, whoever arrives or leaves,
    /// and re-roll the next time they are looked at after that
    fn market_for(&mut self, location: &str, economy: &EconomyType, clock: u32) -> MarketPrices {
        let stale = self
            .markets
            .get(location)
            .is_none_or(|state| clock.saturating_sub(state.rolled_at) >= MARKET_REFRESH_JUMPS);
        if stale {
            let seed = self.market_seed(location, clock);
            self.markets
                .insert(location.to_string(), SystemMarket::rolled(seed, clock));
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

    /// The market here as a table of rows, or `None` for an unknown system
    pub fn market_overview(
        &mut self,
        system_manager: &SystemManager,
        location: &str,
        standing: Standing,
        clock: u32,
        ship: &Ship,
        trade_log: &TradeLog,
    ) -> Option<MarketView> {
        let market = self.market_view(system_manager, location, standing, clock)?;
        let pushed = self.pushed_commodities(location, clock);
        let rows = self
            .trading_system
            .get_commodities()
            .iter()
            .map(|commodity| {
                let buy_price = market.get_buy_price(commodity).unwrap_or(0);
                let held = ship.cargo_hold().get_quantity(commodity);
                MarketRow {
                    commodity: commodity.clone(),
                    buy_price,
                    sell_price: market.get_sell_price(commodity).unwrap_or(0),
                    available: market.get_availability(commodity).unwrap_or(0),
                    percent_vs_base: commodity.percent_vs_base(buy_price),
                    pushed: pushed.get(commodity).copied().unwrap_or_default(),
                    held,
                    paid: trade_log.average_cost(commodity, held),
                }
            })
            .collect();
        Some(MarketView {
            system_name: system_manager.get_system(location)?.name.clone(),
            standing,
            rows,
        })
    }

    /// Buy `quantity` tons of `commodity` at the commander's current system
    #[allow(clippy::too_many_arguments)]
    pub fn buy(
        &mut self,
        commodity: CommodityType,
        quantity: u32,
        commander: &mut Commander,
        ship: &mut Ship,
        system_manager: &SystemManager,
        trade_log: &mut TradeLog,
        clock: u32,
    ) -> Result<TradeReceipt, TradeError> {
        let market = self.market_at(commander, system_manager, clock)?;
        self.execute_buy(commodity, quantity, commander, ship, &market, trade_log)
    }

    /// Sell `quantity` tons of `commodity` at the commander's current system
    #[allow(clippy::too_many_arguments)]
    pub fn sell(
        &mut self,
        commodity: CommodityType,
        quantity: u32,
        commander: &mut Commander,
        ship: &mut Ship,
        system_manager: &SystemManager,
        trade_log: &mut TradeLog,
        clock: u32,
    ) -> Result<TradeReceipt, TradeError> {
        let market = self.market_at(commander, system_manager, clock)?;
        self.execute_sell(commodity, quantity, commander, ship, &market, trade_log)
    }

    fn market_at(
        &mut self,
        commander: &Commander,
        system_manager: &SystemManager,
        clock: u32,
    ) -> Result<MarketPrices, TradeError> {
        let location = commander.location();
        let system = system_manager
            .get_system(location)
            .ok_or(TradeError::UnknownSystem)?;
        Ok(self.priced_market(location, &system.economy, commander.standing(), clock))
    }

    /// The cargo hold as lines with a rough value
    pub fn cargo_view(&self, ship: &Ship, trade_log: &TradeLog) -> CargoView {
        let hold = ship.cargo_hold();
        let items: Vec<CargoLine> = hold
            .get_items()
            .iter()
            .map(|item| CargoLine {
                commodity: item.commodity.clone(),
                quantity: item.quantity,
                estimated_value: item.quantity * item.commodity.base_price(),
                paid: trade_log.average_cost(&item.commodity, item.quantity),
            })
            .collect();
        CargoView {
            weight: hold.current_weight(),
            capacity: ship.cargo_capacity(),
            reserved: hold.reserved,
            total_value: items.iter().map(|line| line.estimated_value).sum(),
            items,
        }
    }

    /// Parse commodity name to CommodityType
    pub fn parse_commodity(&self, name: &str) -> Option<CommodityType> {
        CommodityType::from_string(name)
    }

    fn execute_buy(
        &mut self,
        commodity: CommodityType,
        quantity: u32,
        commander: &mut Commander,
        ship: &mut Ship,
        market: &MarketPrices,
        trade_log: &mut TradeLog,
    ) -> Result<TradeReceipt, TradeError> {
        let available = market
            .get_availability(&commodity)
            .ok_or(TradeError::NotSoldHere)?;
        if available < quantity {
            return Err(TradeError::NotEnoughStock {
                available,
                requested: quantity,
            });
        }
        if !ship.has_cargo_space(quantity) {
            return Err(TradeError::NoCargoSpace {
                needed: quantity,
                available: ship.cargo_available(),
            });
        }
        let unit_price = market
            .get_buy_price(&commodity)
            .ok_or(TradeError::NotSoldHere)?;
        let total = unit_price * quantity;
        if !commander.has_credits(total) {
            return Err(TradeError::NotEnoughCredits {
                needed: total,
                have: commander.credits(),
            });
        }

        let cargo_before = ship.cargo_hold().get_quantity(&commodity);
        commander.remove_credits(total);
        ship.add_cargo(commodity.clone(), quantity);
        if let Some(state) = self.markets.get_mut(commander.location()) {
            state.record_buy(&commodity, quantity);
        }
        trade_log.record_buy(
            commodity.clone(),
            quantity,
            unit_price,
            commander.location(),
            cargo_before,
        );

        let illegal_penalty = Self::penalise_illegal_trade(&commodity, commander);
        Ok(TradeReceipt {
            kind: TradeKind::Buy,
            commodity,
            quantity,
            unit_price,
            total,
            credits_after: commander.credits(),
            illegal_penalty,
        })
    }

    fn execute_sell(
        &mut self,
        commodity: CommodityType,
        quantity: u32,
        commander: &mut Commander,
        ship: &mut Ship,
        market: &MarketPrices,
        trade_log: &mut TradeLog,
    ) -> Result<TradeReceipt, TradeError> {
        let held = ship.cargo_hold().get_quantity(&commodity);
        if held < quantity {
            return Err(TradeError::NotEnoughCargo {
                held,
                requested: quantity,
            });
        }
        let unit_price = market
            .get_sell_price(&commodity)
            .ok_or(TradeError::NotBoughtHere)?;
        let total = unit_price * quantity;

        ship.remove_cargo(&commodity, quantity);
        commander.add_credits(total);
        if let Some(state) = self.markets.get_mut(commander.location()) {
            state.record_sell(&commodity, quantity);
        }
        trade_log.record_sell(
            commodity.clone(),
            quantity,
            unit_price,
            commander.location(),
            held,
        );

        let illegal_penalty = Self::penalise_illegal_trade(&commodity, commander);
        Ok(TradeReceipt {
            kind: TradeKind::Sell,
            commodity,
            quantity,
            unit_price,
            total,
            credits_after: commander.credits(),
            illegal_penalty,
        })
    }

    fn penalise_illegal_trade(
        commodity: &CommodityType,
        commander: &mut Commander,
    ) -> Option<ReputationChange> {
        commodity
            .is_illegal()
            .then(|| commander.change_reputation(ILLEGAL_TRADE_REPUTATION))
    }
}

impl Default for TradingHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests;
