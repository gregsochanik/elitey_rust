use crate::utils::display::{header, show_error, show_info, show_success, Icon};
use crate::utils::table::{Table, LEFT, RIGHT};
use colored::*;
use elite_core::data::ships::ShipType;
use elite_core::game::upgrades::{ShipList, UpgradeError, UpgradeOptions};
use elite_core::game::GameState;

/// `upgrade`: show the ships that can be bought
pub fn show_upgrades(game: &GameState) {
    let view = game
        .upgrade_handler()
        .upgrade_options(&game.commander, &game.ship);
    print_upgrades(&view, game.ship.ship_type_name());
}

/// `ships`: list every ship
pub fn show_ships(game: &GameState) {
    let view = game
        .upgrade_handler()
        .ship_list(&game.commander, &game.ship);
    print_ships(&view, game.ship.ship_type_name());
}

/// `upgrade <ship>`
pub fn buy(game: &mut GameState, args: &[String]) -> bool {
    let name_words = match args.first().map(|word| word.to_lowercase()) {
        Some(word) if word == "buy" => &args[1..],
        _ => args,
    };
    if name_words.is_empty() {
        show_error("Usage: upgrade <ship>");
        return false;
    }
    let name = name_words.join(" ");
    let Some(ship_type) = ShipType::from_string(&name) else {
        show_error(&UpgradeError::UnknownShip(name).to_string());
        return false;
    };

    match game.buy_ship(ship_type) {
        Ok(receipt) => {
            if receipt.equipment_refund > 0 {
                show_info(&format!(
                    "Sold your equipment for {}cr",
                    receipt.equipment_refund
                ));
            }
            show_success(&format!(
                "Successfully upgraded to {} for {}cr!",
                receipt.ship.display_name(),
                receipt.cost
            ));
            true
        }
        Err(error) => {
            show_error(&error.to_string());
            false
        }
    }
}

fn print_upgrades(view: &UpgradeOptions, current_name: &str) {
    header(Icon::Ship, "Available Upgrades");
    println!("Current Ship: {}", current_name.yellow());
    println!("Credits: {}cr", view.credits.to_string().green());
    println!();

    if view.options.is_empty() {
        println!("No upgrades available for your current ship.");
        return;
    }

    let current = &view.current;
    let mut table = Table::new(&[
        ("Ship", LEFT),
        ("Cost", RIGHT),
        ("Cargo", RIGHT),
        ("Fuel", RIGHT),
        ("Hull", RIGHT),
        ("", LEFT),
    ]);
    for option in &view.options {
        let upgrade = &option.ship;
        let (cost, note) = if option.affordable {
            (
                format!("{}cr", option.cost).green().to_string(),
                String::new(),
            )
        } else {
            (
                format!("{}cr", option.cost).red().to_string(),
                "Insufficient credits".red().to_string(),
            )
        };
        table.row([
            upgrade.display_name().yellow().to_string(),
            cost,
            change(current.cargo_capacity(), upgrade.cargo_capacity()),
            change(current.fuel_capacity(), upgrade.fuel_capacity()),
            change(current.max_hull(), upgrade.max_hull()),
            note,
        ]);
    }
    table.print();
    println!();
}

fn change(from: u32, to: u32) -> String {
    format!("{} → {} ({:+})", from, to, i64::from(to) - i64::from(from))
}

fn print_ships(view: &ShipList, current_name: &str) {
    header(Icon::Ship, "Available Ships");
    println!("Current Ship: {}", current_name.yellow());
    println!("Credits: {}cr", view.credits.to_string().green());
    println!();

    let mut table = Table::new(&[
        ("Ship", LEFT),
        ("Cargo", RIGHT),
        ("Fuel", RIGHT),
        ("Hull", RIGHT),
        ("Cost", RIGHT),
        ("Upgrade cost", RIGHT),
    ]);
    for listing in &view.ships {
        let ship_type = &listing.ship;
        let ship_name = if listing.is_current {
            format!("{} (Current)", ship_type.display_name()).green()
        } else {
            ship_type.display_name().yellow()
        };
        let upgrade = match listing.upgrade_cost {
            Some(cost) if listing.affordable => format!("{}cr", cost).green().to_string(),
            Some(cost) => format!("{}cr (Insufficient credits)", cost)
                .red()
                .to_string(),
            None => String::new(),
        };
        table.row([
            ship_name.to_string(),
            ship_type.cargo_capacity().to_string(),
            ship_type.fuel_capacity().to_string(),
            ship_type.max_hull().to_string(),
            format!("{}cr", ship_type.cost()),
            upgrade,
        ]);
    }
    table.print();
    println!();
}
