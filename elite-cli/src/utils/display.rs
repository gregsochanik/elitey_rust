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
    Equipment,
    Save,
    Help,
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
            Icon::Equipment => "🧰",
            Icon::Save => "💾",
            Icon::Help => "❓",
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

/// A screen title with its icon, underlined to the width of the title
pub fn header(icon_kind: Icon, title: &str) {
    println!();
    let icon = icon(icon_kind);
    let width = title.chars().count() + if icon.is_empty() { 0 } else { 4 };
    println!("{}", format!("{}{}", icon, title).cyan().bold());
    println!("{}", "=".repeat(width).cyan());
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
    println!(
        "{}",
        format!("{}Welcome to Elite Rust!", icon(Icon::Ship))
            .green()
            .bold()
    );
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
    header(Icon::Help, "Available Commands");
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
    header(Icon::Market, "Trading Commands");
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
    header(Icon::Ship, "Ship Upgrade Commands");
    println!("{} - Show available ship upgrades", "upgrade".yellow());
    println!("{} - Purchase ship upgrade", "upgrade <ship>".yellow());
    println!("{} - List all available ships", "ships".yellow());
    header(Icon::Equipment, "Equipment Commands");
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
    header(Icon::Jobs, "Job Commands");
    println!("{} - Show the jobs on offer here", "jobs".yellow());
    println!("{} - Take a job from the board", "accept <number>".yellow());
    println!("{} - List your active jobs", "missions".yellow());
    println!(
        "{} - Drop an active job (-3 reputation)",
        "abandon <number>".yellow()
    );
    header(Icon::Repair, "Repair and Refuel Commands");
    println!(
        "{} - Repair hull damage (2 credits per point)",
        "repair [points]".yellow()
    );
    println!(
        "{} - Refuel (2 credits per light year)",
        "refuel [units]".yellow()
    );
    header(Icon::Save, "Save/Load Commands");
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
            Icon::Equipment,
            Icon::Save,
            Icon::Help,
        ];
        let glyphs: std::collections::HashSet<_> = icons.iter().map(Icon::glyph).collect();
        assert_eq!(glyphs.len(), icons.len());
    }

    #[test]
    fn colour_roles_use_distinct_colours() {
        control::set_override(true);
        let negative = credits(-5).to_string();
        let positive = credits(5).to_string();
        let colours: std::collections::HashSet<_> = EquipmentCategory::ALL
            .iter()
            .map(|c| category_name(*c).to_string().replace(c.name(), ""))
            .collect();
        control::unset_override();

        assert!(negative.contains("31m"));
        assert!(positive.contains("32m"));
        assert_eq!(colours.len(), EquipmentCategory::ALL.len());
    }

    #[test]
    fn test_show_help() {
        // This test just ensures the function doesn't panic
        show_help();
    }
}
