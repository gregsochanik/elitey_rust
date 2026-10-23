use crate::data::equipment::{self, Equipment, EquipmentCategory};
use crate::game::{Commander, Ship};
use crate::utils::display::{show_error, show_success};
use colored::*;
use serde::{Deserialize, Serialize};

const POWER_CAPACITY: u32 = 20;

fn slot_capacity(category: EquipmentCategory) -> usize {
    match category {
        EquipmentCategory::Weapon | EquipmentCategory::Special => 2,
        _ => 1,
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Installed {
    pub id: String,
    /// Credits paid; the starting equipment cost nothing
    pub paid: u32,
}

impl Installed {
    fn item(&self) -> &'static Equipment {
        equipment::find(&self.id).expect("installed equipment must exist in the catalog")
    }

    pub fn refund(&self) -> u32 {
        self.paid / 2
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Loadout {
    installed: Vec<Installed>,
}

impl Loadout {
    pub fn starting() -> Self {
        let free = |id: &str| Installed {
            id: id.to_string(),
            paid: 0,
        };
        Self {
            installed: vec![
                free("pulse-laser"),
                free("basic-shield"),
                free("standard-drive"),
                free("basic-scanner"),
                free("standard-racks"),
                free("heat-sink"),
            ],
        }
    }

    pub fn installed(&self) -> &[Installed] {
        &self.installed
    }

    pub fn power_used(&self) -> u32 {
        self.installed.iter().map(|i| i.item().power).sum()
    }

    pub fn power_capacity(&self) -> u32 {
        POWER_CAPACITY
    }

    pub fn slots_used(&self, category: EquipmentCategory) -> usize {
        self.installed
            .iter()
            .filter(|i| i.item().category == category)
            .count()
    }

    pub fn slots_capacity(&self, category: EquipmentCategory) -> usize {
        slot_capacity(category)
    }

    pub fn cargo_multiplier(&self) -> f64 {
        self.installed
            .iter()
            .map(|i| i.item().cargo_multiplier)
            .fold(1.0, f64::max)
    }

    pub fn total_refund(&self) -> u32 {
        self.installed.iter().map(Installed::refund).sum()
    }

    /// Install an item. A full single-slot category swaps out the current item, which is returned.
    pub fn install(
        &mut self,
        item: &'static Equipment,
        paid: u32,
    ) -> Result<Option<Installed>, String> {
        let full = self.slots_used(item.category) >= slot_capacity(item.category);
        let replaced_index = if full {
            if slot_capacity(item.category) > 1 {
                return Err(format!(
                    "No free {} slot",
                    item.category.name().to_lowercase()
                ));
            }
            self.installed
                .iter()
                .position(|i| i.item().category == item.category)
        } else {
            None
        };

        let replaced_power = replaced_index.map_or(0, |index| self.installed[index].item().power);
        if self.power_used() - replaced_power + item.power > POWER_CAPACITY {
            return Err(format!(
                "Not enough power: need {}, {} available",
                item.power,
                POWER_CAPACITY - self.power_used() + replaced_power
            ));
        }

        let replaced = replaced_index.map(|index| self.installed.remove(index));
        self.installed.push(Installed {
            id: item.id.to_string(),
            paid,
        });
        Ok(replaced)
    }

    pub fn remove(&mut self, id: &str) -> Option<Installed> {
        let index = self.installed.iter().position(|i| i.id == id)?;
        Some(self.installed.remove(index))
    }
}

impl Default for Loadout {
    fn default() -> Self {
        Self::starting()
    }
}

pub struct EquipmentHandler;

impl EquipmentHandler {
    pub fn handle(args: &[String], commander: &mut Commander, ship: &mut Ship) -> bool {
        match args.first().map(|a| a.to_lowercase()).as_deref() {
            None => {
                Self::display_shop(commander, ship);
                true
            }
            Some("installed") => {
                Self::display_installed(ship);
                true
            }
            Some("buy") => Self::buy(&args[1..], commander, ship),
            Some("sell") => Self::sell(&args[1..], commander, ship),
            Some(_) => {
                show_error("Usage: equipment [buy <item> | sell <item> | installed]");
                false
            }
        }
    }

    fn item_from_args(args: &[String]) -> Result<&'static Equipment, String> {
        if args.is_empty() {
            return Err("Specify an item, e.g. pulse-laser".to_string());
        }
        let id = args.join("-").to_lowercase();
        equipment::find(&id).ok_or_else(|| format!("Unknown equipment: {}", id))
    }

    fn buy(args: &[String], commander: &mut Commander, ship: &mut Ship) -> bool {
        let item = match Self::item_from_args(args) {
            Ok(item) => item,
            Err(message) => {
                show_error(&message);
                return false;
            }
        };

        if ship.equipment.installed().iter().any(|i| i.id == item.id) {
            show_error(&format!("{} is already installed", item.name));
            return false;
        }

        let mut trial = ship.equipment.clone();
        let replaced = match trial.install(item, item.price) {
            Ok(replaced) => replaced,
            Err(message) => {
                show_error(&message);
                return false;
            }
        };

        let refund = replaced.as_ref().map_or(0, Installed::refund);
        if commander.credits() + refund < item.price {
            show_error(&format!(
                "Insufficient credits. Need {}cr, have {}cr",
                item.price.saturating_sub(refund),
                commander.credits()
            ));
            return false;
        }

        if let Err(message) = ship.set_equipment(trial) {
            show_error(&message);
            return false;
        }
        commander.add_credits(refund);
        commander.remove_credits(item.price);

        match replaced {
            Some(old) => show_success(&format!(
                "Installed {} for {}cr, replacing {} (refund {}cr)",
                item.name,
                item.price,
                old.item().name,
                refund
            )),
            None => show_success(&format!("Installed {} for {}cr", item.name, item.price)),
        }
        true
    }

    fn sell(args: &[String], commander: &mut Commander, ship: &mut Ship) -> bool {
        let item = match Self::item_from_args(args) {
            Ok(item) => item,
            Err(message) => {
                show_error(&message);
                return false;
            }
        };

        let mut trial = ship.equipment.clone();
        let removed = match trial.remove(item.id) {
            Some(removed) => removed,
            None => {
                show_error(&format!("{} is not installed", item.name));
                return false;
            }
        };

        if let Err(message) = ship.set_equipment(trial) {
            show_error(&message);
            return false;
        }
        commander.add_credits(removed.refund());
        show_success(&format!("Sold {} for {}cr", item.name, removed.refund()));
        true
    }

    fn display_shop(commander: &Commander, ship: &Ship) {
        println!("\n{}", "Equipment Shop".cyan().bold());
        println!("{}", "==============".cyan());
        println!("Credits: {}cr", commander.credits().to_string().green());

        for category in EquipmentCategory::ALL {
            println!("\n{}", category.name().yellow());
            for item in equipment::catalog_for(category) {
                let installed = ship.equipment.installed().iter().any(|i| i.id == item.id);
                println!(
                    "  {:<18} {:<9} {:>5}cr  {:>2} power  {}{}",
                    item.id,
                    item.quality.name(),
                    item.price,
                    item.power,
                    item.effect,
                    if installed {
                        "  [installed]".green().to_string()
                    } else {
                        String::new()
                    }
                );
            }
        }
        println!("\nUse: equipment buy <item>\n");
    }

    fn display_installed(ship: &Ship) {
        let loadout = &ship.equipment;
        println!("\n{}", "Installed Equipment".cyan().bold());
        println!("{}", "===================".cyan());
        println!(
            "Power: {}/{}",
            loadout.power_used(),
            loadout.power_capacity()
        );

        for category in EquipmentCategory::ALL {
            println!(
                "\n{} ({}/{} slots)",
                category.name().yellow(),
                loadout.slots_used(category),
                loadout.slots_capacity(category)
            );
            for installed in loadout
                .installed()
                .iter()
                .filter(|i| i.item().category == category)
            {
                let item = installed.item();
                println!(
                    "  {:<18} {:>2} power  {}",
                    item.name, item.power, item.effect
                );
            }
        }
        println!("\nCargo capacity: {} tons\n", ship.cargo_capacity());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(text: &str) -> Vec<String> {
        text.split_whitespace().map(String::from).collect()
    }

    fn setup(credits: u32) -> (Commander, Ship) {
        let mut commander = Commander::new("Test".to_string());
        commander.credits = credits;
        (commander, Ship::new_cobra_mk3())
    }

    #[test]
    fn new_ship_starts_with_basic_equipment_and_unchanged_capacity() {
        let ship = Ship::new_cobra_mk3();
        assert_eq!(ship.equipment.installed().len(), 6);
        assert_eq!(ship.equipment.power_used(), 9);
        assert_eq!(ship.cargo_capacity(), 20);
    }

    fn buy(text: &str, commander: &mut Commander, ship: &mut Ship) -> bool {
        EquipmentHandler::handle(&args(&format!("buy {}", text)), commander, ship)
    }

    fn sell(text: &str, commander: &mut Commander, ship: &mut Ship) -> bool {
        EquipmentHandler::handle(&args(&format!("sell {}", text)), commander, ship)
    }

    fn has(ship: &Ship, id: &str) -> bool {
        ship.equipment.installed().iter().any(|i| i.id == id)
    }

    #[test]
    fn buying_installs_the_item_and_deducts_credits() {
        let (mut commander, mut ship) = setup(500);
        assert!(buy("beam-laser", &mut commander, &mut ship));

        assert_eq!(commander.credits(), 380);
        assert!(has(&ship, "beam-laser"));
        assert_eq!(ship.equipment.power_used(), 13);
    }

    #[test]
    fn item_names_can_be_typed_with_spaces_and_any_case() {
        let (mut commander, mut ship) = setup(500);
        assert!(buy("Beam Laser", &mut commander, &mut ship));
        assert!(has(&ship, "beam-laser"));
    }

    #[test]
    fn buying_without_enough_credits_changes_nothing() {
        let (mut commander, mut ship) = setup(100);
        assert!(!buy("beam-laser", &mut commander, &mut ship));

        assert_eq!(commander.credits(), 100);
        assert!(!has(&ship, "beam-laser"));
    }

    #[test]
    fn buying_into_a_full_multi_slot_category_is_refused() {
        let (mut commander, mut ship) = setup(1_000);
        assert!(buy("beam-laser", &mut commander, &mut ship));
        let credits = commander.credits();

        assert!(!buy("plasma-cannon", &mut commander, &mut ship));
        assert_eq!(commander.credits(), credits);
        assert!(!has(&ship, "plasma-cannon"));
    }

    #[test]
    fn buying_beyond_power_capacity_is_refused() {
        let (mut commander, mut ship) = setup(2_000);
        assert!(buy("military-shield", &mut commander, &mut ship)); // 13
        assert!(buy("plasma-cannon", &mut commander, &mut ship)); // 19
        let credits = commander.credits();

        assert!(!buy("ecm-system", &mut commander, &mut ship)); // 23 > 20
        assert_eq!(commander.credits(), credits);
        assert_eq!(ship.equipment.power_used(), 19);
    }

    #[test]
    fn a_better_single_slot_item_replaces_the_current_one_with_a_refund() {
        let (mut commander, mut ship) = setup(500);
        assert!(buy("expanded-racks", &mut commander, &mut ship));
        assert_eq!(commander.credits(), 470);
        assert_eq!(ship.cargo_capacity(), 25);

        assert!(buy("military-racks", &mut commander, &mut ship));
        assert_eq!(commander.credits(), 470 - 80 + 15);
        assert_eq!(ship.cargo_capacity(), 30);
        assert!(!has(&ship, "expanded-racks"));
    }

    #[test]
    fn selling_refunds_half_and_frees_the_slot() {
        let (mut commander, mut ship) = setup(500);
        buy("beam-laser", &mut commander, &mut ship);

        assert!(sell("beam-laser", &mut commander, &mut ship));
        assert_eq!(commander.credits(), 500 - 120 + 60);
        assert_eq!(ship.equipment.slots_used(EquipmentCategory::Weapon), 1);
    }

    #[test]
    fn starting_equipment_sells_for_nothing() {
        let (mut commander, mut ship) = setup(100);
        assert!(sell("pulse-laser", &mut commander, &mut ship));
        assert_eq!(commander.credits(), 100);
    }

    #[test]
    fn selling_racks_is_refused_when_cargo_would_not_fit() {
        let (mut commander, mut ship) = setup(500);
        buy("expanded-racks", &mut commander, &mut ship);
        ship.add_cargo(crate::data::commodities::CommodityType::Food, 22);
        let credits = commander.credits();

        assert!(!sell("expanded-racks", &mut commander, &mut ship));
        assert!(has(&ship, "expanded-racks"));
        assert_eq!(ship.cargo_capacity(), 25);
        assert_eq!(commander.credits(), credits);
    }

    #[test]
    fn unknown_missing_and_duplicate_items_are_refused() {
        let (mut commander, mut ship) = setup(500);
        assert!(!buy("nonsense", &mut commander, &mut ship));
        assert!(!buy("", &mut commander, &mut ship));
        assert!(!buy("pulse-laser", &mut commander, &mut ship));
        assert!(!sell("beam-laser", &mut commander, &mut ship));
        assert_eq!(commander.credits(), 500);
    }

    #[test]
    fn equipment_survives_serialization() {
        let (mut commander, mut ship) = setup(500);
        buy("expanded-racks", &mut commander, &mut ship);

        let restored: Ship = serde_json::from_str(&serde_json::to_string(&ship).unwrap()).unwrap();
        assert!(has(&restored, "expanded-racks"));
        assert_eq!(restored.cargo_capacity(), 25);
    }

    #[test]
    fn ship_saved_before_equipment_loads_with_starting_equipment() {
        let ship = Ship::new_cobra_mk3();
        let mut value = serde_json::to_value(&ship).unwrap();
        value.as_object_mut().unwrap().remove("equipment");

        let restored: Ship = serde_json::from_value(value).unwrap();
        assert_eq!(restored.equipment, Loadout::starting());
    }
}
