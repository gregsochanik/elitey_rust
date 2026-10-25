use elite_ffi::{
    CargoData, Game, MarketData, StatusData, TradeLogData, TradeRecordData, TradeResult,
};
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

    #[func]
    fn cargo(&self) -> VarDictionary {
        cargo_to_dictionary(self.game.cargo())
    }

    #[func]
    fn trade_log(&self, count: i64) -> VarDictionary {
        log_to_dictionary(self.game.trade_log(count.max(0) as u32))
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

fn cargo_to_dictionary(cargo: CargoData) -> VarDictionary {
    let mut items = VarArray::new();
    for line in cargo.items {
        items.push(
            &vdict! {
                "id": line.id,
                "name": line.name,
                "quantity": line.quantity,
                "estimated_value": line.estimated_value,
                "paid": line.paid.map_or(-1, i64::from),
            }
            .to_variant(),
        );
    }
    vdict! {
        "weight": cargo.weight,
        "capacity": cargo.capacity,
        "reserved": cargo.reserved,
        "items": items,
        "total_value": cargo.total_value,
    }
}

fn record_to_dictionary(record: &TradeRecordData) -> VarDictionary {
    vdict! {
        "bought": record.bought,
        "commodity": record.commodity.clone(),
        "quantity": record.quantity,
        "location": record.location.clone(),
        "total": record.total,
        "has_profit": record.profit.is_some(),
        "profit": record.profit.unwrap_or(0),
    }
}

fn log_to_dictionary(log: TradeLogData) -> VarDictionary {
    let mut recent = VarArray::new();
    for record in &log.recent {
        recent.push(&record_to_dictionary(record).to_variant());
    }
    vdict! {
        "recent": recent,
        "total_trades": log.total_trades,
        "net_profit": log.net_profit,
        "has_success_rate": log.success_rate.is_some(),
        "success_rate": log.success_rate.unwrap_or(0.0),
        "has_best": log.best.is_some(),
        "best": log.best.as_ref().map_or_else(VarDictionary::new, record_to_dictionary),
    }
}
