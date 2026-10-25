use crate::data::commodities::CommodityType;
use crate::data::systems::SystemManager;
use crate::game::refuel::{RefuelError, RefuelOutcome};
use crate::game::repair::{RepairError, RepairReceipt};
use crate::game::trading::{CargoView, MarketView, TradeError, TradeReceipt};
use crate::game::travel::fuel_for_distance;
use crate::game::upgrades::{UpgradeError, UpgradeReceipt};
use crate::utils::display::{show_error, show_success};
use crate::utils::input;
use colored::*;
use std::io::Write;

use super::combat::{Combat, Combatant, Outcome};
use super::fight::run_fight;
use super::jobs::{self, ActiveJob, Job};
use super::smuggling;
use super::{
    Commander, EventManager, JobBoard, SaveLoadHandler, Ship, TradeLog, TradingHandler,
    TravelHandler, UpgradeHandler,
};
use rand::Rng;

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

        self.world_clock += 1;
        self.job_board.clear();
        self.settle_jobs();

        Ok(())
    }

    /// Get all available systems
    pub fn get_available_systems(&self) -> Vec<&crate::data::systems::StarSystem> {
        self.system_manager.get_all_systems()
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

    /// Handle buy command
    pub fn handle_buy_command(&mut self, args: &[String]) -> bool {
        crate::cli::trading::buy(self, args)
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

    /// The market here as a table
    pub fn market_overview(&mut self) -> Option<MarketView> {
        self.trading_handler.market_overview(
            &self.system_manager,
            &self.commander.location,
            self.commander.standing(),
            self.world_clock,
        )
    }

    /// The cargo hold
    pub fn cargo_view(&self) -> CargoView {
        self.trading_handler.cargo_view(&self.ship)
    }

    /// Handle equipment command
    pub fn handle_equipment_command(&mut self, args: &[String]) -> bool {
        crate::cli::equipment::handle(args, &mut self.commander, &mut self.ship)
    }

    /// Handle refuel command
    pub fn handle_refuel_command(&mut self, args: &[String]) -> bool {
        crate::cli::upkeep::refuel(self, args)
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

    /// Handle repair command
    pub fn handle_repair_command(&mut self, args: &[String]) -> bool {
        crate::cli::upkeep::repair(self, args)
    }

    /// Handle log command
    pub fn handle_log_command(&self, args: &[String]) {
        let count = match args.first() {
            None => 10,
            Some(arg) => match arg.parse::<usize>() {
                Ok(n) if n > 0 => n,
                _ => {
                    show_error("Usage: log [count]");
                    return;
                }
            },
        };
        crate::utils::display::show_trade_log(&self.trade_log, count);
    }

    /// Handle sell command
    pub fn handle_sell_command(&mut self, args: &[String]) -> bool {
        crate::cli::trading::sell(self, args)
    }

    /// Handle market command
    pub fn handle_market_command(&mut self) {
        crate::cli::trading::market(self);
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

    /// Handle cargo command
    pub fn handle_cargo_command(&self) {
        crate::cli::trading::cargo(self);
    }

    /// Handle upgrade command
    pub fn handle_upgrade_command(&self) {
        crate::cli::upgrades::show_upgrades(self);
    }

    /// Handle upgrade purchase command
    pub fn handle_upgrade_purchase_command(&mut self, args: &[String]) -> bool {
        crate::cli::upgrades::buy(self, args)
    }

    /// Buy a better ship
    pub fn buy_ship(
        &mut self,
        ship_type: crate::data::ships::ShipType,
    ) -> Result<UpgradeReceipt, UpgradeError> {
        self.upgrade_handler
            .upgrade(ship_type, &mut self.commander, &mut self.ship)
    }

    /// Handle ships command
    pub fn handle_ships_command(&self) {
        crate::cli::upgrades::show_ships(self);
    }

    /// Handle save command
    pub fn handle_save_command(&self, args: &[String]) -> bool {
        crate::cli::saves::save(self, args)
    }

    /// Handle load command
    pub fn handle_load_command(&self, args: &[String]) -> Result<Option<GameState>, String> {
        crate::cli::saves::load(self, args)
    }

    /// Handle list saves command
    pub fn handle_list_saves_command(&self) {
        crate::cli::saves::list(self);
    }

    /// Handle delete save command
    pub fn handle_delete_save_command(&self, args: &[String]) -> bool {
        crate::cli::saves::delete(self, args)
    }

    /// Handle auto-save
    pub fn handle_auto_save(&self) -> bool {
        crate::cli::saves::auto_save(self)
    }

    /// Handle jump command
    pub fn handle_jump_command(&mut self, args: &[String]) -> bool {
        if args.is_empty() {
            show_error("Usage: jump <system_name>");
            return false;
        }

        let destination = &args[0];
        match self.travel_handler.travel_to_system(
            &mut self.commander,
            &mut self.ship,
            &self.event_manager,
            destination,
        ) {
            Ok(success) => {
                if success {
                    self.world_clock += 1;
                    self.job_board.clear();
                    self.settle_jobs();
                }
                success
            }
            Err(e) => {
                show_error(&e);
                false
            }
        }
    }

    /// The jobs on offer here, rolled with the given randomness on first sight
    pub fn jobs_here(&mut self, rng: &mut impl rand::Rng) -> Option<Vec<Job>> {
        let origin = self
            .travel_handler
            .get_system_info(&self.commander.location)?;
        let systems = self.travel_handler.get_available_systems();
        Some(
            self.job_board
                .jobs_at(
                    origin,
                    &systems,
                    rng,
                    self.commander.standing(),
                    self.ship.cargo_capacity(),
                )
                .to_vec(),
        )
    }

    /// Deliver, inspect, fight or fail active jobs after arriving at the commander's location
    pub fn settle_jobs(&mut self) {
        let arrived_at = self.commander.location.clone();
        let arrival = jobs::settle_arrival(
            &mut self.active_jobs,
            &mut self.commander,
            &mut self.ship,
            &arrived_at,
        );
        for message in &arrival.messages {
            println!("{}", message);
        }

        if !arrival.smuggling.is_empty() {
            let government = self
                .travel_handler
                .get_system_info(&arrived_at)
                .map(|system| system.government.clone());
            if let Some(government) = government {
                let messages = smuggling::resolve_smuggling(
                    arrival.smuggling,
                    &mut self.commander,
                    &mut self.ship,
                    &government,
                    &mut rand::thread_rng(),
                    &mut Self::ask_inspection_choice,
                    &mut |ship: &Ship| Self::play_fight(ship, Combatant::patrol_vessel()),
                );
                for message in messages {
                    println!("{}", message);
                }
            }
        }

        if !arrival.bounties.is_empty() {
            let messages = jobs::resolve_bounties(
                arrival.bounties,
                &mut self.commander,
                &mut self.ship,
                &mut |ship: &Ship| Self::play_fight(ship, Combatant::pirate_raider()),
                &mut rand::thread_rng(),
            );
            for message in messages {
                println!("{}", message);
            }
        }
    }

    /// Fight `enemy` at the terminal and return how it ended and the player's remaining hull
    fn play_fight(ship: &Ship, enemy: Combatant) -> std::io::Result<(Outcome, u32)> {
        let mut combat = Combat::new(Combatant::from_ship(ship), enemy);
        let mut dice = rand::thread_rng();
        let mut roll = || dice.gen::<f64>();
        let outcome = run_fight(
            &mut combat,
            &mut std::io::stdin().lock(),
            &mut std::io::stdout(),
            &mut roll,
        )?;
        Ok((outcome, combat.player.hull))
    }

    /// Ask what to do about an inspection; running out of input hands the cargo over
    fn ask_inspection_choice(bribe_cost: u32, can_bribe: bool) -> smuggling::InspectionChoice {
        use smuggling::InspectionChoice;
        loop {
            if can_bribe {
                print!(
                    "[b]ribe ({}cr), [r]un or [h]and over the cargo? ",
                    bribe_cost
                );
            } else {
                print!(
                    "[r]un or [h]and over the cargo (you cannot afford the {}cr bribe)? ",
                    bribe_cost
                );
            }
            std::io::stdout().flush().expect("Failed to flush stdout");
            let line = match input::get_input() {
                Ok(line) if !line.is_empty() => line,
                _ => return InspectionChoice::HandOver,
            };
            match line.trim().to_lowercase().as_str() {
                "b" | "bribe" if can_bribe => return InspectionChoice::Bribe,
                "r" | "run" => return InspectionChoice::Run,
                "h" | "hand" | "hand over" => return InspectionChoice::HandOver,
                _ => println!("Please choose one of the options."),
            }
        }
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

    /// Handle accept command
    pub fn handle_accept_command(&mut self, args: &[String]) {
        let Some(number) = Self::job_number(args, "accept") else {
            return;
        };
        match self.accept_job(number) {
            Ok(job) => show_success(&format!(
                "Accepted: {} for {}cr within {} jumps.",
                job.describe(),
                job.reward,
                job.deadline_jumps
            )),
            Err(e) => show_error(&e),
        }
    }

    /// Handle abandon command
    pub fn handle_abandon_command(&mut self, args: &[String]) {
        let Some(number) = Self::job_number(args, "abandon") else {
            return;
        };
        match self.abandon_job(number) {
            Ok(job) => println!(
                "{}",
                format!(
                    "Abandoned: {}. Reputation {:+}.",
                    job.describe(),
                    jobs::ABANDON_REPUTATION
                )
                .yellow()
            ),
            Err(e) => show_error(&e),
        }
    }

    /// Handle missions command
    pub fn handle_missions_command(&self) {
        crate::utils::display::show_missions(&self.active_jobs);
    }

    fn job_number(args: &[String], command: &str) -> Option<usize> {
        match args.first().map(|arg| arg.parse::<usize>()) {
            Some(Ok(number)) if number > 0 && args.len() == 1 => Some(number),
            _ => {
                show_error(&format!("Usage: {} <number>", command));
                None
            }
        }
    }

    /// Handle jobs command
    pub fn handle_jobs_command(&mut self) {
        match self.jobs_here(&mut rand::thread_rng()) {
            Some(jobs) => crate::utils::display::show_jobs(&self.commander.location, &jobs),
            None => show_error("Unable to determine current location"),
        }
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

    /// Handle systems command
    pub fn handle_systems_command(&self) {
        let Some(reachable_systems) = self.reachable_destinations() else {
            show_error("Unable to determine current location");
            return;
        };
        let current_fuel = self.ship.fuel_current;

        println!();
        println!("{}", "Available Star Systems".cyan().bold());
        println!("{}", "=====================".cyan());
        println!("Current fuel: {} light years", current_fuel);
        println!();

        if reachable_systems.is_empty() {
            println!("{}", "No systems reachable with current fuel.".yellow());
            println!("{}", "Use 'refuel' to top up the tank.".yellow());
        } else {
            for destination in reachable_systems {
                println!(
                    "{} - {:.1} light years, {} fuel",
                    destination.name.yellow(),
                    destination.distance,
                    destination.fuel
                );
            }
        }
        println!();
    }

    /// Handle info command
    pub fn handle_info_command(&self, args: &[String]) -> bool {
        if args.is_empty() {
            show_error("Usage: info <system_name>");
            return false;
        }

        let system_name = &args[0];
        if let Some(system) = self.travel_handler.get_system_info(system_name) {
            self.show_system_info(system);
            true
        } else {
            show_error(&format!("System '{}' not found", system_name));
            false
        }
    }

    /// Show detailed system information
    fn show_system_info(&self, system: &crate::data::systems::StarSystem) {
        println!();
        println!("{}", format!("System: {}", system.name).cyan().bold());
        println!("{}", "=".repeat(30));
        println!("Economy: {:?}", system.economy);
        println!("Government: {:?}", system.government);
        println!("Tech Level: {}", system.tech_level);
        println!("Population: {}", system.population);
        println!("Description: {}", system.description);

        // Show distance from current location
        if let Some(current_system) = self
            .travel_handler
            .get_system_info(&self.commander.location)
        {
            let distance = current_system.distance_to(system);
            println!("Distance: {:.1} light years", distance);
            println!("Fuel required: {} light years", fuel_for_distance(distance));
        }
        println!();
    }
}

#[cfg(test)]
mod tests;
