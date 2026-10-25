use elite_core::data::commodities::CommodityType;
use elite_core::game::refuel::{RefuelError, RefuelOutcome, CREDITS_PER_UNIT};
use elite_core::game::repair::CREDITS_PER_POINT;
use elite_core::game::trade_log::TradeKind;
use elite_core::game::GameState;
use std::sync::{Arc, Mutex};

#[derive(Debug, thiserror::Error)]
pub enum GameError {
    #[error("{message}")]
    Refused { message: String },
}

impl GameError {
    fn refused(message: impl ToString) -> Self {
        GameError::Refused {
            message: message.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MarketRowData {
    pub id: String,
    pub name: String,
    pub buy_price: u32,
    pub sell_price: u32,
    pub available: u32,
    pub percent_vs_base: i32,
    pub held: u32,
    pub paid: Option<u32>,
    pub bought_here: bool,
    pub sold_here: bool,
}

#[derive(Debug, Clone)]
pub struct MarketData {
    pub system_name: String,
    pub credits: u32,
    pub cargo_used: u32,
    pub cargo_capacity: u32,
    pub rows: Vec<MarketRowData>,
}

#[derive(Debug, Clone)]
pub struct StatusData {
    pub commander_name: String,
    pub credits: u32,
    pub location: String,
    pub reputation: i32,
    pub standing: String,
    pub combat_rating: String,
    pub kills: u32,
    pub ship_name: String,
    pub cargo_used: u32,
    pub cargo_capacity: u32,
    pub fuel_current: u32,
    pub fuel_capacity: u32,
    pub hull: u32,
    pub max_hull: u32,
    pub power_used: u32,
    pub power_capacity: u32,
}

#[derive(Debug, Clone)]
pub struct CargoLineData {
    pub id: String,
    pub name: String,
    pub quantity: u32,
    pub estimated_value: u32,
    pub paid: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct CargoData {
    pub weight: u32,
    pub capacity: u32,
    pub reserved: u32,
    pub items: Vec<CargoLineData>,
    pub total_value: u32,
}

#[derive(Debug, Clone)]
pub struct TradeRecordData {
    pub bought: bool,
    pub commodity: String,
    pub quantity: u32,
    pub location: String,
    pub total: u32,
    pub profit: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct TradeLogData {
    pub recent: Vec<TradeRecordData>,
    pub total_trades: u32,
    pub net_profit: i64,
    pub success_rate: Option<f64>,
    pub best: Option<TradeRecordData>,
}

#[derive(Debug, Clone)]
pub struct ServicesData {
    pub credits: u32,
    pub fuel_price: u32,
    pub fuel_missing: u32,
    pub repair_price: u32,
    pub hull_missing: u32,
}

/// What a service did, ready to show
#[derive(Debug, Clone)]
pub struct ActionResult {
    pub ok: bool,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct TradeResult {
    pub message: String,
    pub credits_after: u32,
}

pub struct Game {
    state: Mutex<GameState>,
}

impl Game {
    pub fn new(commander_name: String) -> Arc<Self> {
        Arc::new(Self {
            state: Mutex::new(GameState::new(commander_name)),
        })
    }

    pub fn status(&self) -> StatusData {
        let view = self.state.lock().expect("game state lock").status_view();
        StatusData {
            commander_name: view.commander_name,
            credits: view.credits,
            location: view.location,
            reputation: view.reputation,
            standing: view.standing.name().to_string(),
            combat_rating: view.combat_rating.name().to_string(),
            kills: view.kills,
            ship_name: view.ship_name,
            cargo_used: view.cargo_used,
            cargo_capacity: view.cargo_capacity,
            fuel_current: view.fuel_current,
            fuel_capacity: view.fuel_capacity,
            hull: view.hull,
            max_hull: view.max_hull,
            power_used: view.power_used,
            power_capacity: view.power_capacity,
        }
    }

    pub fn cargo(&self) -> CargoData {
        let view = self.state.lock().expect("game state lock").cargo_view();
        CargoData {
            weight: view.weight,
            capacity: view.capacity,
            reserved: view.reserved,
            items: view
                .items
                .into_iter()
                .map(|line| CargoLineData {
                    id: line.commodity.id().to_string(),
                    name: line.commodity.name().to_string(),
                    quantity: line.quantity,
                    estimated_value: line.estimated_value,
                    paid: line.paid,
                })
                .collect(),
            total_value: view.total_value,
        }
    }

    pub fn trade_log(&self, count: u32) -> TradeLogData {
        let view = self
            .state
            .lock()
            .expect("game state lock")
            .trade_log_view(count as usize);
        TradeLogData {
            recent: view.recent.iter().map(trade_record).collect(),
            total_trades: view.stats.total_trades as u32,
            net_profit: view.stats.net_profit,
            success_rate: view.stats.success_rate,
            best: view.best.as_ref().map(trade_record),
        }
    }

    pub fn services(&self) -> ServicesData {
        let state = self.state.lock().expect("game state lock");
        let status = state.status_view();
        ServicesData {
            credits: status.credits,
            fuel_price: CREDITS_PER_UNIT,
            fuel_missing: status.fuel_capacity.saturating_sub(status.fuel_current),
            repair_price: CREDITS_PER_POINT,
            hull_missing: status.max_hull.saturating_sub(status.hull),
        }
    }

    /// Buy `units` of fuel, or all that fits and is affordable when `None`
    pub fn refuel(&self, units: Option<u32>) -> ActionResult {
        let mut state = self.state.lock().expect("game state lock");
        match state.refuel(units) {
            Ok(RefuelOutcome::Bought {
                units,
                cost,
                fuel,
                capacity,
            }) => ok(format!(
                "Bought {} light years of fuel for {} credits. Fuel: {}/{}",
                units, cost, fuel, capacity
            )),
            Ok(RefuelOutcome::Rescued {
                fuel,
                capacity,
                change,
            }) => ok(format!(
                "Stranded! A rescue crew tops up your tank to {}/{}. Reputation {:+}.",
                fuel, capacity, change.applied
            )),
            Err(RefuelError::CannotAfford { stranded: true }) => failed(
                "Fuel costs credits you don't have. You are stranded: refuel with no amount to call a rescue crew.",
            ),
            Err(error) => failed(error),
        }
    }

    /// Repair `points` of hull, or all that is affordable when `None`
    pub fn repair(&self, points: Option<u32>) -> ActionResult {
        let mut state = self.state.lock().expect("game state lock");
        match state.repair(points) {
            Ok(receipt) => ok(format!(
                "Repaired {} hull points for {} credits. Hull: {}/{}",
                receipt.points, receipt.cost, receipt.hull, receipt.max_hull
            )),
            Err(error) => failed(error),
        }
    }

    pub fn market(&self) -> Result<MarketData, GameError> {
        let mut state = self.state.lock().expect("game state lock");
        let view = state
            .market_overview()
            .ok_or_else(|| GameError::refused("Unable to determine current location"))?;
        Ok(MarketData {
            system_name: view.system_name,
            credits: state.commander.credits(),
            cargo_used: state.ship.cargo_used(),
            cargo_capacity: state.ship.cargo_capacity(),
            rows: view
                .rows
                .into_iter()
                .map(|row| MarketRowData {
                    id: row.commodity.id().to_string(),
                    name: row.commodity.name().to_string(),
                    buy_price: row.buy_price,
                    sell_price: row.sell_price,
                    available: row.available,
                    percent_vs_base: row.percent_vs_base,
                    held: row.held,
                    paid: row.paid,
                    bought_here: row.pushed.bought,
                    sold_here: row.pushed.sold,
                })
                .collect(),
        })
    }

    pub fn buy(&self, commodity_id: String, quantity: u32) -> Result<TradeResult, GameError> {
        let commodity = parse_commodity(&commodity_id)?;
        let mut state = self.state.lock().expect("game state lock");
        let receipt = state.buy(commodity, quantity).map_err(GameError::refused)?;
        Ok(TradeResult {
            message: format!(
                "{} {} tons of {} for {}cr",
                verb(receipt.kind),
                receipt.quantity,
                receipt.commodity.name(),
                receipt.total
            ),
            credits_after: receipt.credits_after,
        })
    }

    pub fn sell(&self, commodity_id: String, quantity: u32) -> Result<TradeResult, GameError> {
        let commodity = parse_commodity(&commodity_id)?;
        let mut state = self.state.lock().expect("game state lock");
        let receipt = state
            .sell(commodity, quantity)
            .map_err(GameError::refused)?;
        Ok(TradeResult {
            message: format!(
                "{} {} tons of {} for {}cr",
                verb(receipt.kind),
                receipt.quantity,
                receipt.commodity.name(),
                receipt.total
            ),
            credits_after: receipt.credits_after,
        })
    }
}

fn ok(message: String) -> ActionResult {
    ActionResult { ok: true, message }
}

fn failed(message: impl ToString) -> ActionResult {
    ActionResult {
        ok: false,
        message: message.to_string(),
    }
}

fn trade_record(transaction: &elite_core::game::trade_log::Transaction) -> TradeRecordData {
    TradeRecordData {
        bought: transaction.kind == TradeKind::Buy,
        commodity: transaction.commodity.name().to_string(),
        quantity: transaction.quantity,
        location: transaction.location.clone(),
        total: transaction.total(),
        profit: transaction.profit,
    }
}

fn parse_commodity(id: &str) -> Result<CommodityType, GameError> {
    CommodityType::from_string(id)
        .ok_or_else(|| GameError::refused(format!("Unknown commodity: {}", id)))
}

fn verb(kind: TradeKind) -> &'static str {
    match kind {
        TradeKind::Buy => "Bought",
        TradeKind::Sell => "Sold",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_market_lists_every_commodity_with_prices() {
        let game = Game::new("Test".to_string());

        let market = game.market().unwrap();

        assert_eq!(market.system_name, "LAVE");
        assert_eq!(market.rows.len(), CommodityType::all().len());
        assert!(market.rows.iter().all(|row| row.buy_price > 0));
    }

    #[test]
    fn the_status_describes_the_commander_and_ship() {
        let game = Game::new("Test".to_string());

        let status = game.status();

        assert_eq!(status.commander_name, "Test");
        assert_eq!(status.location, "LAVE");
        assert_eq!(status.standing, "Neutral");
        assert_eq!(status.combat_rating, "Harmless");
        assert_eq!((status.power_used, status.power_capacity), (9, 20));
        assert_eq!((status.fuel_current, status.fuel_capacity), (7, 7));
    }

    #[test]
    fn the_cargo_lists_what_is_held_and_what_it_cost() {
        let game = Game::new("Test".to_string());
        assert!(game.cargo().items.is_empty());

        game.buy("food".to_string(), 3).unwrap();
        let cargo = game.cargo();

        assert_eq!((cargo.weight, cargo.capacity), (3, 20));
        assert_eq!(cargo.items.len(), 1);
        assert_eq!(cargo.items[0].name, "Food");
        assert!(cargo.items[0].paid.is_some());
        assert!(cargo.total_value > 0);
    }

    #[test]
    fn the_trade_log_lists_recent_trades_and_totals() {
        let game = Game::new("Test".to_string());
        game.buy("food".to_string(), 2).unwrap();
        game.sell("food".to_string(), 2).unwrap();

        let log = game.trade_log(10);

        assert_eq!(log.total_trades, 2);
        assert_eq!(log.recent.len(), 2);
        assert!(log.recent.iter().any(|t| t.bought));
        assert!(log.recent.iter().any(|t| !t.bought && t.profit.is_some()));
        let profit = log.recent.iter().find_map(|t| t.profit).unwrap();
        assert_eq!(log.best.is_some(), profit > 0);
        assert_eq!(log.net_profit, profit);
        assert_eq!(game.trade_log(1).recent.len(), 1);
    }

    #[test]
    fn services_report_what_is_missing_and_what_it_costs() {
        let game = Game::new("Test".to_string());
        let services = game.services();
        assert_eq!((services.fuel_missing, services.hull_missing), (0, 0));
        assert_eq!((services.fuel_price, services.repair_price), (2, 2));

        let result = game.refuel(None);
        assert!(!result.ok);
        assert_eq!(result.message, "Your tank is already full.");
        assert!(!game.repair(None).ok);
    }

    #[test]
    fn refuelling_and_repairing_spend_credits_and_restore_levels() {
        let game = Game::new("Test".to_string());
        {
            let mut state = game.state.lock().unwrap();
            state.ship.fuel_current = 2;
            state.ship.take_damage(10);
        }
        assert_eq!(game.services().fuel_missing, 5);
        assert_eq!(game.services().hull_missing, 10);

        let fuel = game.refuel(Some(3));
        let hull = game.repair(Some(4));

        assert!(fuel.ok && hull.ok, "{} / {}", fuel.message, hull.message);
        let services = game.services();
        assert_eq!((services.fuel_missing, services.hull_missing), (2, 6));
        assert_eq!(services.credits, 100 - 6 - 8);
    }

    #[test]
    fn buying_and_selling_move_credits_and_cargo() {
        let game = Game::new("Test".to_string());

        let bought = game.buy("food".to_string(), 2).unwrap();
        let market = game.market().unwrap();
        let food = market.rows.iter().find(|row| row.id == "food").unwrap();
        assert_eq!(food.held, 2);
        assert_eq!(market.cargo_used, 2);
        assert_eq!(market.credits, bought.credits_after);

        let sold = game.sell("food".to_string(), 2).unwrap();
        assert!(sold.message.starts_with("Sold 2 tons of Food"));
        assert_eq!(game.market().unwrap().cargo_used, 0);
    }

    #[test]
    fn refusals_carry_the_core_message() {
        let game = Game::new("Test".to_string());

        let error = game.buy("food".to_string(), 999).unwrap_err();
        assert!(error.to_string().contains("Only") || error.to_string().contains("Insufficient"));
        assert!(game.buy("gold".to_string(), 1).is_err());
    }
}
