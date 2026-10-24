use crate::data::ships::{ShipType, ShipUpgradeManager};
use crate::game::{Commander, Ship};
use crate::utils::display::{show_error, show_info, show_success};
use colored::*;

/// Upgrade command handler
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UpgradeHandler {
    upgrade_manager: ShipUpgradeManager,
}

impl UpgradeHandler {
    pub fn new() -> Self {
        Self {
            upgrade_manager: ShipUpgradeManager::new(),
        }
    }

    /// Handle the upgrade command - show available ship upgrades
    pub fn handle_upgrade(&self, commander: &Commander, ship: &Ship) {
        self.display_available_upgrades(commander, ship);
    }

    /// Handle the upgrade purchase command
    pub fn handle_upgrade_purchase(
        &self,
        args: &[String],
        commander: &mut Commander,
        ship: &mut Ship,
    ) -> bool {
        let name_words = match args.first().map(|word| word.to_lowercase()) {
            Some(word) if word == "buy" => &args[1..],
            _ => args,
        };
        if name_words.is_empty() {
            show_error("Usage: upgrade <ship>");
            return false;
        }
        let ship_name = &name_words.join(" ");

        // Parse ship type
        let new_ship_type = match ShipType::from_string(ship_name) {
            Some(ship_type) => ship_type,
            None => {
                show_error(&format!("Unknown ship type: {}", ship_name));
                return false;
            }
        };

        // Check if it's actually an upgrade
        if !new_ship_type.is_upgrade_from(ship.ship_type()) {
            show_error(&format!(
                "{} is not an upgrade from your current ship",
                new_ship_type.display_name()
            ));
            return false;
        }

        // Calculate upgrade cost
        let upgrade_cost = self
            .upgrade_manager
            .calculate_upgrade_cost(ship.ship_type(), &new_ship_type);

        // Check if commander has enough credits
        if !commander.has_credits(upgrade_cost) {
            show_error(&format!(
                "Insufficient credits. Need {}cr, have {}cr",
                upgrade_cost,
                commander.credits()
            ));
            return false;
        }

        // Perform the upgrade; installed equipment is sold at half price
        let equipment_refund = ship.equipment.total_refund();
        match ship.upgrade_to(new_ship_type) {
            Ok(_) => {
                commander.remove_credits(upgrade_cost);
                commander.add_credits(equipment_refund);
                if equipment_refund > 0 {
                    show_info(&format!("Sold your equipment for {}cr", equipment_refund));
                }
                show_success(&format!(
                    "Successfully upgraded to {} for {}cr!",
                    new_ship_type.display_name(),
                    upgrade_cost
                ));
                true
            }
            Err(e) => {
                show_error(&format!("Upgrade failed: {}", e));
                false
            }
        }
    }

    /// Handle the ships command - list all available ships
    pub fn handle_ships(&self, commander: &Commander, ship: &Ship) {
        self.display_all_ships(commander, ship);
    }

    /// Display available upgrades
    fn display_available_upgrades(&self, commander: &Commander, ship: &Ship) {
        println!("\n{}", "Available Upgrades".cyan().bold());
        println!("{}", "==================".cyan());
        println!("Current Ship: {}", ship.ship_type_name().yellow());
        println!("Credits: {}cr", commander.credits().to_string().green());
        println!();

        let upgrades = self
            .upgrade_manager
            .get_available_upgrades(ship.ship_type());

        if upgrades.is_empty() {
            println!("No upgrades available for your current ship.");
            return;
        }

        for upgrade_ship in upgrades {
            let upgrade_cost = self
                .upgrade_manager
                .calculate_upgrade_cost(ship.ship_type(), upgrade_ship);
            let can_afford = commander.has_credits(upgrade_cost);

            let cost_display = if can_afford {
                format!("({}cr)", upgrade_cost).green()
            } else {
                format!("({}cr) - Insufficient credits", upgrade_cost).red()
            };

            println!("{} {}", upgrade_ship.display_name().yellow(), cost_display);
            println!(
                "- Cargo: {} → {} tons ({:+})",
                ship.cargo_capacity(),
                upgrade_ship.cargo_capacity(),
                upgrade_ship.cargo_capacity() as i32 - ship.cargo_capacity() as i32
            );
            println!(
                "- Fuel: {} → {} light years ({:+})",
                ship.fuel_capacity(),
                upgrade_ship.fuel_capacity(),
                upgrade_ship.fuel_capacity() as i32 - ship.fuel_capacity() as i32
            );
            println!(
                "- Hull: {} → {} ({:+})",
                ship.ship_type.max_hull(),
                upgrade_ship.max_hull(),
                upgrade_ship.max_hull() as i32 - ship.ship_type.max_hull() as i32
            );
            println!();
        }
    }

    /// Display all available ships
    fn display_all_ships(&self, commander: &Commander, ship: &Ship) {
        println!("\n{}", "Available Ships".cyan().bold());
        println!("{}", "===============".cyan());
        println!("Current Ship: {}", ship.ship_type_name().yellow());
        println!("Credits: {}cr", commander.credits().to_string().green());
        println!();

        for ship_type in self.upgrade_manager.get_available_ships() {
            let is_current = ship_type == ship.ship_type();
            let ship_name = if is_current {
                format!("{} (Current)", ship_type.display_name()).green()
            } else {
                ship_type.display_name().yellow()
            };

            println!("{}", ship_name);
            println!("- Cargo: {} tons", ship_type.cargo_capacity());
            println!("- Fuel: {} light years", ship_type.fuel_capacity());
            println!("- Hull: {}", ship_type.max_hull());
            println!("- Cost: {}cr", ship_type.cost());

            if !is_current {
                let upgrade_cost = self
                    .upgrade_manager
                    .calculate_upgrade_cost(ship.ship_type(), ship_type);
                if upgrade_cost > 0 {
                    let can_afford = commander.has_credits(upgrade_cost);
                    let cost_display = if can_afford {
                        format!("Upgrade cost: {}cr", upgrade_cost).green()
                    } else {
                        format!("Upgrade cost: {}cr (Insufficient credits)", upgrade_cost).red()
                    };
                    println!("- {}", cost_display);
                }
            }
            println!();
        }
    }
}

impl Default for UpgradeHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upgrade_handler_creation() {
        let handler = UpgradeHandler::new();
        assert_eq!(
            handler
                .upgrade_manager
                .get_available_upgrades(&ShipType::starting())
                .len(),
            3
        );
    }

    #[test]
    fn changing_ship_sells_equipment_at_half_price_and_resets_loadout() {
        let handler = UpgradeHandler::new();
        let mut commander = Commander::new("Test".to_string());
        commander.credits = 5_000;
        let mut ship = Ship::new_cobra_mk3();
        let mut loadout = ship.equipment.clone();
        loadout
            .install(crate::data::equipment::find("expanded-racks").unwrap(), 30)
            .unwrap();
        ship.set_equipment(loadout).unwrap();

        let cost = handler.upgrade_manager.calculate_upgrade_cost(
            &ShipType::starting(),
            &ShipType::from_string("cobra4").unwrap(),
        );
        let args = vec!["cobra4".to_string()];
        assert!(handler.handle_upgrade_purchase(&args, &mut commander, &mut ship));

        assert_eq!(commander.credits(), 5_000 - cost + 15);
        assert_eq!(ship.equipment, crate::game::equipment::Loadout::starting());
        assert_eq!(ship.cargo_capacity(), 35);
    }
}
