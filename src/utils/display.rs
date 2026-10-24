use crate::game::trade_log::TradeKind;
use crate::game::{GameState, TradeLog};
use colored::*;

/// Print welcome message
pub fn print_welcome(commander_name: &str) {
    println!();
    println!("{}", "Welcome to Elite Rust!".green().bold());
    println!("{}", "=====================".green());
    println!(
        "Commander {}, your journey begins!",
        commander_name.yellow()
    );
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
    println!(
        "{} - Show information about a system",
        "info <system>".yellow()
    );
    println!(
        "{} - Travel to a star system (with events!)",
        "jump <system>".yellow()
    );
    println!();
    println!("{}", "Trading Commands:".cyan().bold());
    println!("{}", "=================".cyan());
    println!(
        "{} - Show current system's market prices",
        "market".yellow()
    );
    println!("{} - Buy commodities", "buy <item> <quantity>".yellow());
    println!("{} - Sell commodities", "sell <item> <quantity>".yellow());
    println!("{} - Show cargo hold contents", "cargo".yellow());
    println!(
        "{} - Show trading history and statistics",
        "log [count]".yellow()
    );
    println!();
    println!("{}", "Ship Upgrade Commands:".cyan().bold());
    println!("{}", "=====================".cyan());
    println!("{} - Show available ship upgrades", "upgrade".yellow());
    println!("{} - Purchase ship upgrade", "upgrade <ship>".yellow());
    println!("{} - List all available ships", "ships".yellow());
    println!();
    println!("{}", "Equipment Commands:".cyan().bold());
    println!("{}", "===================".cyan());
    println!("{} - Show equipment for sale", "equipment".yellow());
    println!(
        "{} - Buy and install equipment",
        "equipment buy <item>".yellow()
    );
    println!(
        "{} - Sell installed equipment (half price)",
        "equipment sell <item>".yellow()
    );
    println!(
        "{} - Show installed equipment, slots and power",
        "equipment installed".yellow()
    );
    println!();
    println!("{}", "Repair Commands:".cyan().bold());
    println!("{}", "================".cyan());
    println!(
        "{} - Repair hull damage (2 credits per point)",
        "repair [points]".yellow()
    );
    println!();
    println!("{}", "Save/Load Commands:".cyan().bold());
    println!("{}", "===================".cyan());
    println!("{} - Save game to slot", "save <slot>".yellow());
    println!("{} - Load game from slot", "load <slot>".yellow());
    println!("{} - List all save files", "saves".yellow());
    println!("{} - Delete save file", "delete <slot>".yellow());
    println!();
    println!("{} - Exit the game", "quit".yellow());
    println!();
}

/// Show recent trades and trading statistics
pub fn show_trade_log(log: &TradeLog, count: usize) {
    println!("\n{}", "Trading Log".cyan().bold());
    println!("{}", "===========".cyan());

    let recent = log.recent(count);
    if recent.is_empty() {
        println!("No trades yet.\n");
        return;
    }

    println!("Recent Transactions:");
    for t in recent {
        let name = t.commodity.name();
        match (t.kind, t.profit) {
            (TradeKind::Sell, Some(profit)) => {
                let profit_str = format!("{:+}cr profit", profit);
                let coloured = if profit >= 0 {
                    profit_str.green()
                } else {
                    profit_str.red()
                };
                println!(
                    "- Sold {} {} at {} for {}cr ({})",
                    t.quantity,
                    name,
                    t.location,
                    t.total(),
                    coloured
                );
            }
            _ => println!(
                "- Bought {} {} at {} for {}cr",
                t.quantity,
                name,
                t.location,
                t.total()
            ),
        }
    }

    let stats = log.stats();
    println!("\nStatistics:");
    println!("- Total Trades: {}", stats.total_trades);
    println!("- Net Profit: {}cr", stats.net_profit);
    if let Some(best) = log.best_trade() {
        println!(
            "- Best Trade: {:+}cr ({} {} at {})",
            best.profit.unwrap_or(0),
            best.quantity,
            best.commodity.name(),
            best.location
        );
    }
    if let Some(rate) = stats.success_rate {
        println!("- Success Rate: {:.0}%", rate * 100.0);
    }
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
    println!("Reputation: {}", commander.reputation);
    println!();
    println!("{}", "Ship Status".cyan().bold());
    println!("{}", "===========".cyan());
    println!("Ship: {}", ship.ship_type_name().yellow());
    println!(
        "Cargo: {}/{} tons",
        ship.cargo_used(),
        ship.cargo_capacity()
    );
    println!(
        "Fuel: {}/{} light years",
        ship.fuel_current(),
        ship.fuel_capacity()
    );
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

/// Show error message
pub fn show_error(message: &str) {
    println!("{}", format!("Error: {}", message).red());
}

/// Show success message
pub fn show_success(message: &str) {
    println!("{}", format!("Success: {}", message).green());
}

/// Show info message
pub fn show_info(message: &str) {
    println!("{}", format!("Info: {}", message).blue());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{Commander, GameState, Ship};

    #[test]
    fn test_show_help() {
        // This test just ensures the function doesn't panic
        show_help();
    }

    #[test]
    fn test_show_status() {
        let commander = Commander::new("Test Commander".to_string());
        let ship = Ship::new_cobra_mk3();
        let system_manager =
            crate::data::systems::SystemManager::new().expect("Failed to load galaxy data");
        let system_manager_clone = system_manager.clone();
        let game_state = GameState {
            commander,
            ship,
            system_manager,
            trading_handler: crate::game::TradingHandler::new(),
            upgrade_handler: crate::game::UpgradeHandler::new(),
            save_load_handler: crate::game::SaveLoadHandler::new()
                .expect("Failed to create save/load handler"),
            event_manager: crate::game::EventManager::new(),
            travel_handler: crate::game::TravelHandler::new(system_manager_clone),
            trade_log: crate::game::TradeLog::new(),
        };

        // This test just ensures the function doesn't panic
        show_status(&game_state);
    }

    #[test]
    fn test_show_status_with_damage() {
        let commander = Commander::new("Test Commander".to_string());
        let mut ship = Ship::new_cobra_mk3();
        ship.take_damage(30); // Add some damage
        let system_manager =
            crate::data::systems::SystemManager::new().expect("Failed to load galaxy data");
        let system_manager_clone = system_manager.clone();
        let game_state = GameState {
            commander,
            ship,
            system_manager,
            trading_handler: crate::game::TradingHandler::new(),
            upgrade_handler: crate::game::UpgradeHandler::new(),
            save_load_handler: crate::game::SaveLoadHandler::new()
                .expect("Failed to create save/load handler"),
            event_manager: crate::game::EventManager::new(),
            travel_handler: crate::game::TravelHandler::new(system_manager_clone),
            trade_log: crate::game::TradeLog::new(),
        };

        // This test just ensures the function doesn't panic
        show_status(&game_state);
    }

    #[test]
    fn test_show_status_with_low_fuel() {
        let commander = Commander::new("Test Commander".to_string());
        let mut ship = Ship::new_cobra_mk3();
        ship.consume_fuel(5); // Consume most fuel
        let system_manager =
            crate::data::systems::SystemManager::new().expect("Failed to load galaxy_data");
        let system_manager_clone = system_manager.clone();
        let game_state = GameState {
            commander,
            ship,
            system_manager,
            trading_handler: crate::game::TradingHandler::new(),
            upgrade_handler: crate::game::UpgradeHandler::new(),
            save_load_handler: crate::game::SaveLoadHandler::new()
                .expect("Failed to create save/load handler"),
            event_manager: crate::game::EventManager::new(),
            travel_handler: crate::game::TravelHandler::new(system_manager_clone),
            trade_log: crate::game::TradeLog::new(),
        };

        // This test just ensures the function doesn't panic
        show_status(&game_state);
    }
}
