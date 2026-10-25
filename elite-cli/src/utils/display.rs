use colored::*;
use elite_core::data::equipment::EquipmentCategory;
use elite_core::data::systems::{EconomyType, GovernmentType};

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
    Explore,
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
            Icon::Explore => "🧭",
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

/// The sector an economy works in
pub fn economy_icon(economy: &EconomyType) -> String {
    let glyph = match economy {
        EconomyType::RichIndustrial
        | EconomyType::PoorIndustrial
        | EconomyType::MainlyIndustrial
        | EconomyType::AverageIndustrial => "🏭",
        EconomyType::RichAgricultural
        | EconomyType::PoorAgricultural
        | EconomyType::MainlyAgricultural
        | EconomyType::AverageAgricultural => "🌾",
    };
    glyph_prefix(glyph, control::SHOULD_COLORIZE.should_colorize())
}

pub fn government_icon(government: &GovernmentType) -> String {
    let glyph = match government {
        GovernmentType::Anarchy => "💥",
        GovernmentType::Feudal => "🏰",
        GovernmentType::MultiGovernment => "🤝",
        GovernmentType::Dictatorship => "👑",
        GovernmentType::Communist => "🚩",
        GovernmentType::Confederacy => "🏛️",
        GovernmentType::Democracy => "🗳️",
        GovernmentType::CorporateState => "🏢",
    };
    glyph_prefix(glyph, control::SHOULD_COLORIZE.should_colorize())
}

fn glyph_prefix(glyph: &str, enabled: bool) -> String {
    if enabled {
        format!("{}  ", glyph)
    } else {
        String::new()
    }
}

const BAR_WIDTH: usize = 10;

/// Whether a full bar is healthy (fuel, hull) or a warning (cargo, power in use)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fill {
    FullIsGood,
    FullIsWarning,
}

fn filled_cells(current: u32, max: u32) -> usize {
    if max == 0 {
        return 0;
    }
    let ratio = f64::from(current.min(max)) / f64::from(max);
    (ratio * BAR_WIDTH as f64).round() as usize
}

fn bar_text(current: u32, max: u32, unicode: bool) -> String {
    let filled = filled_cells(current, max);
    let (on, off) = if unicode { ('█', '░') } else { ('#', '-') };
    format!(
        "[{}{}]",
        on.to_string().repeat(filled),
        off.to_string().repeat(BAR_WIDTH - filled)
    )
}

fn bar_colour(current: u32, max: u32, fill: Fill) -> Color {
    let percent = if max == 0 {
        0
    } else {
        u64::from(current.min(max)) * 100 / u64::from(max)
    };
    let health = match fill {
        Fill::FullIsGood => percent,
        Fill::FullIsWarning => 100 - percent,
    };
    match health {
        60..=100 => Color::Green,
        30..=59 => Color::Yellow,
        _ => Color::Red,
    }
}

/// A `[████░░░░░░]` gauge coloured by how healthy the level is; `[####------]` without colour
pub fn bar(current: u32, max: u32, fill: Fill) -> ColoredString {
    let unicode = control::SHOULD_COLORIZE.should_colorize();
    bar_text(current, max, unicode).color(bar_colour(current, max, fill))
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
    fn economy_and_government_icons_are_dropped_with_colour_off() {
        assert_eq!(glyph_prefix("🏭", false), "");
        assert_eq!(glyph_prefix("🏭", true), "🏭  ");
    }

    #[test]
    fn bars_fill_in_proportion() {
        assert_eq!(bar_text(0, 10, true), "[░░░░░░░░░░]");
        assert_eq!(bar_text(8, 10, true), "[████████░░]");
        assert_eq!(bar_text(10, 10, true), "[██████████]");
        assert_eq!(bar_text(5, 20, false), "[###-------]");
    }

    #[test]
    fn bars_stay_within_range() {
        assert_eq!(bar_text(30, 10, true), "[██████████]");
        assert_eq!(bar_text(3, 0, true), "[░░░░░░░░░░]");
    }

    #[test]
    fn bar_colour_follows_health() {
        assert_eq!(bar_colour(8, 10, Fill::FullIsGood), Color::Green);
        assert_eq!(bar_colour(4, 10, Fill::FullIsGood), Color::Yellow);
        assert_eq!(bar_colour(1, 10, Fill::FullIsGood), Color::Red);
        assert_eq!(bar_colour(1, 10, Fill::FullIsWarning), Color::Green);
        assert_eq!(bar_colour(9, 10, Fill::FullIsWarning), Color::Red);
    }
}
