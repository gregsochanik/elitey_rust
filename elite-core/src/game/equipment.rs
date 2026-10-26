use crate::data::equipment::{self, Equipment, EquipmentCategory};
use crate::game::{Commander, Ship};
use serde::{Deserialize, Serialize};
use thiserror::Error;

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

const STARTING_REACTOR: &str = "basic-reactor";

impl Installed {
    fn free(id: &str) -> Self {
        Self {
            id: id.to_string(),
            paid: 0,
        }
    }

    pub fn item(&self) -> &'static Equipment {
        equipment::find(&self.id).expect("installed equipment must exist in the catalog")
    }

    pub fn refund(&self) -> u32 {
        self.paid / 2
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(from = "SavedLoadout")]
pub struct Loadout {
    installed: Vec<Installed>,
}

#[derive(Deserialize)]
struct SavedLoadout {
    installed: Vec<Installed>,
}

impl From<SavedLoadout> for Loadout {
    fn from(saved: SavedLoadout) -> Self {
        let mut installed = saved.installed;
        if !installed
            .iter()
            .any(|i| i.item().category == EquipmentCategory::Power)
        {
            installed.insert(0, Installed::free(STARTING_REACTOR));
        }
        Self { installed }
    }
}

impl Loadout {
    pub fn starting() -> Self {
        let free = Installed::free;
        Self {
            installed: vec![
                free(STARTING_REACTOR),
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
        self.installed.iter().map(|i| i.item().output).sum()
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

        let replaced_item = replaced_index.map(|index| self.installed[index].item());
        let freed_power = replaced_item.map_or(0, |old| old.power);
        let lost_output = replaced_item.map_or(0, |old| old.output);
        let used = self.power_used() - freed_power + item.power;
        let capacity = self.power_capacity() - lost_output + item.output;
        if used > capacity {
            return Err(format!(
                "Not enough power: need {}, {} available",
                used, capacity
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

/// Why an equipment purchase or sale was refused
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum EquipmentError {
    #[error("Unknown equipment: {0}")]
    Unknown(String),
    #[error("{0} is already installed")]
    AlreadyInstalled(String),
    #[error("{0} is not installed")]
    NotInstalled(String),
    #[error("Insufficient credits. Need {needed}cr, have {have}cr")]
    NotEnoughCredits { needed: u32, have: u32 },
    #[error("{name} needs tech level {needed} (this system is {have})")]
    TechTooLow { name: String, needed: u8, have: u8 },
    /// No free slot, not enough power, or the ship could not take the change
    #[error("{0}")]
    CannotFit(String),
}

/// What buying equipment did
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EquipmentPurchase {
    pub name: String,
    pub price: u32,
    /// The item swapped out and what it refunded
    pub replaced: Option<(String, u32)>,
}

/// What selling equipment did
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EquipmentSale {
    pub name: String,
    pub refund: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShopItem {
    pub item: &'static Equipment,
    pub installed: bool,
    /// The system's tech level is below the item's
    pub locked: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShopCategory {
    pub category: EquipmentCategory,
    pub items: Vec<ShopItem>,
}

/// Everything for sale, by category
#[derive(Debug, Clone, PartialEq)]
pub struct ShopView {
    pub credits: u32,
    pub power_used: u32,
    pub power_capacity: u32,
    pub categories: Vec<ShopCategory>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InstalledCategory {
    pub category: EquipmentCategory,
    pub slots_used: usize,
    pub slots_capacity: usize,
    pub items: Vec<&'static Equipment>,
}

/// The ship's loadout
#[derive(Debug, Clone, PartialEq)]
pub struct InstalledView {
    pub power_used: u32,
    pub power_capacity: u32,
    pub categories: Vec<InstalledCategory>,
    pub cargo_capacity: u32,
}

/// Look an item up by catalog id
pub fn find_item(id: &str) -> Result<&'static Equipment, EquipmentError> {
    equipment::find(id).ok_or_else(|| EquipmentError::Unknown(id.to_string()))
}

/// Buy and install an item; a full single-slot category swaps out the current item for a refund
pub fn buy(
    commander: &mut Commander,
    ship: &mut Ship,
    item: &'static Equipment,
) -> Result<EquipmentPurchase, EquipmentError> {
    if ship.equipment.installed().iter().any(|i| i.id == item.id) {
        return Err(EquipmentError::AlreadyInstalled(item.name.clone()));
    }

    let mut trial = ship.equipment.clone();
    let replaced = trial
        .install(item, item.price)
        .map_err(EquipmentError::CannotFit)?;

    let refund = replaced.as_ref().map_or(0, Installed::refund);
    if commander.credits() + refund < item.price {
        return Err(EquipmentError::NotEnoughCredits {
            needed: item.price.saturating_sub(refund),
            have: commander.credits(),
        });
    }

    ship.set_equipment(trial)
        .map_err(EquipmentError::CannotFit)?;
    commander.add_credits(refund);
    commander.remove_credits(item.price);

    Ok(EquipmentPurchase {
        name: item.name.clone(),
        price: item.price,
        replaced: replaced.map(|old| (old.item().name.clone(), refund)),
    })
}

/// Sell an installed item for half what was paid
pub fn sell(
    commander: &mut Commander,
    ship: &mut Ship,
    item: &'static Equipment,
) -> Result<EquipmentSale, EquipmentError> {
    let mut trial = ship.equipment.clone();
    let removed = trial
        .remove(&item.id)
        .ok_or_else(|| EquipmentError::NotInstalled(item.name.clone()))?;

    ship.set_equipment(trial)
        .map_err(EquipmentError::CannotFit)?;
    commander.add_credits(removed.refund());
    Ok(EquipmentSale {
        name: item.name.clone(),
        refund: removed.refund(),
    })
}

/// Whether a system of this tech level sells the item
pub fn check_tech(item: &Equipment, tech_level: u8) -> Result<(), EquipmentError> {
    if tech_level < item.min_tech {
        return Err(EquipmentError::TechTooLow {
            name: item.name.clone(),
            needed: item.min_tech,
            have: tech_level,
        });
    }
    Ok(())
}

pub fn shop_view(commander: &Commander, ship: &Ship, tech_level: u8) -> ShopView {
    ShopView {
        credits: commander.credits(),
        power_used: ship.equipment.power_used(),
        power_capacity: ship.equipment.power_capacity(),
        categories: EquipmentCategory::ALL
            .into_iter()
            .map(|category| ShopCategory {
                category,
                items: equipment::catalog_for(category)
                    .map(|item| ShopItem {
                        item,
                        installed: ship.equipment.installed().iter().any(|i| i.id == item.id),
                        locked: tech_level < item.min_tech,
                    })
                    .collect(),
            })
            .collect(),
    }
}

pub fn installed_view(ship: &Ship) -> InstalledView {
    let loadout = &ship.equipment;
    InstalledView {
        power_used: loadout.power_used(),
        power_capacity: loadout.power_capacity(),
        categories: EquipmentCategory::ALL
            .into_iter()
            .map(|category| InstalledCategory {
                category,
                slots_used: loadout.slots_used(category),
                slots_capacity: loadout.slots_capacity(category),
                items: loadout
                    .installed()
                    .iter()
                    .map(Installed::item)
                    .filter(|item| item.category == category)
                    .collect(),
            })
            .collect(),
        cargo_capacity: ship.cargo_capacity(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup(credits: u32) -> (Commander, Ship) {
        let mut commander = Commander::new("Test".to_string());
        commander.credits = credits;
        (commander, Ship::new_cobra_mk3())
    }

    #[test]
    fn new_ship_starts_with_basic_equipment_and_unchanged_capacity() {
        let ship = Ship::new_cobra_mk3();
        assert_eq!(ship.equipment.installed().len(), 7);
        assert_eq!(ship.equipment.power_used(), 9);
        assert_eq!(ship.cargo_capacity(), 20);
    }

    fn item_id(text: &str) -> String {
        text.split_whitespace()
            .collect::<Vec<_>>()
            .join("-")
            .to_lowercase()
    }

    fn buy(text: &str, commander: &mut Commander, ship: &mut Ship) -> bool {
        find_item(&item_id(text))
            .map(|item| super::buy(commander, ship, item).is_ok())
            .unwrap_or(false)
    }

    fn sell(text: &str, commander: &mut Commander, ship: &mut Ship) -> bool {
        find_item(&item_id(text))
            .map(|item| super::sell(commander, ship, item).is_ok())
            .unwrap_or(false)
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

    #[test]
    fn purchases_and_sales_report_what_changed() {
        let (mut commander, mut ship) = setup(1_000);
        let racks = find_item("expanded-racks").unwrap();

        let bought = super::buy(&mut commander, &mut ship, racks).unwrap();
        assert_eq!(bought.price, racks.price);
        assert_eq!(bought.replaced, Some(("Standard Racks".to_string(), 0)));
        assert_eq!(
            super::buy(&mut commander, &mut ship, racks),
            Err(EquipmentError::AlreadyInstalled(racks.name.clone()))
        );

        let sold = super::sell(&mut commander, &mut ship, racks).unwrap();
        assert_eq!(sold.refund, racks.price / 2);
        assert_eq!(
            super::sell(&mut commander, &mut ship, racks),
            Err(EquipmentError::NotInstalled(racks.name.clone()))
        );
        assert_eq!(
            find_item("nonsense"),
            Err(EquipmentError::Unknown("nonsense".to_string()))
        );
    }

    #[test]
    fn the_views_list_every_category() {
        let (commander, ship) = setup(100);

        let shop = shop_view(&commander, &ship, 15);
        assert_eq!(shop.categories.len(), EquipmentCategory::ALL.len());
        assert!(shop
            .categories
            .iter()
            .flat_map(|group| &group.items)
            .any(|entry| entry.installed));

        let installed = installed_view(&ship);
        assert_eq!(installed.power_capacity, 20);
        assert_eq!(installed.cargo_capacity, 20);
    }

    #[test]
    fn a_better_reactor_raises_capacity_and_makes_room_for_more_equipment() {
        let (mut commander, mut ship) = setup(2_000);
        assert!(buy("military-shield", &mut commander, &mut ship));
        assert!(!buy("missile-launcher", &mut commander, &mut ship));

        assert!(buy("improved-reactor", &mut commander, &mut ship));

        assert_eq!(ship.equipment.power_capacity(), 28);
        assert!(buy("missile-launcher", &mut commander, &mut ship));
        assert_eq!(ship.equipment.power_used(), 21);
    }

    #[test]
    fn swapping_a_reactor_refunds_the_old_one() {
        let (mut commander, mut ship) = setup(500);
        buy("improved-reactor", &mut commander, &mut ship);
        let credits = commander.credits();

        buy("military-reactor", &mut commander, &mut ship);

        assert_eq!(commander.credits(), credits + 30 - 160);
        assert!(!has(&ship, "improved-reactor"));
    }

    #[test]
    fn a_weaker_reactor_is_refused_while_the_equipment_needs_more_power() {
        let (mut commander, mut ship) = setup(2_000);
        buy("military-reactor", &mut commander, &mut ship);
        buy("elite-shield", &mut commander, &mut ship);
        buy("missile-launcher", &mut commander, &mut ship);
        let credits = commander.credits();

        assert!(!buy("basic-reactor", &mut commander, &mut ship));

        assert_eq!(commander.credits(), credits);
        assert!(has(&ship, "military-reactor"));
    }

    #[test]
    fn the_only_reactor_cannot_be_sold() {
        let (mut commander, mut ship) = setup(100);

        assert!(!sell("basic-reactor", &mut commander, &mut ship));

        assert!(has(&ship, "basic-reactor"));
        assert_eq!(ship.equipment.power_capacity(), 20);
    }

    #[test]
    fn saved_loadouts_without_a_reactor_get_the_basic_one() {
        let json =
            r#"{"installed":[{"id":"pulse-laser","paid":0},{"id":"basic-shield","paid":0}]}"#;

        let loadout: Loadout = serde_json::from_str(json).unwrap();

        assert_eq!(loadout.power_capacity(), 20);
        assert_eq!(loadout.power_used(), 5);
        assert_eq!(loadout.slots_used(EquipmentCategory::Power), 1);
    }
}
