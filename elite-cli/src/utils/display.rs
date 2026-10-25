use colored::*;
use elite_core::data::equipment::EquipmentCategory;

/// The emoji vocabulary shared by every screen
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    Credits,
    Ship,
    Fuel,
    Cargo,
    Hull,
    Power,
    Combat,
    Market,
    Repair,
    Jobs,
    Locked,
    Rating,
    Success,
    Error,
    Warning,
    Info,
}

impl Icon {
    pub fn glyph(&self) -> &'static str {
        match self {
            Icon::Credits => "💰",
            Icon::Ship => "🚀",
            Icon::Fuel => "⛽",
            Icon::Cargo => "📦",
            Icon::Hull => "🛡️",
            Icon::Power => "⚡",
            Icon::Combat => "⚔️",
            Icon::Market => "📊",
            Icon::Repair => "🔧",
            Icon::Jobs => "💼",
            Icon::Locked => "🔒",
            Icon::Rating => "⭐",
            Icon::Success => "✅",
            Icon::Error => "❌",
            Icon::Warning => "⚠️",
            Icon::Info => "ℹ️",
        }
    }
}

fn icon_prefix(icon: Icon, enabled: bool) -> String {
    if enabled {
        format!("{}  ", icon.glyph())
    } else {
        String::new()
    }
}

/// The icon and a gap, or nothing when colour is off (NO_COLOR or not a terminal)
pub fn icon(icon: Icon) -> String {
    icon_prefix(icon, control::SHOULD_COLORIZE.should_colorize())
}

/// Credits in green, or red when negative
pub fn credits(amount: i64) -> ColoredString {
    let text = format!("{}cr", amount);
    if amount < 0 {
        text.red()
    } else {
        text.green()
    }
}

/// The system the commander is in is blue; destinations are cyan
pub fn system_name(name: &str, current: bool) -> ColoredString {
    if current {
        name.blue()
    } else {
        name.cyan()
    }
}

/// Each equipment category has its own colour
pub fn category_name(category: EquipmentCategory) -> ColoredString {
    let name = category.name();
    match category {
        EquipmentCategory::Power => name.bright_yellow(),
        EquipmentCategory::Weapon => name.red(),
        EquipmentCategory::Shield => name.blue(),
        EquipmentCategory::Engine => name.yellow(),
        EquipmentCategory::Sensor => name.cyan(),
        EquipmentCategory::Cargo => name.green(),
        EquipmentCategory::Special => name.magenta(),
    }
}

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
    println!(
        "{}",
        format!("{}Error: {}", icon(Icon::Error), message).red()
    );
}

/// Show success message
pub fn show_success(message: &str) {
    println!(
        "{}",
        format!("{}Success: {}", icon(Icon::Success), message).green()
    );
}

/// Show info message
pub fn show_info(message: &str) {
    println!(
        "{}",
        format!("{}Info: {}", icon(Icon::Info), message).blue()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icons_are_dropped_when_colour_is_off() {
        assert_eq!(icon_prefix(Icon::Success, false), "");
        assert_eq!(icon_prefix(Icon::Success, true), "✅  ");
    }

    #[test]
    fn every_icon_has_its_own_glyph() {
        let icons = [
            Icon::Credits,
            Icon::Ship,
            Icon::Fuel,
            Icon::Cargo,
            Icon::Hull,
            Icon::Power,
            Icon::Combat,
            Icon::Market,
            Icon::Repair,
            Icon::Jobs,
            Icon::Locked,
            Icon::Rating,
            Icon::Success,
            Icon::Error,
            Icon::Warning,
            Icon::Info,
        ];
        let glyphs: std::collections::HashSet<_> = icons.iter().map(Icon::glyph).collect();
        assert_eq!(glyphs.len(), icons.len());
    }

    #[test]
    fn negative_credits_are_red_and_the_rest_green() {
        control::set_override(true);
        assert!(credits(-5).to_string().contains("31m"));
        assert!(credits(5).to_string().contains("32m"));
        control::unset_override();
    }

    #[test]
    fn every_equipment_category_has_a_distinct_colour() {
        control::set_override(true);
        let colours: std::collections::HashSet<_> = EquipmentCategory::ALL
            .iter()
            .map(|c| category_name(*c).to_string().replace(c.name(), ""))
            .collect();
        control::unset_override();
        assert_eq!(colours.len(), EquipmentCategory::ALL.len());
    }

    #[test]
    fn test_show_help() {
        // This test just ensures the function doesn't panic
        show_help();
    }
}
