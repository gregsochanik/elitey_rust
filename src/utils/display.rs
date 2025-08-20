use colored::*;
use crate::game::GameState;

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
        let game_state = GameState { commander, ship };
        
        // This test just ensures the function doesn't panic
        show_status(&game_state);
    }

    #[test]
    fn test_show_status_with_damage() {
        let commander = Commander::new("Test Commander".to_string());
        let mut ship = Ship::new_cobra_mk3();
        ship.take_damage(30); // Add some damage
        let game_state = GameState { commander, ship };
        
        // This test just ensures the function doesn't panic
        show_status(&game_state);
    }

    #[test]
    fn test_show_status_with_low_fuel() {
        let commander = Commander::new("Test Commander".to_string());
        let mut ship = Ship::new_cobra_mk3();
        ship.consume_fuel(5); // Consume most fuel
        let game_state = GameState { commander, ship };
        
        // This test just ensures the function doesn't panic
        show_status(&game_state);
    }
}
