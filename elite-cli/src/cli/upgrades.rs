use crate::utils::display::{show_error, show_info, show_success};
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
    println!("\n{}", "Available Upgrades".cyan().bold());
    println!("{}", "==================".cyan());
    println!("Current Ship: {}", current_name.yellow());
    println!("Credits: {}cr", view.credits.to_string().green());
    println!();

    if view.options.is_empty() {
        println!("No upgrades available for your current ship.");
        return;
    }

    let current = &view.current;
    for option in &view.options {
        let upgrade = &option.ship;
        let cost_display = if option.affordable {
            format!("({}cr)", option.cost).green()
        } else {
            format!("({}cr) - Insufficient credits", option.cost).red()
        };

        println!("{} {}", upgrade.display_name().yellow(), cost_display);
        println!(
            "- Cargo: {} → {} tons ({:+})",
            current.cargo_capacity(),
            upgrade.cargo_capacity(),
            upgrade.cargo_capacity() as i32 - current.cargo_capacity() as i32
        );
        println!(
            "- Fuel: {} → {} light years ({:+})",
            current.fuel_capacity(),
            upgrade.fuel_capacity(),
            upgrade.fuel_capacity() as i32 - current.fuel_capacity() as i32
        );
        println!(
            "- Hull: {} → {} ({:+})",
            current.max_hull(),
            upgrade.max_hull(),
            upgrade.max_hull() as i32 - current.max_hull() as i32
        );
        println!();
    }
}

fn print_ships(view: &ShipList, current_name: &str) {
    println!("\n{}", "Available Ships".cyan().bold());
    println!("{}", "===============".cyan());
    println!("Current Ship: {}", current_name.yellow());
    println!("Credits: {}cr", view.credits.to_string().green());
    println!();

    for listing in &view.ships {
        let ship_type = &listing.ship;
        let ship_name = if listing.is_current {
            format!("{} (Current)", ship_type.display_name()).green()
        } else {
            ship_type.display_name().yellow()
        };

        println!("{}", ship_name);
        println!("- Cargo: {} tons", ship_type.cargo_capacity());
        println!("- Fuel: {} light years", ship_type.fuel_capacity());
        println!("- Hull: {}", ship_type.max_hull());
        println!("- Cost: {}cr", ship_type.cost());

        if let Some(cost) = listing.upgrade_cost {
            let cost_display = if listing.affordable {
                format!("Upgrade cost: {}cr", cost).green()
            } else {
                format!("Upgrade cost: {}cr (Insufficient credits)", cost).red()
            };
            println!("- {}", cost_display);
        }
        println!();
    }
}
