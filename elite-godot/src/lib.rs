use elite_ffi::{Game, MarketData, StatusData, TradeResult};
use godot::prelude::*;
use std::sync::Arc;

struct EliteExtension;

#[gdextension]
unsafe impl ExtensionLibrary for EliteExtension {}

/// The game as GDScript sees it: plain dictionaries and arrays, no Rust types
#[derive(GodotClass)]
#[class(base=RefCounted)]
struct EliteGame {
    game: Arc<Game>,
    base: Base<RefCounted>,
}

#[godot_api]
impl IRefCounted for EliteGame {
    fn init(base: Base<RefCounted>) -> Self {
        Self {
            game: Game::new("Commander".to_string()),
            base,
        }
    }
}

#[godot_api]
impl EliteGame {
    /// Commander and ship numbers for the header and status screen
    #[func]
    fn status(&self) -> VarDictionary {
        status_to_dictionary(self.game.status())
    }

    /// The market here: system, credits, cargo and one dictionary per commodity
    #[func]
    fn market(&self) -> VarDictionary {
        match self.game.market() {
            Ok(market) => market_to_dictionary(market),
            Err(error) => vdict! { "error": error.to_string() },
        }
    }

    /// `{ ok, message, credits }`
    #[func]
    fn buy(&self, commodity_id: GString, quantity: i64) -> VarDictionary {
        trade_to_dictionary(self.game.buy(commodity_id.to_string(), quantity as u32))
    }

    #[func]
    fn sell(&self, commodity_id: GString, quantity: i64) -> VarDictionary {
        trade_to_dictionary(self.game.sell(commodity_id.to_string(), quantity as u32))
    }
}

fn market_to_dictionary(market: MarketData) -> VarDictionary {
    let mut rows = VarArray::new();
    for row in market.rows {
        rows.push(
            &vdict! {
                "id": row.id,
                "name": row.name,
                "buy_price": row.buy_price,
                "sell_price": row.sell_price,
                "available": row.available,
                "percent_vs_base": row.percent_vs_base,
                "held": row.held,
                "paid": row.paid.map_or(-1, i64::from),
            }
            .to_variant(),
        );
    }
    vdict! {
        "system_name": market.system_name,
        "credits": market.credits,
        "cargo_used": market.cargo_used,
        "cargo_capacity": market.cargo_capacity,
        "rows": rows,
    }
}

fn trade_to_dictionary(result: Result<TradeResult, elite_ffi::GameError>) -> VarDictionary {
    match result {
        Ok(trade) => vdict! {
            "ok": true,
            "message": trade.message,
            "credits": trade.credits_after,
        },
        Err(error) => vdict! { "ok": false, "message": error.to_string() },
    }
}

fn status_to_dictionary(status: StatusData) -> VarDictionary {
    vdict! {
        "commander_name": status.commander_name,
        "credits": status.credits,
        "location": status.location,
        "reputation": status.reputation,
        "standing": status.standing,
        "combat_rating": status.combat_rating,
        "kills": status.kills,
        "ship_name": status.ship_name,
        "cargo_used": status.cargo_used,
        "cargo_capacity": status.cargo_capacity,
        "fuel_current": status.fuel_current,
        "fuel_capacity": status.fuel_capacity,
        "hull": status.hull,
        "max_hull": status.max_hull,
        "power_used": status.power_used,
        "power_capacity": status.power_capacity,
    }
}
