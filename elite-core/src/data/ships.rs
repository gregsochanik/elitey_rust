use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::sync::OnceLock;

/// One kind of ship, as described in `data/ships.json`
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShipClass {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    /// The lowest system tech level that sells this ship
    pub min_tech: u8,
    /// Orders ships for upgrades: a ship is an upgrade from any ship of a lower tier
    pub tier: u32,
    pub cost: u32,
    pub cargo: u32,
    pub fuel: u32,
    pub hull: u32,
}

/// Lower case letters and digits only, so "Cobra Mk IV", "cobra-mk-iv" and "\"cobra mk iv\"" match
fn name_key(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect::<String>()
        .to_lowercase()
}

impl ShipClass {
    fn matches(&self, key: &str) -> bool {
        name_key(&self.id) == key
            || name_key(&self.name) == key
            || self.aliases.iter().any(|alias| name_key(alias) == key)
    }
}

/// Every ship in the game, lowest tier first
pub fn ship_classes() -> &'static [ShipClass] {
    static SHIPS: OnceLock<Vec<ShipClass>> = OnceLock::new();
    SHIPS.get_or_init(|| {
        let mut ships: Vec<ShipClass> = serde_json::from_str(include_str!("../../data/ships.json"))
            .expect("data/ships.json is valid");
        ships.sort_by_key(|ship| ship.tier);
        ships
    })
}

/// A ship the player can fly or buy. It serialises as its id and loads old saves that stored
/// the earlier names (`CobraMkIII`, `Python`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShipType(&'static ShipClass);

impl Serialize for ShipType {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.id())
    }
}

impl<'de> Deserialize<'de> for ShipType {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let name = String::deserialize(deserializer)?;
        ShipType::from_string(&name)
            .ok_or_else(|| serde::de::Error::custom(format!("unknown ship '{}'", name)))
    }
}

impl ShipType {
    /// The ship every commander starts with: the lowest tier
    pub fn starting() -> Self {
        ShipType(&ship_classes()[0])
    }

    /// Every ship, lowest tier first
    pub fn all() -> Vec<ShipType> {
        ship_classes().iter().map(ShipType).collect()
    }

    pub fn id(&self) -> &'static str {
        &self.0.id
    }

    /// Get the display name for the ship type
    pub fn display_name(&self) -> &'static str {
        &self.0.name
    }

    /// Get the cost to purchase this ship
    pub fn cost(&self) -> u32 {
        self.0.cost
    }

    /// Get the cargo capacity for this ship
    pub fn cargo_capacity(&self) -> u32 {
        self.0.cargo
    }

    /// Get the fuel capacity for this ship
    pub fn fuel_capacity(&self) -> u32 {
        self.0.fuel
    }

    /// Get the maximum hull for this ship
    pub fn max_hull(&self) -> u32 {
        self.0.hull
    }

    /// Check if this ship is an upgrade from another ship
    /// The lowest system tech level that sells this ship
    pub fn min_tech(&self) -> u8 {
        self.0.min_tech
    }

    pub fn is_upgrade_from(&self, other: &ShipType) -> bool {
        self.0.tier > other.0.tier
    }

    /// Get the upgrade cost from another ship to this ship
    pub fn upgrade_cost_from(&self, from: &ShipType) -> u32 {
        self.cost().saturating_sub(from.cost())
    }

    /// Match a ship by id, name or alias, ignoring case, spaces, hyphens and quotes
    /// ("Cobra Mk IV", "cobra-mk-4", "cobra4")
    pub fn from_string(name: &str) -> Option<Self> {
        let key = name_key(name);
        if key.is_empty() {
            return None;
        }
        ship_classes()
            .iter()
            .find(|ship| ship.matches(&key))
            .map(ShipType)
    }
}

/// Manages ship upgrades and comparisons
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ShipUpgradeManager {
    available_ships: Vec<ShipType>,
}

impl ShipUpgradeManager {
    /// Create a new ship upgrade manager
    pub fn new() -> Self {
        Self {
            available_ships: ShipType::all(),
        }
    }

    /// Get available upgrades for a current ship
    pub fn get_available_upgrades(&self, current_ship: &ShipType) -> Vec<&ShipType> {
        self.available_ships
            .iter()
            .filter(|ship| ship.is_upgrade_from(current_ship))
            .collect()
    }

    /// Calculate upgrade cost between ships
    pub fn calculate_upgrade_cost(&self, from: &ShipType, to: &ShipType) -> u32 {
        to.upgrade_cost_from(from)
    }

    /// Get all available ships
    pub fn get_available_ships(&self) -> &[ShipType] {
        &self.available_ships
    }
}

impl Default for ShipUpgradeManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn ship(name: &str) -> ShipType {
        ShipType::from_string(name).unwrap_or_else(|| panic!("no ship '{}'", name))
    }

    #[test]
    fn ship_names_match_in_every_written_form() {
        for name in [
            "cobra4",
            "Cobra Mk IV",
            "\"Cobra Mk IV\"",
            "cobra-mk-4",
            "COBRA MK IV",
            "cobra-mk-iv",
        ] {
            assert_eq!(ship(name).id(), "cobra-mk-iv", "{}", name);
        }
        assert_eq!(ship("Cobra Mk III").id(), "cobra-mk-iii");
        assert_eq!(ship("python").id(), "python");
        assert_eq!(ShipType::from_string("nonsense"), None);
        assert_eq!(ShipType::from_string(""), None);
        assert_eq!(ShipType::from_string("\"\""), None);
    }

    #[test]
    fn the_starting_ship_is_the_free_lowest_tier() {
        let starting = ShipType::starting();
        assert_eq!(starting.id(), "cobra-mk-iii");
        assert_eq!(starting.cost(), 0);
        assert_eq!(starting.cargo_capacity(), 20);
        assert_eq!(starting.fuel_capacity(), 7);
        assert_eq!(starting.max_hull(), 100);
    }

    #[test]
    fn test_ship_type_display_names() {
        assert_eq!(ship("cobra3").display_name(), "Cobra Mk III");
        assert_eq!(ship("python").display_name(), "Python");
    }

    #[test]
    fn test_ship_type_costs() {
        assert_eq!(ship("cobra3").cost(), 0);
        assert_eq!(ship("cobra4").cost(), 500);
        assert_eq!(ship("python").cost(), 2000);
        assert_eq!(ship("anaconda").cost(), 5000);
    }

    #[test]
    fn test_upgrade_relationships() {
        assert!(ship("cobra4").is_upgrade_from(&ship("cobra3")));
        assert!(ship("python").is_upgrade_from(&ship("cobra4")));
        assert!(!ship("cobra3").is_upgrade_from(&ship("cobra4")));
        assert!(!ship("python").is_upgrade_from(&ship("python")));
    }

    #[test]
    fn test_upgrade_costs() {
        assert_eq!(ship("cobra4").upgrade_cost_from(&ship("cobra3")), 500);
        assert_eq!(ship("python").upgrade_cost_from(&ship("cobra4")), 1500);
        assert_eq!(ship("cobra3").upgrade_cost_from(&ship("python")), 0);
    }

    #[test]
    fn ships_serialise_as_ids_and_load_the_names_old_saves_used() {
        assert_eq!(
            serde_json::to_string(&ship("cobra4")).unwrap(),
            "\"cobra-mk-iv\""
        );

        for (old, id) in [
            ("CobraMkIII", "cobra-mk-iii"),
            ("CobraMkIV", "cobra-mk-iv"),
            ("Python", "python"),
            ("Anaconda", "anaconda"),
            ("cobra-mk-iv", "cobra-mk-iv"),
        ] {
            let loaded: ShipType = serde_json::from_str(&format!("\"{}\"", old)).unwrap();
            assert_eq!(loaded.id(), id, "{}", old);
        }
        assert!(serde_json::from_str::<ShipType>("\"Corvette\"").is_err());
    }

    #[test]
    fn ship_data_is_consistent() {
        let ships = ship_classes();
        assert!(ships.len() >= 2);

        let ids: HashSet<_> = ships.iter().map(|s| s.id.as_str()).collect();
        assert_eq!(ids.len(), ships.len(), "ship ids must be unique");
        let tiers: HashSet<_> = ships.iter().map(|s| s.tier).collect();
        assert_eq!(tiers.len(), ships.len(), "ship tiers must be unique");

        let mut owners: std::collections::HashMap<String, &str> = std::collections::HashMap::new();
        for ship in ships {
            assert!(!ship.id.is_empty() && !ship.name.is_empty());
            assert!(
                ship.cargo > 0 && ship.fuel > 0 && ship.hull > 0,
                "{}",
                ship.id
            );
            for name in [&ship.id, &ship.name]
                .into_iter()
                .chain(ship.aliases.iter())
            {
                let key = name_key(name);
                assert!(!key.is_empty(), "{} has an empty name", ship.id);
                let previous = owners.insert(key.clone(), &ship.id);
                assert!(
                    previous.is_none() || previous == Some(ship.id.as_str()),
                    "'{}' names more than one ship",
                    key
                );
            }
        }
    }

    #[test]
    fn higher_tiers_cost_more_cargo_and_hull_and_the_first_is_free() {
        let ships = ship_classes();
        assert_eq!(ships[0].cost, 0);
        for pair in ships.windows(2) {
            assert!(pair[1].cost > pair[0].cost, "{} costs no more", pair[1].id);
            assert!(pair[1].cargo >= pair[0].cargo, "{} holds less", pair[1].id);
            assert!(pair[1].hull >= pair[0].hull, "{} has less hull", pair[1].id);
        }
    }

    #[test]
    fn unknown_fields_in_the_data_are_rejected() {
        let json =
            r#"[{"id":"x","name":"X","tier":1,"cost":0,"cargo":1,"fuel":1,"hull":1,"speed":9}]"#;
        assert!(serde_json::from_str::<Vec<ShipClass>>(json).is_err());
    }

    #[test]
    fn upgrades_lists_only_higher_tiers() {
        let manager = ShipUpgradeManager::new();
        let ids: Vec<_> = manager
            .get_available_upgrades(&ship("cobra4"))
            .iter()
            .map(|s| s.id())
            .collect();
        assert_eq!(ids, vec!["python", "anaconda"]);
    }
}
