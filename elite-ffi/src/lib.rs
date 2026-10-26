use elite_core::data::commodities::CommodityType;
use elite_core::data::equipment::EquipmentCategory;
use elite_core::data::save_load::SaveLoadManager;
use elite_core::data::ships::ShipType;
use elite_core::data::systems::EconomyType;
use elite_core::game::combat::{Action, Combat, Outcome};
use elite_core::game::equipment;
use elite_core::game::events::EventPhase;
use elite_core::game::jobs::{ABANDON_REPUTATION, MAX_ACTIVE_JOBS};
use elite_core::game::journey::{auto_fight, Input, Journey, Notice, Step};
use elite_core::game::npc::NpcAction;
use elite_core::game::refuel::{RefuelError, RefuelOutcome, CREDITS_PER_UNIT};
use elite_core::game::repair::CREDITS_PER_POINT;
use elite_core::game::smuggling::InspectionChoice;
use elite_core::game::trade_log::TradeKind;
use elite_core::game::travel::fuel_for_distance;
use elite_core::game::GameState;
use elite_core::game::SaveLoadHandler;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
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
    /// Why this system does not sell the commodity, or empty when it does
    pub restriction: String,
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
    pub explored_percent: u32,
    pub explorer_rating: String,
    pub systems_visited: u32,
    pub systems_total: u32,
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
    pub visited: bool,
    pub frontier: bool,
}

#[derive(Debug, Clone)]
pub struct HopData {
    pub from: String,
    pub to: String,
    pub distance: f64,
    pub fuel: u32,
}

#[derive(Debug, Clone)]
pub struct RouteData {
    pub ok: bool,
    /// Why there is no route, when there is none
    pub message: String,
    pub hops: Vec<HopData>,
    pub total_fuel: u32,
    pub total_distance: f64,
    pub refuel_at: Vec<String>,
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
    pub star_class: String,
    pub star_type: String,
    pub star_note: String,
    pub visited: bool,
    pub frontier: bool,
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
    pub min_tech: u32,
    /// The system's tech level is below the item's
    pub locked: bool,
}

#[derive(Debug, Clone)]
pub struct EquipmentShopData {
    pub credits: u32,
    pub power_used: u32,
    pub power_capacity: u32,
    pub tech_level: u32,
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
    pub min_tech: u32,
    pub locked: bool,
}

#[derive(Debug, Clone)]
pub struct ShipyardData {
    pub tech_level: u32,
    pub credits: u32,
    pub current: String,
    pub ships: Vec<ShipData>,
}

#[derive(Debug, Clone)]
pub struct JobData {
    pub number: u32,
    pub kind: String,
    pub description: String,
    pub destination: String,
    pub distance: f64,
    pub fuel_cost: u32,
    pub reward: u32,
    pub deadline_jumps: u32,
    pub locked: bool,
    pub lock_reason: String,
}

#[derive(Debug, Clone)]
pub struct JobBoardData {
    pub location: String,
    pub active: u32,
    pub max_active: u32,
    pub jobs: Vec<JobData>,
}

#[derive(Debug, Clone)]
pub struct MissionData {
    pub number: u32,
    pub kind: String,
    pub description: String,
    pub destination: String,
    pub reward: u32,
    pub jumps_left: u32,
}

#[derive(Debug, Clone)]
pub struct NoticeData {
    /// `blank`, `text`, `info`, `success` or `error`
    pub kind: String,
    pub text: String,
}

/// What a journey needs next, with the lines to show first
#[derive(Debug, Clone, Default)]
pub struct StepData {
    /// `event`, `fight`, `inspection`, `arrived` or `error`
    pub kind: String,
    pub notices: Vec<NoticeData>,
    pub title: String,
    pub description: String,
    /// `jump` or `approach` for events
    pub phase: String,
    pub choices: Vec<String>,
    pub enemy_name: String,
    pub enemy_hull: u32,
    pub player_hull: u32,
    pub bribe_cost: u32,
    pub can_bribe: bool,
    pub arrived_at: String,
    pub planned: String,
    pub eventful: bool,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct SaveSlotData {
    pub slot: String,
    pub commander: String,
    pub location: String,
    pub credits: u32,
    pub ship: String,
    pub saved_at: String,
    pub is_autosave: bool,
}

#[derive(Debug, Clone, Default)]
pub struct FighterData {
    pub name: String,
    pub hull: u32,
    pub max_hull: u32,
    pub shield: u32,
    pub max_shield: u32,
    pub weapons: u32,
}

/// The fight in progress: both ships, how it stands and everything said so far
#[derive(Debug, Clone, Default)]
pub struct FightData {
    pub active: bool,
    pub player: FighterData,
    pub enemy: FighterData,
    /// `ongoing`, `victory`, `defeat`, `escaped`, `surrendered`, `enemy_fled` or `enemy_surrendered`
    pub outcome: String,
    pub log: Vec<String>,
    /// What the enemy did in the latest round: `attack`, `flee`, `surrender`, or empty
    pub enemy_action: String,
    /// Why the last action was refused, if it was
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct TradeResult {
    pub message: String,
    pub credits_after: u32,
}

/// A jump in progress and the step it is waiting on
struct Voyage {
    journey: Journey,
    step: Step,
}

pub struct Game {
    state: Mutex<GameState>,
    rng: Mutex<StdRng>,
    voyage: Mutex<Option<Voyage>>,
}

impl Game {
    pub fn new(commander_name: String) -> Arc<Self> {
        Self::with_rng(commander_name, StdRng::from_entropy())
    }

    /// A game whose jobs, events and fights are repeatable
    pub fn with_seed(commander_name: String, seed: u64) -> Arc<Self> {
        Self::with_rng(commander_name, StdRng::seed_from_u64(seed))
    }

    /// A game that keeps its saves in `directory`, for tests
    pub fn with_saves_in(
        commander_name: String,
        seed: u64,
        directory: std::path::PathBuf,
    ) -> Arc<Self> {
        let game = Self::with_seed(commander_name, seed);
        game.state
            .lock()
            .expect("game state lock")
            .save_load_handler =
            SaveLoadHandler::with_directory(directory).expect("save directory");
        game
    }

    fn with_rng(commander_name: String, rng: StdRng) -> Arc<Self> {
        Arc::new(Self {
            state: Mutex::new(GameState::new(commander_name)),
            rng: Mutex::new(rng),
            voyage: Mutex::new(None),
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
            explored_percent: view.exploration.percent,
            explorer_rating: view.exploration.rating.name().to_string(),
            systems_visited: view.exploration.visited,
            systems_total: view.exploration.total,
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
        let frontier = elite_core::game::frontier::frontier_systems(
            &state.travel_handler.get_available_systems(),
        );
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
                    visited: state.commander.has_visited(&system.name),
                    frontier: frontier.contains(&system.name),
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

    /// The cheapest-fuel route from here to `destination`
    pub fn plan_route(&self, destination: String) -> RouteData {
        let state = self.state.lock().expect("game state lock");
        match state.plan_route(&destination) {
            Ok(route) => RouteData {
                ok: true,
                message: String::new(),
                hops: route
                    .hops
                    .into_iter()
                    .map(|hop| HopData {
                        from: hop.from,
                        to: hop.to,
                        distance: hop.distance,
                        fuel: hop.fuel,
                    })
                    .collect(),
                total_fuel: route.total_fuel,
                total_distance: route.total_distance,
                refuel_at: route.refuel_at,
            },
            Err(error) => RouteData {
                ok: false,
                message: error.to_string(),
                hops: Vec::new(),
                total_fuel: 0,
                total_distance: 0.0,
                refuel_at: Vec::new(),
            },
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
            star_class: view
                .star
                .as_ref()
                .map_or_else(String::new, |s| s.code.clone()),
            star_type: view
                .star
                .as_ref()
                .map_or_else(String::new, |s| s.type_name.clone()),
            star_note: view
                .star
                .as_ref()
                .map_or_else(String::new, |s| s.note.to_string()),
            visited: view.visited,
            frontier: view.frontier,
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
        let tech_level = state.local_tech_level();
        let view = equipment::shop_view(&state.commander, &state.ship, tech_level);
        EquipmentShopData {
            tech_level: u32::from(tech_level),
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
                        min_tech: u32::from(entry.item.min_tech),
                        locked: entry.locked,
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
        if let Err(error) = equipment::check_tech(item, state.local_tech_level()) {
            return failed(error);
        }
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
        let list = state.upgrade_handler.ship_list(
            &state.commander,
            &state.ship,
            state.local_tech_level(),
        );
        ShipyardData {
            tech_level: u32::from(state.local_tech_level()),
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
                    min_tech: u32::from(listing.ship.min_tech()),
                    locked: listing.locked,
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

    /// The jobs on offer here, rolled on first sight; locked ones say why
    pub fn jobs(&self) -> JobBoardData {
        let mut state = self.state.lock().expect("game state lock");
        let listings = state
            .jobs_here(&mut *self.rng.lock().expect("rng lock"))
            .unwrap_or_default();
        JobBoardData {
            location: state.commander.location.clone(),
            active: state.active_jobs.len() as u32,
            max_active: MAX_ACTIVE_JOBS as u32,
            jobs: listings
                .into_iter()
                .enumerate()
                .map(|(index, listing)| JobData {
                    number: index as u32 + 1,
                    kind: job_kind(&listing.job),
                    description: listing.job.describe(),
                    destination: listing.job.destination.clone(),
                    distance: listing.job.distance,
                    fuel_cost: listing.job.fuel_cost,
                    reward: listing.job.reward,
                    deadline_jumps: listing.job.deadline_jumps,
                    locked: listing.lock.is_some(),
                    lock_reason: listing.lock.map(|lock| lock.describe()).unwrap_or_default(),
                })
                .collect(),
        }
    }

    pub fn accept_job(&self, number: u32) -> ActionResult {
        let mut state = self.state.lock().expect("game state lock");
        match state.accept_job(number as usize) {
            Ok(job) => ok(format!(
                "Accepted: {} for {}cr within {} jumps.",
                job.describe(),
                job.reward,
                job.deadline_jumps
            )),
            Err(message) => failed(message),
        }
    }

    pub fn missions(&self) -> Vec<MissionData> {
        let state = self.state.lock().expect("game state lock");
        state
            .active_jobs
            .iter()
            .enumerate()
            .map(|(index, entry)| MissionData {
                number: index as u32 + 1,
                kind: job_kind(&entry.job),
                description: entry.job.describe(),
                destination: entry.job.destination.clone(),
                reward: entry.job.reward,
                jumps_left: entry.jumps_left,
            })
            .collect()
    }

    pub fn abandon_job(&self, number: u32) -> ActionResult {
        let mut state = self.state.lock().expect("game state lock");
        let standing = state.commander.standing();
        match state.abandon_job(number as usize) {
            Ok(job) => {
                let mut message = format!(
                    "Abandoned: {}. Reputation {:+}.",
                    job.describe(),
                    ABANDON_REPUTATION
                );
                if state.commander.standing() != standing {
                    message.push_str(&format!(
                        " Your standing is now {}.",
                        state.commander.standing().name()
                    ));
                }
                ok(message)
            }
            Err(message) => failed(message),
        }
    }

    /// Confirm nothing: plan and start a jump, returning its first step
    pub fn begin_jump(&self, destination: String) -> StepData {
        let mut state = self.state.lock().expect("game state lock");
        let mut voyage = self.voyage.lock().expect("voyage lock");
        if voyage.is_some() {
            return step_error("A jump is already in progress");
        }
        let plan = match state.plan_jump(&destination) {
            Ok(plan) => plan,
            Err(error) => return step_error(error),
        };
        let mut rng = self.rng.lock().expect("rng lock");
        let (mut journey, step) = state.start_jump(plan, &mut *rng);
        Self::settle(&mut voyage, journey.take_notices(), journey, step)
    }

    /// Answer an event with the 0-based choice
    pub fn answer_event(&self, choice: u32) -> StepData {
        self.answer(Input::Choice(choice as usize))
    }

    /// Answer an inspection: `bribe`, `run` or `hand_over`
    pub fn answer_inspection(&self, choice: String) -> StepData {
        let choice = match choice.as_str() {
            "bribe" => InspectionChoice::Bribe,
            "run" => InspectionChoice::Run,
            "hand_over" => InspectionChoice::HandOver,
            other => return step_error(format!("Unknown inspection choice: {}", other)),
        };
        self.answer(Input::Inspection(choice))
    }

    /// Play the current fight out automatically
    pub fn resolve_fight_automatically(&self) -> StepData {
        let mut voyage = self.voyage.lock().expect("voyage lock");
        let Some(Voyage {
            step: Step::Fight(combat),
            ..
        }) = voyage.as_mut()
        else {
            return step_error("No fight is waiting");
        };
        let mut combat = combat.clone();
        auto_fight(&mut combat, &mut *self.rng.lock().expect("rng lock"));
        drop(voyage);
        self.answer(Input::FightEnded(combat))
    }

    /// The fight the journey is waiting on
    pub fn fight(&self) -> FightData {
        let voyage = self.voyage.lock().expect("voyage lock");
        match voyage.as_ref().map(|voyage| &voyage.step) {
            Some(Step::Fight(combat)) => fight_data(combat, String::new()),
            _ => FightData::default(),
        }
    }

    /// Play one round: `attack`, `flee` or `surrender`
    pub fn fight_action(&self, action: String) -> FightData {
        let action = match action.as_str() {
            "attack" => Action::Attack,
            "flee" => Action::Flee,
            "surrender" => Action::Surrender,
            _ => return self.fight_with_message(format!("Unknown action: {}", action)),
        };
        let mut voyage = self.voyage.lock().expect("voyage lock");
        let Some(Voyage {
            step: Step::Fight(combat),
            ..
        }) = voyage.as_mut()
        else {
            return FightData::default();
        };
        let mut rng = self.rng.lock().expect("rng lock");
        let mut roll = || rng.gen::<f64>();
        let message = match combat.resolve_round(action, &mut roll) {
            Ok(_) => String::new(),
            Err(error) => error.to_string(),
        };
        fight_data(combat, message)
    }

    /// Hand the finished fight back to the journey and carry on
    pub fn finish_fight(&self) -> StepData {
        let combat = {
            let voyage = self.voyage.lock().expect("voyage lock");
            match voyage.as_ref().map(|voyage| &voyage.step) {
                Some(Step::Fight(combat)) if combat.outcome() != Outcome::Ongoing => combat.clone(),
                Some(Step::Fight(_)) => return step_error("The fight is not over"),
                _ => return step_error("No fight is waiting"),
            }
        };
        self.answer(Input::FightEnded(combat))
    }

    fn fight_with_message(&self, message: String) -> FightData {
        FightData {
            message,
            ..self.fight()
        }
    }

    pub fn journey_in_progress(&self) -> bool {
        self.voyage.lock().expect("voyage lock").is_some()
    }

    fn answer(&self, input: Input) -> StepData {
        let mut state = self.state.lock().expect("game state lock");
        let mut voyage = self.voyage.lock().expect("voyage lock");
        let Some(current) = voyage.as_mut() else {
            return step_error("No jump is in progress");
        };
        let mut rng = self.rng.lock().expect("rng lock");
        match state.advance_journey(&mut current.journey, input, &mut *rng) {
            Ok(step) => {
                let notices = current.journey.take_notices();
                let data = step_data(&step, notices);
                if data.kind == "arrived" {
                    *voyage = None;
                } else {
                    current.step = step;
                }
                data
            }
            Err(error) => step_error(error),
        }
    }

    fn settle(
        voyage: &mut Option<Voyage>,
        notices: Vec<Notice>,
        journey: Journey,
        step: Step,
    ) -> StepData {
        let data = step_data(&step, notices);
        if data.kind != "arrived" {
            *voyage = Some(Voyage { journey, step });
        }
        data
    }

    /// Every save, newest first, with the slot name it loads from
    pub fn saves(&self) -> Vec<SaveSlotData> {
        let state = self.state.lock().expect("game state lock");
        state
            .save_load_handler
            .list_slots()
            .unwrap_or_default()
            .into_iter()
            .map(|(slot, metadata)| SaveSlotData {
                is_autosave: slot.starts_with("autosave"),
                saved_at: SaveLoadManager::format_timestamp(metadata.timestamp),
                slot,
                commander: metadata.commander_name,
                location: metadata.location,
                credits: metadata.credits,
                ship: metadata.ship_type,
            })
            .collect()
    }

    /// Whether this commander already has an autosave that a new game would replace
    pub fn autosave_exists(&self, commander_name: String) -> bool {
        let state = self.state.lock().expect("game state lock");
        state
            .save_load_handler
            .preview(&SaveLoadHandler::autosave_slot(commander_name.trim()))
            .is_ok()
    }

    /// Start over as a new commander; nothing is saved until you save or quit
    pub fn new_game(&self, commander_name: String) -> ActionResult {
        let name = commander_name.trim();
        if name.is_empty() {
            return failed("Name cannot be empty");
        }
        self.replace_state(|_| GameState::new(name.to_string()));
        ok(format!("Commander {}, your journey begins!", name))
    }

    /// Continue a commander's autosave
    pub fn continue_game(&self, commander_name: String) -> ActionResult {
        let found = {
            let state = self.state.lock().expect("game state lock");
            state.save_load_handler.find_autosave(commander_name.trim())
        };
        match found {
            Ok(Some(loaded)) => self.finish_load(loaded),
            Ok(None) => failed(format!("No save found for {}", commander_name.trim())),
            Err(error) => failed(error),
        }
    }

    pub fn load_slot(&self, slot: String) -> ActionResult {
        let loaded = {
            let state = self.state.lock().expect("game state lock");
            state.save_load_handler.load(&slot)
        };
        match loaded {
            Ok(loaded) => self.finish_load(loaded),
            Err(error) => failed(error),
        }
    }

    pub fn save_slot(&self, slot: String) -> ActionResult {
        let state = self.state.lock().expect("game state lock");
        match state.save_load_handler.save(&state, &slot) {
            Ok(()) => ok(format!("Game saved to slot '{}'.", slot)),
            Err(error) => failed(error),
        }
    }

    pub fn delete_slot(&self, slot: String) -> ActionResult {
        let state = self.state.lock().expect("game state lock");
        match state.save_load_handler.delete(&slot) {
            Ok(()) => ok(format!("Deleted save '{}'.", slot)),
            Err(error) => failed(error),
        }
    }

    /// Write the commander's autosave
    pub fn autosave(&self) -> ActionResult {
        let state = self.state.lock().expect("game state lock");
        match state.save_load_handler.auto_save(&state) {
            Ok(()) => ok(format!(
                "Commander {}'s progress has been saved.",
                state.commander.name()
            )),
            Err(error) => failed(error),
        }
    }

    fn finish_load(&self, loaded: GameState) -> ActionResult {
        let name = loaded.commander.name().to_string();
        self.replace_state(|_| loaded);
        ok(format!(
            "Game loaded. Commander {}, your journey continues!",
            name
        ))
    }

    fn replace_state(&self, next: impl FnOnce(&GameState) -> GameState) {
        let mut state = self.state.lock().expect("game state lock");
        let handler = state.save_load_handler.clone();
        let mut replacement = next(&state);
        replacement.save_load_handler = handler;
        *state = replacement;
        *self.voyage.lock().expect("voyage lock") = None;
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
                    restriction: row.restriction.unwrap_or_default(),
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

    pub fn sell_all(&self, commodity_id: Option<String>) -> Result<TradeResult, GameError> {
        let commodity = commodity_id.as_deref().map(parse_commodity).transpose()?;
        let mut state = self.state.lock().expect("game state lock");
        let receipts = state.sell_all(commodity).map_err(GameError::refused)?;
        let sold: Vec<String> = receipts
            .iter()
            .map(|receipt| format!("{} tons of {}", receipt.quantity, receipt.commodity.name()))
            .collect();
        let total: u32 = receipts.iter().map(|receipt| receipt.total).sum();
        Ok(TradeResult {
            message: format!("Sold {} for {}cr", sold.join(", "), total),
            credits_after: receipts.last().map_or(0, |receipt| receipt.credits_after),
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

fn fighter_data(fighter: &elite_core::game::combat::Combatant) -> FighterData {
    FighterData {
        name: fighter.name.clone(),
        hull: fighter.hull,
        max_hull: fighter.max_hull,
        shield: fighter.shield,
        max_shield: fighter.max_shield,
        weapons: fighter.weapons.len() as u32,
    }
}

fn fight_data(combat: &Combat, message: String) -> FightData {
    FightData {
        active: true,
        player: fighter_data(&combat.player),
        enemy: fighter_data(&combat.enemy),
        outcome: match combat.outcome() {
            Outcome::Ongoing => "ongoing",
            Outcome::Victory => "victory",
            Outcome::Defeat => "defeat",
            Outcome::Escaped => "escaped",
            Outcome::Surrendered => "surrendered",
            Outcome::EnemyFled => "enemy_fled",
            Outcome::EnemySurrendered => "enemy_surrendered",
        }
        .to_string(),
        log: combat.log.clone(),
        enemy_action: match combat.last_npc_action {
            Some(NpcAction::Attack) => "attack",
            Some(NpcAction::Flee) => "flee",
            Some(NpcAction::Surrender) => "surrender",
            None => "",
        }
        .to_string(),
        message,
    }
}

fn step_error(message: impl ToString) -> StepData {
    StepData {
        kind: "error".to_string(),
        message: message.to_string(),
        ..StepData::default()
    }
}

fn step_data(step: &Step, notices: Vec<Notice>) -> StepData {
    let notices = notices
        .into_iter()
        .map(|notice| match notice {
            Notice::Blank => NoticeData {
                kind: "blank".to_string(),
                text: String::new(),
            },
            Notice::Text(text) => NoticeData {
                kind: "text".to_string(),
                text,
            },
            Notice::Info(text) => NoticeData {
                kind: "info".to_string(),
                text,
            },
            Notice::Success(text) => NoticeData {
                kind: "success".to_string(),
                text,
            },
            Notice::Error(text) => NoticeData {
                kind: "error".to_string(),
                text,
            },
        })
        .collect();
    let base = StepData {
        notices,
        ..StepData::default()
    };
    match step {
        Step::Event(event) => StepData {
            kind: "event".to_string(),
            title: event.title.clone(),
            description: event.description.clone(),
            phase: match event.phase {
                EventPhase::Jump => "jump".to_string(),
                EventPhase::Approach => "approach".to_string(),
            },
            choices: event
                .choices
                .iter()
                .map(|choice| choice.description.clone())
                .collect(),
            ..base
        },
        Step::Fight(combat) => StepData {
            kind: "fight".to_string(),
            enemy_name: combat.enemy.name.clone(),
            enemy_hull: combat.enemy.hull,
            player_hull: combat.player.hull,
            ..base
        },
        Step::Inspection {
            bribe_cost,
            can_bribe,
        } => StepData {
            kind: "inspection".to_string(),
            bribe_cost: *bribe_cost,
            can_bribe: *can_bribe,
            ..base
        },
        Step::Arrived {
            arrived_at,
            planned,
            eventful,
        } => StepData {
            kind: "arrived".to_string(),
            arrived_at: arrived_at.clone(),
            planned: planned.clone(),
            eventful: *eventful,
            ..base
        },
    }
}

fn job_kind(job: &elite_core::game::jobs::Job) -> String {
    format!("{:?}", job.kind).to_lowercase()
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

        assert_eq!(market.system_name, "Sol");
        assert_eq!(market.rows.len(), CommodityType::all().len());
        assert!(market.rows.iter().all(|row| row.buy_price > 0));
    }

    #[test]
    fn the_status_describes_the_commander_and_ship() {
        let game = Game::new("Test".to_string());

        let status = game.status();

        assert_eq!(status.commander_name, "Test");
        assert_eq!(status.location, "Sol");
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

        assert_eq!(galaxy.current, "Sol");
        assert!(galaxy.systems.len() > 200);
        let sol = galaxy.systems.iter().find(|s| s.is_current).unwrap();
        assert!(!sol.reachable);
        let reachable: Vec<_> = galaxy.systems.iter().filter(|s| s.reachable).collect();
        assert!(!reachable.is_empty());
        assert!(reachable.iter().all(|s| s.fuel_cost <= galaxy.fuel));
        assert!(galaxy
            .systems
            .iter()
            .any(|s| s.name == "Proxima Centauri" && s.reachable));
    }

    #[test]
    fn a_system_detail_says_what_the_jump_costs() {
        let game = Game::new("Test".to_string());

        let proxima = game.system_detail("Proxima Centauri".to_string()).unwrap();

        assert_eq!(proxima.fuel_cost, 3);
        assert!(proxima.reachable && !proxima.is_current);
        assert_eq!(proxima.economy, "Mainly Industrial");
        assert!(game.system_detail("NOWHERE".to_string()).is_none());
        assert!(game.system_detail("Sol".to_string()).unwrap().is_current);
    }

    #[test]
    fn planning_a_jump_reports_cost_or_the_reason_it_cannot_be_made() {
        let game = Game::new("Test".to_string());

        let plan = game.plan_jump("proxima centauri".to_string());
        assert!(plan.ok);
        assert_eq!(
            (plan.origin.as_str(), plan.destination.as_str()),
            ("Sol", "Proxima Centauri")
        );
        assert_eq!(plan.fuel_cost, 3);

        assert!(!game.plan_jump("Sol".to_string()).ok);
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

        let locked = game.buy_equipment("elite-reactor".to_string());
        assert!(!locked.ok && locked.message.contains("needs tech level 11"));
        assert!(game
            .equipment_shop()
            .items
            .iter()
            .any(|item| item.id == "elite-reactor" && item.locked));
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
        let locked = yard.ships.iter().find(|s| s.id == "python").unwrap();
        assert!(locked.locked && locked.min_tech == 7);
        let result = game.buy_ship("python".to_string());
        assert!(!result.ok);
        assert!(result.message.contains("needs tech level 7"));
        let result = game.buy_ship("cobra-mk-iv".to_string());
        assert!(!result.ok);
        assert!(result.message.contains("Insufficient credits"));
        assert!(!game.buy_ship("nonsense".to_string()).ok);
    }

    #[test]
    fn the_job_board_numbers_jobs_and_says_why_some_are_locked() {
        let game = Game::new("Test".to_string());

        let board = game.jobs();

        assert_eq!(board.location, "Sol");
        assert!((3..=5).contains(&board.jobs.len()));
        assert_eq!(board.jobs[0].number, 1);
        assert!(board
            .jobs
            .iter()
            .all(|j| j.locked != j.lock_reason.is_empty()));
        assert_eq!(game.jobs().jobs.len(), board.jobs.len());
    }

    #[test]
    fn accepting_and_abandoning_jobs_moves_them_between_the_board_and_missions() {
        let (game, board) = (0..40)
            .map(|seed| Game::with_seed("Test".to_string(), seed))
            .map(|game| {
                let board = game.jobs();
                (game, board)
            })
            .find(|(_, board)| board.jobs.iter().any(|j| !j.locked))
            .unwrap();
        let open = board.jobs.iter().find(|j| !j.locked).unwrap();

        let accepted = game.accept_job(open.number);
        assert!(accepted.ok, "{}", accepted.message);
        let missions = game.missions();
        assert_eq!(missions.len(), 1);
        assert_eq!(missions[0].reward, open.reward);
        assert_eq!(game.jobs().jobs.len(), board.jobs.len() - 1);
        assert_eq!(game.jobs().active, 1);

        let abandoned = game.abandon_job(1);
        assert!(abandoned.ok);
        assert!(abandoned.message.contains("Reputation -3"));
        assert!(game.missions().is_empty());
        assert!(!game.abandon_job(1).ok);
    }

    #[test]
    fn a_locked_job_cannot_be_accepted() {
        let game = Game::with_seed("Test".to_string(), 5);
        for _ in 0..40 {
            let board = game.jobs();
            if let Some(locked) = board.jobs.iter().find(|j| j.locked) {
                let result = game.accept_job(locked.number);
                assert!(!result.ok);
                assert!(result.message.starts_with("Locked:"));
                return;
            }
            game.state.lock().unwrap().job_board.clear();
        }
        panic!("no locked job appeared");
    }

    fn play_out(game: &Game, mut step: StepData) -> (StepData, Vec<String>) {
        let mut kinds = Vec::new();
        for _ in 0..50 {
            kinds.push(step.kind.clone());
            step = match step.kind.as_str() {
                "event" => game.answer_event(0),
                "inspection" => game.answer_inspection("hand_over".to_string()),
                "fight" => game.resolve_fight_automatically(),
                _ => return (step, kinds),
            };
        }
        panic!("the journey never ended: {:?}", kinds);
    }

    #[test]
    fn a_jump_runs_to_arrival_and_spends_fuel() {
        let game = Game::with_seed("Test".to_string(), 1);

        let first = game.begin_jump("Proxima Centauri".to_string());
        assert!(game.journey_in_progress() || first.kind == "arrived");
        let (last, _) = play_out(&game, first);

        assert_eq!(last.kind, "arrived", "{}", last.message);
        assert!(!game.journey_in_progress());
        assert_eq!(last.planned, "Proxima Centauri");
        let status = game.status();
        assert!(status.fuel_current < 7);
        assert_eq!(status.location, last.arrived_at);
    }

    #[test]
    fn events_and_fights_come_up_and_every_journey_still_ends() {
        let mut seen = std::collections::HashSet::new();
        for seed in 0..60 {
            let game = Game::with_seed("Test".to_string(), seed);
            let first = game.begin_jump("Proxima Centauri".to_string());
            let (last, kinds) = play_out(&game, first);
            assert_eq!(last.kind, "arrived", "seed {}", seed);
            seen.extend(kinds);
        }
        assert!(seen.contains("event"), "{:?}", seen);
    }

    #[test]
    fn events_describe_their_choices() {
        for seed in 0..60 {
            let game = Game::with_seed("Test".to_string(), seed);
            let step = game.begin_jump("Proxima Centauri".to_string());
            if step.kind == "event" {
                assert!(!step.title.is_empty());
                assert!(!step.choices.is_empty());
                assert!(step.phase == "jump" || step.phase == "approach");
                return;
            }
            play_out(&game, step);
        }
        panic!("no event appeared");
    }

    #[test]
    fn a_journey_refuses_bad_requests() {
        let game = Game::with_seed("Test".to_string(), 1);

        assert_eq!(game.begin_jump("Sol".to_string()).kind, "error");
        assert_eq!(game.begin_jump("nowhere".to_string()).kind, "error");
        assert_eq!(game.answer_event(0).kind, "error");
        assert_eq!(game.answer_inspection("bribe".to_string()).kind, "error");
        assert_eq!(game.resolve_fight_automatically().kind, "error");
        assert!(!game.journey_in_progress());
    }

    #[test]
    fn a_second_jump_cannot_start_mid_journey() {
        for seed in 0..60 {
            let game = Game::with_seed("Test".to_string(), seed);
            let step = game.begin_jump("Proxima Centauri".to_string());
            if step.kind != "arrived" {
                let again = game.begin_jump("Alpha Centauri".to_string());
                assert_eq!(again.kind, "error");
                assert!(again.message.contains("already in progress"));
                assert!(game
                    .answer_inspection("nonsense".to_string())
                    .message
                    .contains("Unknown"));
                return;
            }
        }
        panic!("every journey was quiet");
    }

    #[test]
    fn the_same_seed_gives_the_same_journey() {
        let run = |seed| {
            let game = Game::with_seed("Test".to_string(), seed);
            let first = game.begin_jump("Proxima Centauri".to_string());
            let (last, kinds) = play_out(&game, first);
            (kinds, last.arrived_at, game.status().credits)
        };
        assert_eq!(run(7), run(7));
    }

    fn temp_saves(label: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("elite_ffi_{}_{}", label, std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn a_game_saves_and_loads_by_slot() {
        let game = Game::with_saves_in("Saver".to_string(), 1, temp_saves("slots"));
        game.buy("food".to_string(), 3).unwrap();
        let credits = game.status().credits;

        assert!(game.save_slot("first".to_string()).ok);
        assert!(!game.save_slot("bad name!".to_string()).ok);
        let saves = game.saves();
        assert_eq!(saves.len(), 1);
        assert_eq!(saves[0].slot, "first");
        assert_eq!(saves[0].commander, "Saver");
        assert!(!saves[0].is_autosave);

        game.new_game("Other".to_string());
        assert_eq!(game.status().commander_name, "Other");
        assert_eq!(game.status().credits, 100);

        assert!(game.load_slot("first".to_string()).ok);
        assert_eq!(game.status().commander_name, "Saver");
        assert_eq!(game.status().credits, credits);
        assert_eq!(game.cargo().weight, 3);
        assert!(!game.load_slot("missing".to_string()).ok);
    }

    #[test]
    fn the_autosave_is_found_by_commander_name_and_deleted_by_slot() {
        let game = Game::with_saves_in("Greg".to_string(), 1, temp_saves("auto"));
        assert!(!game.autosave_exists("Greg".to_string()));
        game.buy("food".to_string(), 1).unwrap();

        assert!(game.autosave().ok);

        assert!(game.autosave_exists(" greg ".to_string()));
        let saves = game.saves();
        assert!(saves[0].is_autosave);
        game.new_game("Greg".to_string());
        assert_eq!(game.cargo().weight, 0);
        assert!(game.continue_game("greg".to_string()).ok);
        assert_eq!(game.cargo().weight, 1);
        assert!(!game.continue_game("nobody".to_string()).ok);
        assert!(game.delete_slot(saves[0].slot.clone()).ok);
        assert!(game.saves().is_empty());
        assert!(!game.delete_slot("gone".to_string()).ok);
    }

    #[test]
    fn a_new_game_needs_a_name_and_ends_any_journey() {
        let game = Game::with_saves_in("Test".to_string(), 1, temp_saves("new"));
        assert!(!game.new_game("   ".to_string()).ok);
        let step = game.begin_jump("Proxima Centauri".to_string());
        if step.kind != "arrived" {
            assert!(game.journey_in_progress());
            assert!(game.new_game("Fresh".to_string()).ok);
            assert!(!game.journey_in_progress());
        }
    }

    fn game_in_a_fight() -> Option<(Arc<Game>, StepData)> {
        (0..400).find_map(|seed| {
            let game = Game::with_seed("Test".to_string(), seed);
            let mut step = game.begin_jump("Proxima Centauri".to_string());
            for _ in 0..20 {
                step = match step.kind.as_str() {
                    "fight" => return Some((game, step)),
                    "event" => game.answer_event(0),
                    "inspection" => game.answer_inspection("hand_over".to_string()),
                    _ => return None,
                };
            }
            None
        })
    }

    #[test]
    fn a_fight_can_be_played_round_by_round_and_handed_back() {
        let (game, step) = game_in_a_fight().expect("a seed with a fight");
        assert_eq!(step.kind, "fight");

        let before = game.fight();
        assert!(before.active);
        assert_eq!(before.outcome, "ongoing");
        assert!(before.log.is_empty());
        assert!(before.player.weapons > 0);
        assert!(!game.finish_fight().message.is_empty());

        let mut fight = before;
        for _ in 0..40 {
            if fight.outcome != "ongoing" {
                break;
            }
            fight = game.fight_action("attack".to_string());
        }
        assert_ne!(fight.outcome, "ongoing");
        assert!(fight.log.len() >= 2);

        let next = game.finish_fight();
        assert_ne!(next.kind, "error", "{}", next.message);
    }

    #[test]
    fn fleeing_and_surrendering_end_or_continue_a_fight() {
        let (game, _) = game_in_a_fight().expect("a seed with a fight");

        let surrendered = game.fight_action("surrender".to_string());

        assert_eq!(surrendered.outcome, "surrendered");
        assert!(surrendered
            .log
            .iter()
            .any(|line| line.contains("surrender")));
        assert!(!game.fight_action("attack".to_string()).message.is_empty());
        assert!(game
            .fight_action("dance".to_string())
            .message
            .contains("Unknown"));
    }

    #[test]
    fn there_is_no_fight_outside_one() {
        let game = Game::with_seed("Test".to_string(), 1);

        assert!(!game.fight().active);
        assert!(!game.fight_action("attack".to_string()).active);
        assert_eq!(game.finish_fight().kind, "error");
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
    fn selling_all_sells_one_commodity_or_the_whole_hold() {
        let game = Game::new("Test".to_string());
        game.buy("food".to_string(), 1).unwrap();
        game.buy("textiles".to_string(), 1).unwrap();

        let one = game.sell_all(Some("food".to_string())).unwrap();
        assert!(one.message.starts_with("Sold 1 tons of Food"));
        assert_eq!(game.market().unwrap().cargo_used, 1);

        let everything = game.sell_all(None).unwrap();
        assert!(everything.message.contains("1 tons of Textiles"));
        assert_eq!(game.market().unwrap().cargo_used, 0);
        assert!(game.sell_all(None).is_err());
        assert!(game.sell_all(Some("food".to_string())).is_err());
    }

    #[test]
    fn refusals_carry_the_core_message() {
        let game = Game::new("Test".to_string());

        let error = game.buy("food".to_string(), 999).unwrap_err();
        assert!(error.to_string().contains("Only") || error.to_string().contains("Insufficient"));
        assert!(game.buy("gold".to_string(), 1).is_err());
    }
}
