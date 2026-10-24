use crate::data::systems::SystemManager;
use crate::game::travel::fuel_for_distance;
use crate::utils::display::{show_error, show_success};
use crate::utils::input;
use colored::*;
use std::io::Write;

use super::jobs::{self, ActiveJob, Job};
use super::{
    Commander, EventManager, JobBoard, SaveLoadHandler, Ship, TradeLog, TradingHandler,
    TravelHandler, UpgradeHandler,
};

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
    /// The event manager
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
}

/// A system the ship can reach with its current fuel
#[derive(Debug, Clone, PartialEq)]
pub struct Destination {
    pub name: String,
    pub distance: f64,
    pub fuel: u32,
}

#[allow(dead_code)]
impl GameState {
    /// Start a new game, asking before replacing an existing autosave
    pub fn new_game() -> Self {
        Self::start(true)
    }

    fn new_game_named(name: String) -> Self {
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
        }
    }

    /// Load existing game or create new one
    pub fn load_or_new() -> Self {
        Self::start(false)
    }

    fn start(force_new: bool) -> Self {
        println!("{}", "Welcome to Elite Rust!".green().bold());
        println!("{}", "=====================".green());

        let name = Self::get_commander_name();
        let handler = SaveLoadHandler::new().expect("Failed to create save/load handler");
        match handler.find_autosave(&name) {
            Ok(None) => Self::new_game_named(name),
            Ok(Some(saved)) => {
                let summary = format!(
                    "Found a save for {} at {} with {} credits.",
                    saved.commander.name().green(),
                    saved.commander.location().cyan(),
                    saved.commander.credits().to_string().green()
                );
                let keep = if force_new {
                    !Self::ask_yes_no(&format!("{} Start over and replace it? (y/n): ", summary))
                        .unwrap_or(false)
                } else {
                    Self::ask_yes_no(&format!("{} Continue it? (y/n): ", summary)).unwrap_or(true)
                };
                if keep {
                    Self::from_save(saved)
                } else {
                    Self::new_game_named(name)
                }
            }
            Err(e) => {
                show_error(&format!("Could not read the save for {}: {}", name, e));
                let replace =
                    Self::ask_yes_no("Start a new game and replace it? (y/n): ").unwrap_or(false);
                if !replace {
                    std::process::exit(1);
                }
                Self::new_game_named(name)
            }
        }
    }

    /// Ask until the answer is yes or no; `None` when input has ended
    fn ask_yes_no(prompt: &str) -> Option<bool> {
        loop {
            print!("{}", prompt);
            std::io::stdout().flush().expect("Failed to flush stdout");
            let answer = input::get_input().ok().filter(|line| !line.is_empty())?;
            match answer.trim().to_lowercase().as_str() {
                "y" | "yes" => return Some(true),
                "n" | "no" => return Some(false),
                _ => println!("Please answer y or n."),
            }
        }
    }

    /// Create game state from existing save
    pub fn from_save(save_state: GameState) -> Self {
        save_state
    }

    /// Get commander name from user input
    fn get_commander_name() -> String {
        loop {
            print!("Enter your commander name: ");
            std::io::stdout().flush().expect("Failed to flush stdout");

            match input::get_input() {
                Ok(input) => {
                    let name = input.trim();
                    if !name.is_empty() {
                        return name.to_string();
                    } else {
                        println!("{}", "Name cannot be empty. Please try again.".red());
                    }
                }
                Err(_) => {
                    println!("{}", "Error reading input. Please try again.".red());
                }
            }
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
        self.trading_handler.handle_buy(
            args,
            &mut self.commander,
            &mut self.ship,
            &self.system_manager,
            &mut self.trade_log,
        )
    }

    /// Handle equipment command
    pub fn handle_equipment_command(&mut self, args: &[String]) -> bool {
        super::equipment::EquipmentHandler::handle(args, &mut self.commander, &mut self.ship)
    }

    /// Handle refuel command
    pub fn handle_refuel_command(&mut self, args: &[String]) -> bool {
        let nearest = self.nearest_jump_cost();
        super::refuel::handle(args, &mut self.commander, &mut self.ship, nearest)
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
        super::repair::handle(args, &mut self.commander, &mut self.ship)
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
        self.trading_handler.handle_sell(
            args,
            &mut self.commander,
            &mut self.ship,
            &self.system_manager,
            &mut self.trade_log,
        )
    }

    /// Handle market command
    pub fn handle_market_command(&mut self) {
        self.trading_handler.handle_market(
            &self.system_manager,
            &self.commander.location,
            self.commander.standing(),
        );
    }

    /// Handle cargo command
    pub fn handle_cargo_command(&self) {
        self.trading_handler.handle_cargo(&self.ship);
    }

    /// Handle upgrade command
    pub fn handle_upgrade_command(&self) {
        self.upgrade_handler
            .handle_upgrade(&self.commander, &self.ship);
    }

    /// Handle upgrade purchase command
    pub fn handle_upgrade_purchase_command(&mut self, args: &[String]) -> bool {
        self.upgrade_handler
            .handle_upgrade_purchase(args, &mut self.commander, &mut self.ship)
    }

    /// Handle ships command
    pub fn handle_ships_command(&self) {
        self.upgrade_handler
            .handle_ships(&self.commander, &self.ship);
    }

    /// Handle save command
    pub fn handle_save_command(&self, args: &[String]) -> bool {
        self.save_load_handler.handle_save(args, self)
    }

    /// Handle load command
    pub fn handle_load_command(&self, args: &[String]) -> Result<Option<GameState>, String> {
        self.save_load_handler.handle_load(args)
    }

    /// Handle list saves command
    pub fn handle_list_saves_command(&self) {
        self.save_load_handler.handle_list_saves();
    }

    /// Handle delete save command
    pub fn handle_delete_save_command(&self, args: &[String]) -> bool {
        self.save_load_handler.handle_delete_save(args)
    }

    /// Handle auto-save
    pub fn handle_auto_save(&self) -> bool {
        self.save_load_handler.auto_save(self)
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
        Some(self.job_board.jobs_at(origin, &systems, rng).to_vec())
    }

    /// Deliver or fail active jobs after arriving at the commander's location
    pub fn settle_jobs(&mut self) {
        let arrived_at = self.commander.location.clone();
        for message in jobs::settle_arrival(&mut self.active_jobs, &mut self.commander, &arrived_at)
        {
            println!("{}", message);
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
            &self.commander.location,
            number,
        )
    }

    /// Drop active job `number`, losing reputation
    pub fn abandon_job(&mut self, number: usize) -> Result<Job, String> {
        jobs::abandon(&mut self.active_jobs, &mut self.commander, number)
    }

    /// Handle accept command
    pub fn handle_accept_command(&mut self, args: &[String]) {
        let Some(number) = Self::job_number(args, "accept") else {
            return;
        };
        match self.accept_job(number) {
            Ok(job) => show_success(&format!(
                "Accepted: courier to {} for {}cr within {} jumps.",
                job.destination, job.reward, job.deadline_jumps
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
                    "Abandoned the courier job to {}. Reputation {:+}.",
                    job.destination,
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
mod tests {
    use super::*;

    #[test]
    fn test_game_state_creation() {
        // This test would need to be modified to handle user input
        // For now, we'll test the basic structure
        let commander = Commander::new("Test Commander".to_string());
        let ship = Ship::new_cobra_mk3();
        let system_manager = SystemManager::new().expect("Failed to load galaxy data");
        let system_manager_clone = system_manager.clone();
        let game_state = GameState {
            commander,
            ship,
            system_manager,
            trading_handler: TradingHandler::new(),
            upgrade_handler: UpgradeHandler::new(),
            save_load_handler: SaveLoadHandler::new().expect("Failed to create save/load handler"),
            event_manager: EventManager::new(),
            travel_handler: TravelHandler::new(system_manager_clone),
            trade_log: TradeLog::new(),
            job_board: JobBoard::new(),
            active_jobs: Vec::new(),
        };

        assert_eq!(game_state.commander().name(), "Test Commander");
        assert_eq!(game_state.ship().ship_type_name(), "Cobra Mk III");
        assert!(game_state.is_running());
    }

    #[test]
    fn test_game_state_running() {
        let commander = Commander::new("Test".to_string());
        let ship = Ship::new_cobra_mk3();
        let system_manager = SystemManager::new().expect("Failed to load galaxy data");
        let system_manager_clone = system_manager.clone();
        let game_state = GameState {
            commander,
            ship,
            system_manager,
            trading_handler: TradingHandler::new(),
            upgrade_handler: UpgradeHandler::new(),
            save_load_handler: SaveLoadHandler::new().expect("Failed to create save/load handler"),
            event_manager: EventManager::new(),
            travel_handler: TravelHandler::new(system_manager_clone),
            trade_log: TradeLog::new(),
            job_board: JobBoard::new(),
            active_jobs: Vec::new(),
        };

        assert!(game_state.is_running());
    }

    #[test]
    fn test_game_state_destroyed() {
        let commander = Commander::new("Test".to_string());
        let mut ship = Ship::new_cobra_mk3();
        ship.take_damage(100); // Destroy the ship
        let system_manager = SystemManager::new().expect("Failed to load galaxy data");
        let system_manager_clone = system_manager.clone();

        let game_state = GameState {
            commander,
            ship,
            system_manager,
            trading_handler: TradingHandler::new(),
            upgrade_handler: UpgradeHandler::new(),
            save_load_handler: SaveLoadHandler::new().expect("Failed to create save/load handler"),
            event_manager: EventManager::new(),
            travel_handler: TravelHandler::new(system_manager_clone),
            trade_log: TradeLog::new(),
            job_board: JobBoard::new(),
            active_jobs: Vec::new(),
        };

        assert!(!game_state.is_running());
    }

    #[test]
    fn trade_log_missing_from_old_save_loads_empty() {
        let system_manager = SystemManager::new().expect("Failed to load galaxy data");
        let state = GameState {
            commander: Commander::new("Old".to_string()),
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
        };
        let mut value = serde_json::to_value(&state).unwrap();
        value.as_object_mut().unwrap().remove("trade_log");

        let loaded: GameState = serde_json::from_value(value).unwrap();
        assert!(loaded.trade_log.transactions().is_empty());
    }

    #[test]
    fn trade_log_survives_serialization() {
        let system_manager = SystemManager::new().expect("Failed to load galaxy data");
        let mut state = GameState {
            commander: Commander::new("New".to_string()),
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
        };
        state.trade_log.record_buy(
            crate::data::commodities::CommodityType::Food,
            5,
            10,
            "LAVE",
            0,
        );

        let json = serde_json::to_string(&state).unwrap();
        let loaded: GameState = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.trade_log.transactions().len(), 1);
    }
}
