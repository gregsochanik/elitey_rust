use elite_core::data::commodities::CommodityType;
use elite_core::game::trade_log::TradeKind;
use elite_core::game::GameState;
use std::sync::{Arc, Mutex};

uniffi::setup_scaffolding!();

#[derive(Debug, thiserror::Error, uniffi::Error)]
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

#[derive(Debug, Clone, uniffi::Record)]
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

#[derive(Debug, Clone, uniffi::Record)]
pub struct MarketData {
    pub system_name: String,
    pub credits: u32,
    pub cargo_used: u32,
    pub cargo_capacity: u32,
    pub rows: Vec<MarketRowData>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct TradeResult {
    pub message: String,
    pub credits_after: u32,
}

#[derive(uniffi::Object)]
pub struct Game {
    state: Mutex<GameState>,
}

#[uniffi::export]
impl Game {
    #[uniffi::constructor]
    pub fn new(commander_name: String) -> Arc<Self> {
        Arc::new(Self {
            state: Mutex::new(GameState::new(commander_name)),
        })
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
