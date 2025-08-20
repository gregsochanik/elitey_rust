use crate::utils::input;
use colored::*;
use std::io::Write;

use super::{Commander, Ship};

/// Represents the complete game state
#[derive(Debug, Clone)]
pub struct GameState {
    /// The player commander
    pub commander: Commander,
    /// The player's ship
    pub ship: Ship,
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
        }
    }

    /// Load existing game or create new one
    pub fn load_or_new() -> Self {
        // For now, always create new game
        // TODO: Implement save/load system in Feature 05
        Self::new_game()
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
        let game_state = GameState {
            commander,
            ship,
        };

        assert_eq!(game_state.commander().name(), "Test Commander");
        assert_eq!(game_state.ship().ship_type(), "Cobra Mk III");
        assert!(game_state.is_running());
    }

    #[test]
    fn test_game_state_running() {
        let commander = Commander::new("Test".to_string());
        let ship = Ship::new_cobra_mk3();
        let game_state = GameState { commander, ship };
        
        assert!(game_state.is_running());
    }

    #[test]
    fn test_game_state_destroyed() {
        let commander = Commander::new("Test".to_string());
        let mut ship = Ship::new_cobra_mk3();
        ship.take_damage(100); // Destroy the ship
        
        let game_state = GameState { commander, ship };
        
        assert!(!game_state.is_running());
    }
}
