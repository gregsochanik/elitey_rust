use elite_core::data::commodities::CommodityType;
use elite_core::data::equipment::EquipmentCategory;
use elite_core::data::ships::ShipType;
use elite_core::data::systems::EconomyType;
use elite_core::game::equipment;
use elite_core::game::refuel::{RefuelError, RefuelOutcome, CREDITS_PER_UNIT};
use elite_core::game::repair::CREDITS_PER_POINT;
use elite_core::game::trade_log::TradeKind;
use elite_core::game::travel::fuel_for_distance;
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
pub struct SystemPointData {
    pub name: String,
    pub x: u32,
    pub y: u32,
    pub distance: f64,
    pub fuel_cost: u32,
    pub reachable: bool,
    pub is_current: bool,
    pub industrial: bool,
}

#[derive(Debug, Clone)]
pub struct GalaxyData {
    pub current: String,
    pub fuel: u32,
    pub systems: Vec<SystemPointData>,
}

#[derive(Debug, Clone)]
pub struct SystemDetailData {
    pub name: String,
    pub economy: String,
    pub government: String,
    pub tech_level: u32,
    pub population: String,
    pub description: String,
    pub distance: f64,
    pub fuel_cost: u32,
    pub reachable: bool,
    pub is_current: bool,
}

#[derive(Debug, Clone)]
pub struct JumpPlanData {
    pub ok: bool,
    pub message: String,
    pub origin: String,
    pub destination: String,
    pub distance: f64,
    pub fuel_cost: u32,
}

#[derive(Debug, Clone)]
pub struct EquipmentItemData {
    pub id: String,
    pub name: String,
    pub category: String,
    pub quality: String,
    pub price: u32,
    pub power: u32,
    pub output: u32,
    pub effect: String,
    pub installed: bool,
}

#[derive(Debug, Clone)]
pub struct EquipmentShopData {
    pub credits: u32,
    pub power_used: u32,
    pub power_capacity: u32,
    pub items: Vec<EquipmentItemData>,
}

#[derive(Debug, Clone)]
pub struct InstalledItemData {
    pub id: String,
    pub name: String,
    pub category: String,
    pub slots_used: u32,
    pub slots_capacity: u32,
    pub power: u32,
    pub output: u32,
    pub effect: String,
    pub refund: u32,
}

#[derive(Debug, Clone)]
pub struct InstalledData {
    pub power_used: u32,
    pub power_capacity: u32,
    pub cargo_capacity: u32,
    pub items: Vec<InstalledItemData>,
}

#[derive(Debug, Clone)]
pub struct ShipData {
    pub id: String,
    pub name: String,
    pub cargo: u32,
    pub fuel: u32,
    pub hull: u32,
    pub cost: u32,
    pub upgrade_cost: Option<u32>,
    pub is_current: bool,
    pub affordable: bool,
    pub is_upgrade: bool,
}

#[derive(Debug, Clone)]
pub struct ShipyardData {
    pub credits: u32,
    pub current: String,
    pub ships: Vec<ShipData>,
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

    /// Every system with its position and what a jump there would cost from here
    pub fn galaxy(&self) -> GalaxyData {
        let state = self.state.lock().expect("game state lock");
        let current = state.commander.location.clone();
        let fuel = state.ship.fuel_current;
        let origin = state.travel_handler.get_system_info(&current);
        let mut systems: Vec<SystemPointData> = state
            .travel_handler
            .get_available_systems()
            .into_iter()
            .map(|system| {
                let distance = origin.map_or(0.0, |origin| origin.distance_to(system));
                let fuel_cost = fuel_for_distance(distance);
                let is_current = system.name == current;
                SystemPointData {
                    name: system.name.clone(),
                    x: u32::from(system.x),
                    y: u32::from(system.y),
                    distance,
                    fuel_cost,
                    reachable: !is_current && fuel_cost <= fuel,
                    is_current,
                    industrial: matches!(
                        system.economy,
                        EconomyType::RichIndustrial
                            | EconomyType::PoorIndustrial
                            | EconomyType::MainlyIndustrial
                            | EconomyType::AverageIndustrial
                    ),
                }
            })
            .collect();
        systems.sort_by(|a, b| a.name.cmp(&b.name));
        GalaxyData {
            current,
            fuel,
            systems,
        }
    }

    pub fn system_detail(&self, name: String) -> Option<SystemDetailData> {
        let state = self.state.lock().expect("game state lock");
        let view = state.system_info(&name)?;
        let from_here = view.from_here.as_ref();
        let is_current = view.name == state.commander.location;
        Some(SystemDetailData {
            economy: view.economy.display_name().to_string(),
            government: view.government.display_name().to_string(),
            tech_level: u32::from(view.tech_level),
            population: view.population,
            description: view.description,
            distance: from_here.map_or(0.0, |jump| jump.distance),
            fuel_cost: from_here.map_or(0, |jump| jump.fuel),
            reachable: !is_current
                && from_here.is_some_and(|jump| jump.fuel <= state.ship.fuel_current),
            is_current,
            name: view.name,
        })
    }

    /// Check a jump and say what it would cost, without making it
    pub fn plan_jump(&self, destination: String) -> JumpPlanData {
        let state = self.state.lock().expect("game state lock");
        match state
            .travel_handler
            .plan_jump(&state.commander, &state.ship, &destination)
        {
            Ok(plan) => JumpPlanData {
                ok: true,
                message: String::new(),
                origin: plan.origin,
                destination: plan.destination,
                distance: plan.distance,
                fuel_cost: plan.fuel_cost,
            },
            Err(error) => JumpPlanData {
                ok: false,
                message: error.to_string(),
                origin: String::new(),
                destination,
                distance: 0.0,
                fuel_cost: 0,
            },
        }
    }

    pub fn equipment_shop(&self) -> EquipmentShopData {
        let state = self.state.lock().expect("game state lock");
        let view = equipment::shop_view(&state.commander, &state.ship);
        EquipmentShopData {
            credits: view.credits,
            power_used: view.power_used,
            power_capacity: view.power_capacity,
            items: view
                .categories
                .into_iter()
                .flat_map(|group| {
                    group.items.into_iter().map(|entry| EquipmentItemData {
                        id: entry.item.id.clone(),
                        name: entry.item.name.clone(),
                        category: entry.item.category.name().to_string(),
                        quality: entry.item.quality.name().to_string(),
                        price: entry.item.price,
                        power: entry.item.power,
                        output: entry.item.output,
                        effect: entry.item.effect.clone(),
                        installed: entry.installed,
                    })
                })
                .collect(),
        }
    }

    pub fn installed_equipment(&self) -> InstalledData {
        let state = self.state.lock().expect("game state lock");
        let loadout = &state.ship.equipment;
        let items = EquipmentCategory::ALL
            .into_iter()
            .flat_map(|category| {
                loadout
                    .installed()
                    .iter()
                    .filter(move |installed| installed.item().category == category)
                    .map(move |installed| {
                        let item = installed.item();
                        InstalledItemData {
                            id: item.id.clone(),
                            name: item.name.clone(),
                            category: category.name().to_string(),
                            slots_used: loadout.slots_used(category) as u32,
                            slots_capacity: loadout.slots_capacity(category) as u32,
                            power: item.power,
                            output: item.output,
                            effect: item.effect.clone(),
                            refund: installed.refund(),
                        }
                    })
            })
            .collect();
        InstalledData {
            power_used: loadout.power_used(),
            power_capacity: loadout.power_capacity(),
            cargo_capacity: state.ship.cargo_capacity(),
            items,
        }
    }

    pub fn buy_equipment(&self, item_id: String) -> ActionResult {
        let mut guard = self.state.lock().expect("game state lock");
        let state = &mut *guard;
        let item = match equipment::find_item(&item_id) {
            Ok(item) => item,
            Err(error) => return failed(error),
        };
        match equipment::buy(&mut state.commander, &mut state.ship, item) {
            Ok(purchase) => ok(match purchase.replaced {
                Some((old, refund)) => format!(
                    "Installed {} for {}cr, replacing {} (refund {}cr)",
                    purchase.name, purchase.price, old, refund
                ),
                None => format!("Installed {} for {}cr", purchase.name, purchase.price),
            }),
            Err(error) => failed(error),
        }
    }

    pub fn sell_equipment(&self, item_id: String) -> ActionResult {
        let mut guard = self.state.lock().expect("game state lock");
        let state = &mut *guard;
        let item = match equipment::find_item(&item_id) {
            Ok(item) => item,
            Err(error) => return failed(error),
        };
        match equipment::sell(&mut state.commander, &mut state.ship, item) {
            Ok(sale) => ok(format!("Sold {} for {}cr", sale.name, sale.refund)),
            Err(error) => failed(error),
        }
    }

    pub fn shipyard(&self) -> ShipyardData {
        let state = self.state.lock().expect("game state lock");
        let list = state
            .upgrade_handler
            .ship_list(&state.commander, &state.ship);
        ShipyardData {
            credits: list.credits,
            current: list.current.display_name().to_string(),
            ships: list
                .ships
                .into_iter()
                .map(|listing| ShipData {
                    id: listing.ship.id().to_string(),
                    name: listing.ship.display_name().to_string(),
                    cargo: listing.ship.cargo_capacity(),
                    fuel: listing.ship.fuel_capacity(),
                    hull: listing.ship.max_hull(),
                    cost: listing.ship.cost(),
                    upgrade_cost: listing.upgrade_cost,
                    is_current: listing.is_current,
                    affordable: listing.affordable,
                    is_upgrade: listing.ship.is_upgrade_from(&list.current),
                })
                .collect(),
        }
    }

    pub fn buy_ship(&self, ship_id: String) -> ActionResult {
        let Some(ship) = ShipType::from_string(&ship_id) else {
            return failed(format!("Unknown ship type: {}", ship_id));
        };
        let mut state = self.state.lock().expect("game state lock");
        match state.buy_ship(ship) {
            Ok(receipt) => {
                let refund = if receipt.equipment_refund > 0 {
                    format!(" Your equipment sold for {}cr.", receipt.equipment_refund)
                } else {
                    String::new()
                };
                ok(format!(
                    "Bought a {} for {}cr.{}",
                    receipt.ship.display_name(),
                    receipt.cost,
                    refund
                ))
            }
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
    fn the_galaxy_marks_the_current_and_reachable_systems() {
        let game = Game::new("Test".to_string());

        let galaxy = game.galaxy();

        assert_eq!(galaxy.current, "LAVE");
        assert!(galaxy.systems.len() > 200);
        let lave = galaxy.systems.iter().find(|s| s.is_current).unwrap();
        assert!(!lave.reachable);
        let reachable: Vec<_> = galaxy.systems.iter().filter(|s| s.reachable).collect();
        assert!(!reachable.is_empty());
        assert!(reachable.iter().all(|s| s.fuel_cost <= galaxy.fuel));
        assert!(galaxy
            .systems
            .iter()
            .any(|s| s.name == "DISO" && s.reachable));
    }

    #[test]
    fn a_system_detail_says_what_the_jump_costs() {
        let game = Game::new("Test".to_string());

        let diso = game.system_detail("DISO".to_string()).unwrap();

        assert_eq!(diso.fuel_cost, 3);
        assert!(diso.reachable && !diso.is_current);
        assert_eq!(diso.economy, "Average Agricultural");
        assert!(game.system_detail("NOWHERE".to_string()).is_none());
        assert!(game.system_detail("LAVE".to_string()).unwrap().is_current);
    }

    #[test]
    fn planning_a_jump_reports_cost_or_the_reason_it_cannot_be_made() {
        let game = Game::new("Test".to_string());

        let plan = game.plan_jump("diso".to_string());
        assert!(plan.ok);
        assert_eq!(
            (plan.origin.as_str(), plan.destination.as_str()),
            ("LAVE", "DISO")
        );
        assert_eq!(plan.fuel_cost, 3);

        assert!(!game.plan_jump("LAVE".to_string()).ok);
        assert!(!game.plan_jump("nowhere".to_string()).ok);
        let far = game
            .galaxy()
            .systems
            .into_iter()
            .find(|s| !s.reachable && !s.is_current)
            .unwrap();
        assert!(!game.plan_jump(far.name).ok);
    }

    #[test]
    fn the_shop_lists_equipment_and_marks_what_is_installed() {
        let game = Game::new("Test".to_string());

        let shop = game.equipment_shop();

        assert_eq!((shop.power_used, shop.power_capacity), (9, 20));
        assert!(shop
            .items
            .iter()
            .any(|i| i.id == "basic-reactor" && i.installed));
        assert!(shop
            .items
            .iter()
            .any(|i| i.id == "elite-reactor" && !i.installed));
        let installed = game.installed_equipment();
        assert_eq!(installed.items.len(), 7);
        assert_eq!(installed.cargo_capacity, 20);
    }

    #[test]
    fn buying_equipment_spends_credits_and_selling_refunds_half() {
        let game = Game::new("Test".to_string());

        let bought = game.buy_equipment("improved-reactor".to_string());
        assert!(bought.ok, "{}", bought.message);
        assert_eq!(game.installed_equipment().power_capacity, 28);
        assert_eq!(game.status().credits, 100 - 60);
        let refund = game
            .installed_equipment()
            .items
            .iter()
            .find(|i| i.id == "improved-reactor")
            .unwrap()
            .refund;
        assert_eq!(refund, 30);

        assert!(!game.buy_equipment("elite-reactor".to_string()).ok);
        assert!(!game.buy_equipment("nothing".to_string()).ok);
        assert!(!game
            .sell_equipment("basic-shield".to_string())
            .message
            .is_empty());
        assert!(!game.sell_equipment("improved-reactor".to_string()).ok);
        assert!(game.buy_equipment("advanced-scanner".to_string()).ok);
        let sold = game.sell_equipment("advanced-scanner".to_string());
        assert_eq!(sold.message, "Sold Advanced Scanner for 20cr");
    }

    #[test]
    fn the_shipyard_lists_ships_and_refuses_what_cannot_be_afforded() {
        let game = Game::new("Test".to_string());

        let yard = game.shipyard();

        assert_eq!(yard.current, "Cobra Mk III");
        assert!(yard.ships.iter().any(|s| s.is_current));
        assert!(yard.ships.iter().any(|s| s.is_upgrade && !s.affordable));
        let result = game.buy_ship("python".to_string());
        assert!(!result.ok);
        assert!(result.message.contains("Insufficient credits"));
        assert!(!game.buy_ship("nonsense".to_string()).ok);
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
