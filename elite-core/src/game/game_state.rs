use crate::data::commodities::CommodityType;
use crate::data::star_class::StarClass;
use crate::data::systems::{EconomyType, GovernmentType, SystemManager};
use crate::game::exploration::Exploration;
use crate::game::refuel::{RefuelError, RefuelOutcome};
use crate::game::repair::{RepairError, RepairReceipt};
use crate::game::trading::{CargoView, MarketView, TradeError, TradeReceipt};
use crate::game::travel::fuel_for_distance;
use crate::game::upgrades::{UpgradeError, UpgradeReceipt};
use crate::game::views::{JumpFromHere, StatusView, SystemInfoView, TradeLogView};

use super::jobs::{self, ActiveJob, Job, JobLock};
use super::{
    Commander, EventManager, JobBoard, SaveLoadHandler, Ship, TradeLog, TradingHandler,
    TravelHandler, UpgradeHandler,
};

/// A job on the board and what stops the commander taking it
#[derive(Debug, Clone, PartialEq)]
pub struct JobListing {
    pub job: Job,
    pub lock: Option<JobLock>,
}

/// Represents the complete game state
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GameState {
    /// The player commander
    pub commander: Commander,
    /// The player's ship
    pub ship: Ship,
    /// The system manager for travel
    pub system_manager: SystemManager,
    /// The trading handler
    pub trading_handler: TradingHandler,
    /// The upgrade handler
    pub upgrade_handler: UpgradeHandler,
    /// The save/load handler
    pub save_load_handler: SaveLoadHandler,
    /// The event manager; events are rebuilt on load so saves pick up rule changes
    #[serde(skip)]
    pub event_manager: EventManager,
    /// The travel handler
    pub travel_handler: TravelHandler,
    /// Buy/sell history
    #[serde(default)]
    pub trade_log: TradeLog,
    /// Jobs on offer at the current system
    #[serde(default)]
    pub job_board: JobBoard,
    /// Accepted jobs, numbered by position
    #[serde(default)]
    pub active_jobs: Vec<ActiveJob>,
    /// Jumps made so far; markets change with it
    #[serde(default)]
    pub world_clock: u32,
}

/// A system the ship can reach with its current fuel
#[derive(Debug, Clone, PartialEq)]
pub struct Destination {
    pub name: String,
    pub distance: f64,
    pub fuel: u32,
    pub economy: EconomyType,
    pub government: GovernmentType,
}

impl GameState {
    /// A fresh game for a commander
    pub fn new(name: String) -> Self {
        let system_manager = SystemManager::new().expect("Failed to load galaxy data");
        Self {
            commander: Commander::new(name),
            ship: Ship::new_cobra_mk3(),
            system_manager: system_manager.clone(),
            trading_handler: TradingHandler::new(),
            upgrade_handler: UpgradeHandler::new(),
            save_load_handler: SaveLoadHandler::new().expect("Failed to create save/load handler"),
            event_manager: EventManager::new(),
            travel_handler: TravelHandler::new(system_manager),
            trade_log: TradeLog::new(),
            job_board: JobBoard::new(),
            active_jobs: Vec::new(),
            world_clock: 0,
        }
    }

    /// Get the commander
    pub fn commander(&self) -> &Commander {
        &self.commander
    }

    /// Get mutable reference to commander
    pub fn commander_mut(&mut self) -> &mut Commander {
        &mut self.commander
    }

    /// Get the ship
    pub fn ship(&self) -> &Ship {
        &self.ship
    }

    /// Get mutable reference to ship
    pub fn ship_mut(&mut self) -> &mut Ship {
        &mut self.ship
    }

    /// Check if game should continue
    pub fn is_running(&self) -> bool {
        !self.ship.is_destroyed()
    }

    /// Travel to a new system
    pub fn travel_to_system(&mut self, destination: &str) -> Result<(), String> {
        let current_location = &self.commander.location;

        // Check if destination exists
        if self.system_manager.get_system(destination).is_none() {
            return Err(format!("Unknown system: {}", destination));
        }

        // Check if we're already there
        if current_location == destination {
            return Err("You are already in this system.".to_string());
        }

        // Check if we have enough fuel
        if !self.system_manager.can_travel(
            current_location,
            destination,
            self.ship.fuel_current as f64,
        ) {
            return Err("Insufficient fuel for this journey.".to_string());
        }

        // Calculate fuel cost
        let fuel_cost = self
            .system_manager
            .calculate_fuel_cost(current_location, destination)
            .ok_or("Unable to calculate fuel cost.")?;

        // Consume fuel
        self.ship.consume_fuel(fuel_cost as u32);

        // Update location
        self.commander.location = destination.to_string();
        self.commander.visit(destination);

        self.world_clock += 1;
        self.job_board.clear();
        self.settle_jobs();

        Ok(())
    }

    /// Get all available systems
    pub fn get_available_systems(&self) -> Vec<&crate::data::systems::StarSystem> {
        self.system_manager.get_all_systems()
    }

    /// Move a commander whose saved location is not in the galaxy to Sol, or the first system if there is no Sol
    pub fn relocate_if_lost(&mut self) {
        if self
            .system_manager
            .get_system(&self.commander.location)
            .is_some()
        {
            return;
        }
        let home = if self.system_manager.get_system("Sol").is_some() {
            Some("Sol".to_string())
        } else {
            self.system_manager
                .get_all_systems()
                .into_iter()
                .map(|system| system.name.clone())
                .min()
        };
        if let Some(home) = home {
            self.commander.location = home;
        }
    }

    /// How many of the galaxy's stars the commander has been to
    pub fn exploration(&self) -> Exploration {
        let systems = self.system_manager.get_all_systems();
        let visited = systems
            .iter()
            .filter(|system| self.commander.has_visited(&system.name))
            .count();
        Exploration::new(visited as u32, systems.len() as u32)
    }

    /// Get system information
    pub fn get_system_info(&self, system_name: &str) -> Option<&crate::data::systems::StarSystem> {
        self.system_manager.get_system(system_name)
    }

    /// Calculate travel cost to a system
    pub fn calculate_travel_cost(&self, destination: &str) -> Option<f64> {
        self.system_manager
            .calculate_fuel_cost(&self.commander.location, destination)
    }

    /// Get the trading handler
    pub fn trading_handler(&self) -> &TradingHandler {
        &self.trading_handler
    }

    /// Get mutable reference to trading handler
    pub fn trading_handler_mut(&mut self) -> &mut TradingHandler {
        &mut self.trading_handler
    }

    /// Get the upgrade handler
    pub fn upgrade_handler(&self) -> &UpgradeHandler {
        &self.upgrade_handler
    }

    /// Get mutable reference to upgrade handler
    pub fn upgrade_handler_mut(&mut self) -> &mut UpgradeHandler {
        &mut self.upgrade_handler
    }

    /// Get the save/load handler
    pub fn save_load_handler(&self) -> &SaveLoadHandler {
        &self.save_load_handler
    }

    /// Get mutable reference to save/load handler
    pub fn save_load_handler_mut(&mut self) -> &mut SaveLoadHandler {
        &mut self.save_load_handler
    }

    /// Get the event manager
    pub fn event_manager(&self) -> &EventManager {
        &self.event_manager
    }

    /// Get mutable reference to event manager
    pub fn event_manager_mut(&mut self) -> &mut EventManager {
        &mut self.event_manager
    }

    /// Get the travel handler
    pub fn travel_handler(&self) -> &TravelHandler {
        &self.travel_handler
    }

    /// Get mutable reference to travel handler
    pub fn travel_handler_mut(&mut self) -> &mut TravelHandler {
        &mut self.travel_handler
    }

    /// Buy tons of a commodity at the current system
    pub fn buy(
        &mut self,
        commodity: CommodityType,
        quantity: u32,
    ) -> Result<TradeReceipt, TradeError> {
        self.trading_handler.buy(
            commodity,
            quantity,
            &mut self.commander,
            &mut self.ship,
            &self.system_manager,
            &mut self.trade_log,
            self.world_clock,
        )
    }

    /// Sell tons of a commodity at the current system
    pub fn sell(
        &mut self,
        commodity: CommodityType,
        quantity: u32,
    ) -> Result<TradeReceipt, TradeError> {
        self.trading_handler.sell(
            commodity,
            quantity,
            &mut self.commander,
            &mut self.ship,
            &self.system_manager,
            &mut self.trade_log,
            self.world_clock,
        )
    }

    /// Sell everything held of one commodity, or the whole hold when `None`; one receipt per commodity
    pub fn sell_all(
        &mut self,
        commodity: Option<CommodityType>,
    ) -> Result<Vec<TradeReceipt>, TradeError> {
        let lines: Vec<_> = self
            .cargo_view()
            .items
            .into_iter()
            .filter(|line| {
                commodity
                    .as_ref()
                    .is_none_or(|wanted| *wanted == line.commodity)
            })
            .collect();
        if lines.is_empty() {
            return Err(TradeError::NothingToSell);
        }
        let mut receipts = Vec::new();
        for line in lines {
            receipts.push(self.sell(line.commodity, line.quantity)?);
        }
        Ok(receipts)
    }

    /// The market here as a table
    pub fn market_overview(&mut self) -> Option<MarketView> {
        self.trading_handler.market_overview(
            &self.system_manager,
            &self.commander.location,
            self.commander.standing(),
            self.world_clock,
            &self.ship,
            &self.trade_log,
        )
    }

    /// The cargo hold
    pub fn cargo_view(&self) -> CargoView {
        self.trading_handler.cargo_view(&self.ship, &self.trade_log)
    }

    /// Buy fuel (all that fits and is affordable when `None`), or be rescued if stranded
    pub fn refuel(&mut self, units: Option<u32>) -> Result<RefuelOutcome, RefuelError> {
        let nearest = self.nearest_jump_cost();
        super::refuel::refuel(&mut self.commander, &mut self.ship, units, nearest)
    }

    /// Repair hull damage for credits
    pub fn repair(&mut self, points: Option<u32>) -> Result<RepairReceipt, RepairError> {
        super::repair::repair(&mut self.commander, &mut self.ship, points)
    }

    /// Fuel needed to reach the closest other system
    pub fn nearest_jump_cost(&self) -> u32 {
        let Some(current) = self
            .travel_handler
            .get_system_info(&self.commander.location)
        else {
            return self.ship.fuel_capacity();
        };
        self.travel_handler
            .get_available_systems()
            .iter()
            .filter(|system| system.name != current.name)
            .map(|system| fuel_for_distance(current.distance_to(system)))
            .min()
            .unwrap_or_else(|| self.ship.fuel_capacity())
    }

    /// Recent trades and totals
    pub fn trade_log_view(&self, count: usize) -> TradeLogView {
        TradeLogView {
            recent: self.trade_log.recent(count).into_iter().cloned().collect(),
            stats: self.trade_log.stats(),
            best: self.trade_log.best_trade().cloned(),
        }
    }

    /// The commander and ship at a glance
    pub fn status_view(&self) -> StatusView {
        StatusView {
            commander_name: self.commander.name().to_string(),
            credits: self.commander.credits(),
            location: self.commander.location().to_string(),
            reputation: self.commander.reputation,
            standing: self.commander.standing(),
            kills: self.commander.kills,
            combat_rating: self.commander.combat_rating(),
            ship_name: self.ship.ship_type_name().to_string(),
            cargo_used: self.ship.cargo_used(),
            cargo_capacity: self.ship.cargo_capacity(),
            fuel_current: self.ship.fuel_current(),
            fuel_capacity: self.ship.fuel_capacity(),
            hull: self.ship.hull_percentage(),
            max_hull: self.ship.ship_type.max_hull(),
            power_used: self.ship.equipment.power_used(),
            power_capacity: self.ship.equipment.power_capacity(),
            damaged: self.ship.is_damaged(),
            exploration: self.exploration(),
        }
    }

    /// What is known about a system, with the jump from here
    pub fn system_info(&self, name: &str) -> Option<SystemInfoView> {
        let system = self.travel_handler.get_system_info(name)?;
        let from_here = self
            .travel_handler
            .get_system_info(&self.commander.location)
            .map(|current| {
                let distance = current.distance_to(system);
                JumpFromHere {
                    distance,
                    fuel: fuel_for_distance(distance),
                }
            });
        Some(SystemInfoView {
            name: system.name.clone(),
            economy: system.economy.clone(),
            government: system.government.clone(),
            tech_level: system.tech_level,
            population: system.population.clone(),
            description: system.description.clone(),
            star: StarClass::from_code(&system.star_class),
            visited: self.commander.has_visited(&system.name),
            from_here,
        })
    }

    /// Commodities here whose prices the player's recent trading is still moving
    pub fn pushed_commodities(
        &self,
    ) -> std::collections::HashMap<crate::data::commodities::CommodityType, super::trading::Pushed>
    {
        self.trading_handler
            .pushed_commodities(&self.commander.location, self.world_clock)
    }

    /// The market here as the player sees it
    pub fn market_prices(&mut self) -> Option<crate::data::commodities::MarketPrices> {
        self.trading_handler.market_view(
            &self.system_manager,
            &self.commander.location,
            self.commander.standing(),
            self.world_clock,
        )
    }

    /// Buy a better ship
    pub fn buy_ship(
        &mut self,
        ship_type: crate::data::ships::ShipType,
    ) -> Result<UpgradeReceipt, UpgradeError> {
        self.upgrade_handler
            .upgrade(ship_type, &mut self.commander, &mut self.ship)
    }

    /// The jobs on offer here, rolled with the given randomness on first sight
    pub fn jobs_here(&mut self, rng: &mut impl rand::Rng) -> Option<Vec<JobListing>> {
        let origin = self
            .travel_handler
            .get_system_info(&self.commander.location)?;
        let systems = self.travel_handler.get_available_systems();
        let standing = self.commander.standing();
        let rating = self.commander.combat_rating();
        Some(
            self.job_board
                .jobs_at(origin, &systems, rng, self.ship.cargo_capacity())
                .iter()
                .map(|job| JobListing {
                    lock: job.lock(standing, rating),
                    job: job.clone(),
                })
                .collect(),
        )
    }

    /// The jobs left on the board
    pub fn job_board_jobs(&self) -> &[Job] {
        self.job_board.jobs()
    }

    /// Take job `number` from the board here
    pub fn accept_job(&mut self, number: usize) -> Result<Job, String> {
        jobs::accept(
            &mut self.job_board,
            &mut self.active_jobs,
            &mut self.ship,
            &self.commander.location,
            self.commander.standing(),
            self.commander.combat_rating(),
            number,
        )
    }

    /// Drop active job `number`, losing reputation
    pub fn abandon_job(&mut self, number: usize) -> Result<Job, String> {
        jobs::abandon(
            &mut self.active_jobs,
            &mut self.commander,
            &mut self.ship,
            number,
        )
    }

    /// Systems reachable with the current fuel, nearest first
    pub fn reachable_destinations(&self) -> Option<Vec<Destination>> {
        let current_fuel = self.ship.fuel_current as f64;
        let current_system = self
            .travel_handler
            .get_system_info(&self.commander.location)?;

        let mut destinations: Vec<Destination> = self
            .travel_handler
            .get_available_systems()
            .iter()
            .filter(|system| system.name != current_system.name)
            .filter_map(|system| {
                let distance = current_system.distance_to(system);
                (distance <= current_fuel).then(|| Destination {
                    name: system.name.clone(),
                    distance,
                    fuel: fuel_for_distance(distance),
                    economy: system.economy.clone(),
                    government: system.government.clone(),
                })
            })
            .collect();
        destinations.sort_by(|a, b| {
            a.distance
                .partial_cmp(&b.distance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        Some(destinations)
    }
}

#[cfg(test)]
mod tests;
