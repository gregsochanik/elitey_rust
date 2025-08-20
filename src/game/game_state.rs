use crate::utils::input;
use crate::data::systems::SystemManager;
use colored::*;
use std::io::Write;

use super::{Commander, Ship, TradingHandler, UpgradeHandler, SaveLoadHandler};

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
}

#[allow(dead_code)]
impl GameState {
    /// Create a new game with commander creation
    pub fn new_game() -> Self {
        println!("{}", "Welcome to Elite Rust!".green().bold());
        println!("{}", "=====================".green());
        
        // Get commander name
        let name = Self::get_commander_name();
        
        Self {
            commander: Commander::new(name),
            ship: Ship::new_cobra_mk3(),
            system_manager: SystemManager::new().expect("Failed to load galaxy data"),
            trading_handler: TradingHandler::new(),
            upgrade_handler: UpgradeHandler::new(),
            save_load_handler: SaveLoadHandler::new().expect("Failed to create save/load handler"),
        }
    }

    /// Load existing game or create new one
    pub fn load_or_new() -> Self {
        // For now, always create new game
        // TODO: Implement save/load system in Feature 05
        Self::new_game()
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
        if !self.system_manager.can_travel(current_location, destination, self.ship.fuel_current as f64) {
            return Err("Insufficient fuel for this journey.".to_string());
        }
        
        // Calculate fuel cost
        let fuel_cost = self.system_manager.calculate_fuel_cost(current_location, destination)
            .ok_or("Unable to calculate fuel cost.")?;
        
        // Consume fuel
        self.ship.consume_fuel(fuel_cost as u32);
        
        // Update location
        self.commander.location = destination.to_string();
        
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
        self.system_manager.calculate_fuel_cost(&self.commander.location, destination)
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

    /// Handle buy command
    pub fn handle_buy_command(&mut self, args: &[String]) -> bool {
        self.trading_handler.handle_buy(
            args,
            &mut self.commander,
            &mut self.ship,
            &self.system_manager,
        )
    }

    /// Handle sell command
    pub fn handle_sell_command(&mut self, args: &[String]) -> bool {
        self.trading_handler.handle_sell(
            args,
            &mut self.commander,
            &mut self.ship,
            &self.system_manager,
        )
    }

    /// Handle market command
    pub fn handle_market_command(&self) {
        self.trading_handler.handle_market(
            &self.system_manager,
            &self.commander.location,
        );
    }

    /// Handle cargo command
    pub fn handle_cargo_command(&self) {
        self.trading_handler.handle_cargo(&self.ship);
    }

    /// Handle upgrade command
    pub fn handle_upgrade_command(&self) {
        self.upgrade_handler.handle_upgrade(&self.commander, &self.ship);
    }

    /// Handle upgrade purchase command
    pub fn handle_upgrade_purchase_command(&mut self, args: &[String]) -> bool {
        self.upgrade_handler.handle_upgrade_purchase(args, &mut self.commander, &mut self.ship)
    }

    /// Handle ships command
    pub fn handle_ships_command(&self) {
        self.upgrade_handler.handle_ships(&self.commander, &self.ship);
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
        let game_state = GameState {
            commander,
            ship,
            system_manager,
            trading_handler: TradingHandler::new(),
            upgrade_handler: UpgradeHandler::new(),
            save_load_handler: SaveLoadHandler::new().expect("Failed to create save/load handler"),
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
        let game_state = GameState { 
            commander, 
            ship, 
            system_manager, 
            trading_handler: TradingHandler::new(),
            upgrade_handler: UpgradeHandler::new(),
            save_load_handler: SaveLoadHandler::new().expect("Failed to create save/load handler"),
        };
        
        assert!(game_state.is_running());
    }

    #[test]
    fn test_game_state_destroyed() {
        let commander = Commander::new("Test".to_string());
        let mut ship = Ship::new_cobra_mk3();
        ship.take_damage(100); // Destroy the ship
        let system_manager = SystemManager::new().expect("Failed to load galaxy data");
        
        let game_state = GameState { 
            commander, 
            ship, 
            system_manager, 
            trading_handler: TradingHandler::new(),
            upgrade_handler: UpgradeHandler::new(),
            save_load_handler: SaveLoadHandler::new().expect("Failed to create save/load handler"),
        };
        
        assert!(!game_state.is_running());
    }
}
