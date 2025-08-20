use colored::*;
use crate::game::GameState;
use crate::data::systems::StarSystem;

/// Print welcome message
pub fn print_welcome(commander_name: &str) {
    println!();
    println!("{}", "Welcome to Elite Rust!".green().bold());
    println!("{}", "=====================".green());
    println!("Commander {}, your journey begins!", commander_name.yellow());
    println!("Type 'help' for available commands.");
    println!();
}

/// Print goodbye message
pub fn print_goodbye(commander_name: &str) {
    println!();
    println!("Safe travels, Commander {}!", commander_name.yellow());
    println!("{}", "Thanks for playing Elite Rust!".green());
}

/// Show help menu with available commands
pub fn show_help() {
    println!();
    println!("{}", "Available Commands:".cyan().bold());
    println!("{}", "==================".cyan());
    println!("{} - Show this help menu", "help".yellow());
    println!("{} - Display ship and commander status", "status".yellow());
    println!("{} - List all available star systems", "systems".yellow());
    println!("{} - Show information about a system", "info <system>".yellow());
    println!("{} - Travel to a star system", "jump <system>".yellow());
    println!("{} - Exit the game", "quit".yellow());
    println!();
}

/// Show status display with all required information
pub fn show_status(game_state: &GameState) {
    let commander = &game_state.commander;
    let ship = &game_state.ship;
    
    println!();
    println!("{}", "Commander Status".cyan().bold());
    println!("{}", "================".cyan());
    println!("Commander: {}", commander.name().yellow());
    println!("Credits: {} cr", commander.credits().to_string().green());
    println!("Location: {}", commander.location().blue());
    println!();
    println!("{}", "Ship Status".cyan().bold());
    println!("{}", "===========".cyan());
    println!("Ship: {}", ship.ship_type().yellow());
    println!("Cargo: {}/{} tons", ship.cargo_used(), ship.cargo_capacity());
    println!("Fuel: {}/{} light years", ship.fuel_current(), ship.fuel_capacity());
    println!("Hull: {}%", ship.hull_percentage());
    
    // Add status indicators
    if ship.is_damaged() {
        println!("{}", "⚠️  Hull damage detected!".red());
    }
    
    if ship.fuel_current() < ship.fuel_capacity() / 3 {
        println!("{}", "⚠️  Low fuel warning!".yellow());
    }
    
    if ship.cargo_used() > ship.cargo_capacity() * 3 / 4 {
        println!("{}", "⚠️  Cargo hold nearly full!".yellow());
    }
    
    println!();
}

/// Show all available star systems
pub fn show_systems(game_state: &GameState) {
    println!();
    println!("{}", "Available Star Systems".cyan().bold());
    println!("{}", "=====================".cyan());
    
    let systems = game_state.get_available_systems();
    let current_location = &game_state.commander.location;
    
    for system in systems {
        let distance = game_state.calculate_travel_cost(system.name())
            .unwrap_or(0.0);
        
        let location_indicator = if system.name() == current_location {
            "📍 ".green()
        } else {
            "   ".normal()
        };
        
        println!("{}{} - {} ({} light years)", 
            location_indicator,
            system.name().yellow(),
            system.economy().display_name().blue(),
            distance.to_string().cyan()
        );
    }
    println!();
}

/// Show detailed information about a specific system
pub fn show_system_info(system: &StarSystem, game_state: &GameState) {
    println!();
    println!("{}", format!("System: {}", system.name()).cyan().bold());
    println!("{}", "=".repeat(system.name().len() + 8).cyan());
    println!("Economy: {}", system.economy().display_name().blue());
    println!("Distance from Lave: {} light years", system.distance_from_lave().to_string().cyan());
    
    let distance_from_current = game_state.calculate_travel_cost(system.name());
    if let Some(distance) = distance_from_current {
        println!("Distance from current location: {} light years", distance.to_string().cyan());
        println!("Fuel required: {} light years", distance.to_string().yellow());
    }
    
    println!("Available commodities: {}", system.available_commodities().join(", ").green());
    println!("Description: {}", system.description().white());
    println!();
}

/// Show travel confirmation
pub fn show_travel_confirmation(from: &str, to: &str, fuel_cost: f64) {
    println!();
    println!("{}", "Travel Confirmation".cyan().bold());
    println!("{}", "===================".cyan());
    println!("From: {}", from.yellow());
    println!("To: {}", to.yellow());
    println!("Fuel cost: {} light years", fuel_cost.to_string().red());
    println!("{}", "Confirm travel? (y/n): ".cyan());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{Commander, Ship, GameState};

    #[test]
    fn test_show_help() {
        // This test just ensures the function doesn't panic
        show_help();
    }

    #[test]
    fn test_show_status() {
        let commander = Commander::new("Test Commander".to_string());
        let ship = Ship::new_cobra_mk3();
        let system_manager = crate::data::systems::SystemManager::new();
        let game_state = GameState { commander, ship, system_manager };
        
        // This test just ensures the function doesn't panic
        show_status(&game_state);
    }

    #[test]
    fn test_show_status_with_damage() {
        let commander = Commander::new("Test Commander".to_string());
        let mut ship = Ship::new_cobra_mk3();
        ship.take_damage(30); // Add some damage
        let system_manager = crate::data::systems::SystemManager::new();
        let game_state = GameState { commander, ship, system_manager };
        
        // This test just ensures the function doesn't panic
        show_status(&game_state);
    }

    #[test]
    fn test_show_status_with_low_fuel() {
        let commander = Commander::new("Test Commander".to_string());
        let mut ship = Ship::new_cobra_mk3();
        ship.consume_fuel(5); // Consume most fuel
        let system_manager = crate::data::systems::SystemManager::new();
        let game_state = GameState { commander, ship, system_manager };
        
        // This test just ensures the function doesn't panic
        show_status(&game_state);
    }
}
