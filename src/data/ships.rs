use serde::{Deserialize, Serialize};

/// Represents different ship types available in the game
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ShipType {
    CobraMkIII,
    CobraMkIV,
    Python,
    Anaconda,
}

impl ShipType {
    /// Get the display name for the ship type
    pub fn display_name(&self) -> &'static str {
        match self {
            ShipType::CobraMkIII => "Cobra Mk III",
            ShipType::CobraMkIV => "Cobra Mk IV",
            ShipType::Python => "Python",
            ShipType::Anaconda => "Anaconda",
        }
    }

    /// Get the short name for commands
    #[allow(dead_code)]
    pub fn short_name(&self) -> &'static str {
        match self {
            ShipType::CobraMkIII => "cobra3",
            ShipType::CobraMkIV => "cobra4",
            ShipType::Python => "python",
            ShipType::Anaconda => "anaconda",
        }
    }

    /// Get the cost to purchase this ship
    pub fn cost(&self) -> u32 {
        match self {
            ShipType::CobraMkIII => 0,  // Starting ship
            ShipType::CobraMkIV => 500, // First upgrade
            ShipType::Python => 2000,   // Mid-game ship
            ShipType::Anaconda => 5000, // End-game ship
        }
    }

    /// Get the cargo capacity for this ship
    pub fn cargo_capacity(&self) -> u32 {
        match self {
            ShipType::CobraMkIII => 20,
            ShipType::CobraMkIV => 35,
            ShipType::Python => 60,
            ShipType::Anaconda => 100,
        }
    }

    /// Get the fuel capacity for this ship
    pub fn fuel_capacity(&self) -> u32 {
        match self {
            ShipType::CobraMkIII => 7,
            ShipType::CobraMkIV => 10,
            ShipType::Python => 12,
            ShipType::Anaconda => 15,
        }
    }

    /// Get the maximum hull percentage for this ship
    pub fn max_hull(&self) -> u32 {
        match self {
            ShipType::CobraMkIII => 100,
            ShipType::CobraMkIV => 120,
            ShipType::Python => 150,
            ShipType::Anaconda => 200,
        }
    }

    /// Check if this ship is an upgrade from another ship
    pub fn is_upgrade_from(&self, other: &ShipType) -> bool {
        self.cost() > other.cost()
    }

    /// Get the upgrade cost from another ship to this ship
    pub fn upgrade_cost_from(&self, from: &ShipType) -> u32 {
        if self.cost() > from.cost() {
            self.cost() - from.cost()
        } else {
            0
        }
    }

    /// Parse ship type from string
    /// Match a ship by name, ignoring case, spaces, hyphens and quotes ("Cobra Mk IV", "cobra-mk-4", "cobra4")
    pub fn from_string(name: &str) -> Option<Self> {
        let key: String = name
            .chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .collect::<String>()
            .to_lowercase();
        match key.as_str() {
            "cobra3" | "cobramkiii" | "cobramk3" => Some(ShipType::CobraMkIII),
            "cobra4" | "cobramkiv" | "cobramk4" => Some(ShipType::CobraMkIV),
            "python" => Some(ShipType::Python),
            "anaconda" => Some(ShipType::Anaconda),
            _ => None,
        }
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
            available_ships: vec![
                ShipType::CobraMkIII,
                ShipType::CobraMkIV,
                ShipType::Python,
                ShipType::Anaconda,
            ],
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

    #[test]
    fn ship_names_match_in_every_written_form() {
        for name in [
            "cobra4",
            "Cobra Mk IV",
            "\"Cobra Mk IV\"",
            "cobra-mk-4",
            "COBRA MK IV",
        ] {
            assert_eq!(
                ShipType::from_string(name),
                Some(ShipType::CobraMkIV),
                "{}",
                name
            );
        }
        assert_eq!(
            ShipType::from_string("Cobra Mk III"),
            Some(ShipType::CobraMkIII)
        );
        assert_eq!(ShipType::from_string("python"), Some(ShipType::Python));
        assert_eq!(ShipType::from_string("nonsense"), None);
        assert_eq!(ShipType::from_string(""), None);
    }

    #[test]
    fn test_ship_type_display_names() {
        assert_eq!(ShipType::CobraMkIII.display_name(), "Cobra Mk III");
        assert_eq!(ShipType::Python.display_name(), "Python");
    }

    #[test]
    fn test_ship_type_costs() {
        assert_eq!(ShipType::CobraMkIII.cost(), 0);
        assert_eq!(ShipType::CobraMkIV.cost(), 500);
        assert_eq!(ShipType::Python.cost(), 2000);
    }

    #[test]
    fn test_upgrade_relationships() {
        assert!(ShipType::CobraMkIV.is_upgrade_from(&ShipType::CobraMkIII));
        assert!(ShipType::Python.is_upgrade_from(&ShipType::CobraMkIV));
        assert!(!ShipType::CobraMkIII.is_upgrade_from(&ShipType::CobraMkIV));
    }

    #[test]
    fn test_upgrade_costs() {
        assert_eq!(
            ShipType::CobraMkIV.upgrade_cost_from(&ShipType::CobraMkIII),
            500
        );
        assert_eq!(
            ShipType::Python.upgrade_cost_from(&ShipType::CobraMkIV),
            1500
        );
    }
}
