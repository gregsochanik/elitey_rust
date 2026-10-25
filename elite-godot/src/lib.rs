use elite_ffi::{
    ActionResult, CargoData, EquipmentShopData, FightData, FighterData, GalaxyData, Game,
    InstalledData, JobBoardData, JumpPlanData, MarketData, ShipyardData, StatusData, StepData,
    SystemDetailData, TradeLogData, TradeRecordData, TradeResult,
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
            game: match std::env::var("ELITE_SEED")
                .ok()
                .and_then(|seed| seed.parse().ok())
            {
                Some(seed) => Game::with_seed("Commander".to_string(), seed),
                None => Game::new("Commander".to_string()),
            },
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

    /// Every save, newest first: `slot`, `commander`, `location`, `credits`, `ship`, `saved_at`, `is_autosave`
    #[func]
    fn saves(&self) -> VarArray {
        let mut saves = VarArray::new();
        for save in self.game.saves() {
            saves.push(
                &vdict! {
                    "slot": save.slot,
                    "commander": save.commander,
                    "location": save.location,
                    "credits": save.credits,
                    "ship": save.ship,
                    "saved_at": save.saved_at,
                    "is_autosave": save.is_autosave,
                }
                .to_variant(),
            );
        }
        saves
    }

    #[func]
    fn autosave_exists(&self, commander_name: GString) -> bool {
        self.game.autosave_exists(commander_name.to_string())
    }

    #[func]
    fn new_game(&self, commander_name: GString) -> VarDictionary {
        action_to_dictionary(self.game.new_game(commander_name.to_string()))
    }

    #[func]
    fn continue_game(&self, commander_name: GString) -> VarDictionary {
        action_to_dictionary(self.game.continue_game(commander_name.to_string()))
    }

    #[func]
    fn load_slot(&self, slot: GString) -> VarDictionary {
        action_to_dictionary(self.game.load_slot(slot.to_string()))
    }

    #[func]
    fn save_slot(&self, slot: GString) -> VarDictionary {
        action_to_dictionary(self.game.save_slot(slot.to_string()))
    }

    #[func]
    fn delete_slot(&self, slot: GString) -> VarDictionary {
        action_to_dictionary(self.game.delete_slot(slot.to_string()))
    }

    #[func]
    fn autosave(&self) -> VarDictionary {
        action_to_dictionary(self.game.autosave())
    }

    /// Start a jump; returns the first step: `kind` is event, fight, inspection, arrived or error
    #[func]
    fn begin_jump(&self, destination: GString) -> VarDictionary {
        step_to_dictionary(self.game.begin_jump(destination.to_string()))
    }

    #[func]
    fn answer_event(&self, choice: i64) -> VarDictionary {
        step_to_dictionary(self.game.answer_event(choice.max(0) as u32))
    }

    /// `bribe`, `run` or `hand_over`
    #[func]
    fn answer_inspection(&self, choice: GString) -> VarDictionary {
        step_to_dictionary(self.game.answer_inspection(choice.to_string()))
    }

    /// The fight now waiting: `active`, `player`, `enemy`, `outcome`, `log` (everything so far), `message`
    #[func]
    fn fight(&self) -> VarDictionary {
        fight_to_dictionary(self.game.fight())
    }

    /// Play a round: `attack`, `flee` or `surrender`
    #[func]
    fn fight_action(&self, action: GString) -> VarDictionary {
        fight_to_dictionary(self.game.fight_action(action.to_string()))
    }

    /// Hand a finished fight back to the journey
    #[func]
    fn finish_fight(&self) -> VarDictionary {
        step_to_dictionary(self.game.finish_fight())
    }

    #[func]
    fn resolve_fight(&self) -> VarDictionary {
        step_to_dictionary(self.game.resolve_fight_automatically())
    }

    #[func]
    fn journey_in_progress(&self) -> bool {
        self.game.journey_in_progress()
    }

    /// The board here: `location`, `active`, `max_active` and numbered `jobs` with lock reasons
    #[func]
    fn jobs(&self) -> VarDictionary {
        board_to_dictionary(self.game.jobs())
    }

    #[func]
    fn accept_job(&self, number: i64) -> VarDictionary {
        action_to_dictionary(self.game.accept_job(number.max(0) as u32))
    }

    /// The jobs being worked on, numbered from 1
    #[func]
    fn missions(&self) -> VarArray {
        let mut missions = VarArray::new();
        for mission in self.game.missions() {
            missions.push(
                &vdict! {
                    "number": mission.number,
                    "kind": mission.kind,
                    "description": mission.description,
                    "destination": mission.destination,
                    "reward": mission.reward,
                    "jumps_left": mission.jumps_left,
                }
                .to_variant(),
            );
        }
        missions
    }

    #[func]
    fn abandon_job(&self, number: i64) -> VarDictionary {
        action_to_dictionary(self.game.abandon_job(number.max(0) as u32))
    }

    /// The equipment catalogue with `installed` flags, plus credits and power
    #[func]
    fn equipment_shop(&self) -> VarDictionary {
        shop_to_dictionary(self.game.equipment_shop())
    }

    /// What is fitted, with slots, power and what selling it would refund
    #[func]
    fn installed_equipment(&self) -> VarDictionary {
        installed_to_dictionary(self.game.installed_equipment())
    }

    #[func]
    fn buy_equipment(&self, item_id: GString) -> VarDictionary {
        action_to_dictionary(self.game.buy_equipment(item_id.to_string()))
    }

    #[func]
    fn sell_equipment(&self, item_id: GString) -> VarDictionary {
        action_to_dictionary(self.game.sell_equipment(item_id.to_string()))
    }

    #[func]
    fn shipyard(&self) -> VarDictionary {
        shipyard_to_dictionary(self.game.shipyard())
    }

    #[func]
    fn buy_ship(&self, ship_id: GString) -> VarDictionary {
        action_to_dictionary(self.game.buy_ship(ship_id.to_string()))
    }

    /// Every system: `current`, `fuel` and `systems` with position, distance, fuel cost and flags
    #[func]
    fn galaxy(&self) -> VarDictionary {
        galaxy_to_dictionary(self.game.galaxy())
    }

    /// Details of one system; empty when unknown
    #[func]
    fn system_detail(&self, name: GString) -> VarDictionary {
        self.game
            .system_detail(name.to_string())
            .map_or_else(VarDictionary::new, detail_to_dictionary)
    }

    /// Check a jump without making it: `ok`, `message`, `origin`, `destination`, `distance`, `fuel_cost`
    #[func]
    fn plan_jump(&self, destination: GString) -> VarDictionary {
        plan_to_dictionary(self.game.plan_jump(destination.to_string()))
    }

    /// Prices and what is missing: `fuel_missing`, `hull_missing`, `fuel_price`, `repair_price`, `credits`
    #[func]
    fn services(&self) -> VarDictionary {
        let services = self.game.services();
        vdict! {
            "credits": services.credits,
            "fuel_price": services.fuel_price,
            "fuel_missing": services.fuel_missing,
            "repair_price": services.repair_price,
            "hull_missing": services.hull_missing,
        }
    }

    /// Buy `units` of fuel; 0 or less means as much as fits and is affordable
    #[func]
    fn refuel(&self, units: i64) -> VarDictionary {
        action_to_dictionary(self.game.refuel(positive(units)))
    }

    /// Repair `points`; 0 or less means as much as is affordable
    #[func]
    fn repair(&self, points: i64) -> VarDictionary {
        action_to_dictionary(self.game.repair(positive(points)))
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
    fn sell_all(&self, commodity_id: GString) -> VarDictionary {
        trade_to_dictionary(self.game.sell_all(Some(commodity_id.to_string())))
    }

    #[func]
    fn sell_everything(&self) -> VarDictionary {
        trade_to_dictionary(self.game.sell_all(None))
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

fn positive(amount: i64) -> Option<u32> {
    (amount > 0).then_some(amount as u32)
}

fn action_to_dictionary(result: ActionResult) -> VarDictionary {
    vdict! { "ok": result.ok, "message": result.message }
}

fn galaxy_to_dictionary(galaxy: GalaxyData) -> VarDictionary {
    let mut systems = VarArray::new();
    for system in galaxy.systems {
        systems.push(
            &vdict! {
                "name": system.name,
                "x": system.x,
                "y": system.y,
                "distance": system.distance,
                "fuel_cost": system.fuel_cost,
                "reachable": system.reachable,
                "is_current": system.is_current,
                "industrial": system.industrial,
            }
            .to_variant(),
        );
    }
    vdict! { "current": galaxy.current, "fuel": galaxy.fuel, "systems": systems }
}

fn detail_to_dictionary(detail: SystemDetailData) -> VarDictionary {
    vdict! {
        "name": detail.name,
        "economy": detail.economy,
        "government": detail.government,
        "tech_level": detail.tech_level,
        "population": detail.population,
        "description": detail.description,
        "star_class": detail.star_class,
        "star_type": detail.star_type,
        "star_note": detail.star_note,
        "distance": detail.distance,
        "fuel_cost": detail.fuel_cost,
        "reachable": detail.reachable,
        "is_current": detail.is_current,
    }
}

fn plan_to_dictionary(plan: JumpPlanData) -> VarDictionary {
    vdict! {
        "ok": plan.ok,
        "message": plan.message,
        "origin": plan.origin,
        "destination": plan.destination,
        "distance": plan.distance,
        "fuel_cost": plan.fuel_cost,
    }
}

fn shop_to_dictionary(shop: EquipmentShopData) -> VarDictionary {
    let mut items = VarArray::new();
    for item in shop.items {
        items.push(
            &vdict! {
                "id": item.id,
                "name": item.name,
                "category": item.category,
                "quality": item.quality,
                "price": item.price,
                "power": item.power,
                "output": item.output,
                "effect": item.effect,
                "installed": item.installed,
            }
            .to_variant(),
        );
    }
    vdict! {
        "credits": shop.credits,
        "power_used": shop.power_used,
        "power_capacity": shop.power_capacity,
        "items": items,
    }
}

fn installed_to_dictionary(installed: InstalledData) -> VarDictionary {
    let mut items = VarArray::new();
    for item in installed.items {
        items.push(
            &vdict! {
                "id": item.id,
                "name": item.name,
                "category": item.category,
                "slots_used": item.slots_used,
                "slots_capacity": item.slots_capacity,
                "power": item.power,
                "output": item.output,
                "effect": item.effect,
                "refund": item.refund,
            }
            .to_variant(),
        );
    }
    vdict! {
        "power_used": installed.power_used,
        "power_capacity": installed.power_capacity,
        "cargo_capacity": installed.cargo_capacity,
        "items": items,
    }
}

fn shipyard_to_dictionary(yard: ShipyardData) -> VarDictionary {
    let mut ships = VarArray::new();
    for ship in yard.ships {
        ships.push(
            &vdict! {
                "id": ship.id,
                "name": ship.name,
                "cargo": ship.cargo,
                "fuel": ship.fuel,
                "hull": ship.hull,
                "cost": ship.cost,
                "upgrade_cost": ship.upgrade_cost.map_or(-1, i64::from),
                "is_current": ship.is_current,
                "affordable": ship.affordable,
                "is_upgrade": ship.is_upgrade,
            }
            .to_variant(),
        );
    }
    vdict! { "credits": yard.credits, "current": yard.current, "ships": ships }
}

fn board_to_dictionary(board: JobBoardData) -> VarDictionary {
    let mut jobs = VarArray::new();
    for job in board.jobs {
        jobs.push(
            &vdict! {
                "number": job.number,
                "kind": job.kind,
                "description": job.description,
                "destination": job.destination,
                "distance": job.distance,
                "fuel_cost": job.fuel_cost,
                "reward": job.reward,
                "deadline_jumps": job.deadline_jumps,
                "locked": job.locked,
                "lock_reason": job.lock_reason,
            }
            .to_variant(),
        );
    }
    vdict! {
        "location": board.location,
        "active": board.active,
        "max_active": board.max_active,
        "jobs": jobs,
    }
}

fn step_to_dictionary(step: StepData) -> VarDictionary {
    let mut notices = VarArray::new();
    for notice in step.notices {
        notices.push(&vdict! { "kind": notice.kind, "text": notice.text }.to_variant());
    }
    let mut choices = VarArray::new();
    for choice in step.choices {
        choices.push(&choice.to_variant());
    }
    vdict! {
        "kind": step.kind,
        "notices": notices,
        "title": step.title,
        "description": step.description,
        "phase": step.phase,
        "choices": choices,
        "enemy_name": step.enemy_name,
        "enemy_hull": step.enemy_hull,
        "player_hull": step.player_hull,
        "bribe_cost": step.bribe_cost,
        "can_bribe": step.can_bribe,
        "arrived_at": step.arrived_at,
        "planned": step.planned,
        "eventful": step.eventful,
        "message": step.message,
    }
}

fn fighter_to_dictionary(fighter: FighterData) -> VarDictionary {
    vdict! {
        "name": fighter.name,
        "hull": fighter.hull,
        "max_hull": fighter.max_hull,
        "shield": fighter.shield,
        "max_shield": fighter.max_shield,
        "weapons": fighter.weapons,
    }
}

fn fight_to_dictionary(fight: FightData) -> VarDictionary {
    let mut log = VarArray::new();
    for line in fight.log {
        log.push(&line.to_variant());
    }
    vdict! {
        "active": fight.active,
        "player": fighter_to_dictionary(fight.player),
        "enemy": fighter_to_dictionary(fight.enemy),
        "outcome": fight.outcome,
        "log": log,
        "message": fight.message,
    }
}
