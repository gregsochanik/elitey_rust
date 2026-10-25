use crate::data::equipment::EquipmentCategory;
use crate::game::equipment::{self, InstalledView, ShopView};
use crate::game::{Commander, Ship};
use crate::utils::display::{show_error, show_success};
use colored::*;

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

fn item_from_args(args: &[String]) -> Result<&'static crate::data::equipment::Equipment, String> {
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

fn category_name(category: EquipmentCategory) -> ColoredString {
    category.name().yellow()
}

fn print_shop(view: &ShopView) {
    println!("\n{}", "Equipment Shop".cyan().bold());
    println!("{}", "==============".cyan());
    println!("Credits: {}cr", view.credits.to_string().green());

    for group in &view.categories {
        println!("\n{}", category_name(group.category));
        for entry in &group.items {
            let item = entry.item;
            println!(
                "  {:<18} {:<9} {:>5}cr  {:>2} power  {}{}",
                item.id,
                item.quality.name(),
                item.price,
                item.power,
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
    println!("\n{}", "Installed Equipment".cyan().bold());
    println!("{}", "===================".cyan());
    println!("Power: {}/{}", view.power_used, view.power_capacity);

    for group in &view.categories {
        println!(
            "\n{} ({}/{} slots)",
            category_name(group.category),
            group.slots_used,
            group.slots_capacity
        );
        for item in &group.items {
            println!(
                "  {:<18} {:>2} power  {}",
                item.name, item.power, item.effect
            );
        }
    }
    println!("\nCargo capacity: {} tons\n", view.cargo_capacity);
}
