use crate::utils::display::{self, credits, header, icon, show_error, show_success, Icon};
use colored::*;
use elite_core::data::equipment::Equipment;
use elite_core::game::equipment::{self, InstalledView, ShopView};
use elite_core::game::{Commander, Ship};

/// `equipment ...` on a game
pub fn command(game: &mut elite_core::game::GameState, args: &[String]) -> bool {
    handle(args, &mut game.commander, &mut game.ship)
}

/// `equipment [buy <item> | sell <item> | installed]`
pub fn handle(args: &[String], commander: &mut Commander, ship: &mut Ship) -> bool {
    match args.first().map(|a| a.to_lowercase()).as_deref() {
        None => {
            print_shop(&equipment::shop_view(commander, ship));
            true
        }
        Some("installed") => {
            print_installed(&equipment::installed_view(ship));
            true
        }
        Some("buy") => buy(&args[1..], commander, ship),
        Some("sell") => sell(&args[1..], commander, ship),
        Some(_) => {
            show_error("Usage: equipment [buy <item> | sell <item> | installed]");
            false
        }
    }
}

fn item_from_args(
    args: &[String],
) -> Result<&'static elite_core::data::equipment::Equipment, String> {
    if args.is_empty() {
        return Err("Specify an item, e.g. pulse-laser".to_string());
    }
    let id = args.join("-").to_lowercase();
    equipment::find_item(&id).map_err(|error| error.to_string())
}

fn buy(args: &[String], commander: &mut Commander, ship: &mut Ship) -> bool {
    let item = match item_from_args(args) {
        Ok(item) => item,
        Err(message) => {
            show_error(&message);
            return false;
        }
    };
    match equipment::buy(commander, ship, item) {
        Ok(purchase) => {
            match purchase.replaced {
                Some((old, refund)) => show_success(&format!(
                    "Installed {} for {}cr, replacing {} (refund {}cr)",
                    purchase.name, purchase.price, old, refund
                )),
                None => show_success(&format!(
                    "Installed {} for {}cr",
                    purchase.name, purchase.price
                )),
            }
            true
        }
        Err(error) => {
            show_error(&error.to_string());
            false
        }
    }
}

fn sell(args: &[String], commander: &mut Commander, ship: &mut Ship) -> bool {
    let item = match item_from_args(args) {
        Ok(item) => item,
        Err(message) => {
            show_error(&message);
            return false;
        }
    };
    match equipment::sell(commander, ship, item) {
        Ok(sale) => {
            show_success(&format!("Sold {} for {}cr", sale.name, sale.refund));
            true
        }
        Err(error) => {
            show_error(&error.to_string());
            false
        }
    }
}

fn power_column(item: &Equipment) -> String {
    if item.output > 0 {
        format!("gives {}", item.output)
    } else {
        format!("{:>2} power", item.power)
    }
}

fn power_line(used: u32, capacity: u32) -> String {
    format!(
        "Power: {}/{} used, {} free",
        used,
        capacity,
        capacity.saturating_sub(used)
    )
}

fn print_shop(view: &ShopView) {
    header(Icon::Equipment, "Equipment Shop");
    println!(
        "{}Credits: {}",
        icon(Icon::Credits),
        credits(i64::from(view.credits))
    );
    println!(
        "{}{}",
        icon(Icon::Power),
        power_line(view.power_used, view.power_capacity)
    );

    for group in &view.categories {
        println!("\n{}", display::category_name(group.category));
        for entry in &group.items {
            let item = entry.item;
            println!(
                "  {:<18} {:<9} {:>5}cr  {:<10}  {}{}",
                item.id,
                item.quality.name(),
                item.price,
                power_column(item),
                item.effect,
                if entry.installed {
                    "  [installed]".green().to_string()
                } else {
                    String::new()
                }
            );
        }
    }
    println!("\nUse: equipment buy <item>\n");
}

fn print_installed(view: &InstalledView) {
    header(Icon::Equipment, "Installed Equipment");
    println!(
        "{}{}",
        icon(Icon::Power),
        power_line(view.power_used, view.power_capacity)
    );

    for group in &view.categories {
        println!(
            "\n{} ({}/{} slots)",
            display::category_name(group.category),
            group.slots_used,
            group.slots_capacity
        );
        for item in &group.items {
            println!(
                "  {:<18} {:<10}  {}",
                item.name,
                power_column(item),
                item.effect
            );
        }
    }
    println!("\nCargo capacity: {} tons\n", view.cargo_capacity);
}
