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
    println!("{}", "Job Commands:".cyan().bold());
    println!("{}", "=============".cyan());
    println!("{} - Show the jobs on offer here", "jobs".yellow());
    println!("{} - Take a job from the board", "accept <number>".yellow());
    println!("{} - List your active jobs", "missions".yellow());
    println!(
        "{} - Drop an active job (-3 reputation)",
        "abandon <number>".yellow()
    );
    println!();
    println!("{}", "Repair and Refuel Commands:".cyan().bold());
    println!("{}", "============================".cyan());
    println!(
        "{} - Repair hull damage (2 credits per point)",
        "repair [points]".yellow()
    );
    println!(
        "{} - Refuel (2 credits per light year)",
        "refuel [units]".yellow()
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

    #[test]
    fn test_show_help() {
        // This test just ensures the function doesn't panic
        show_help();
    }
}
