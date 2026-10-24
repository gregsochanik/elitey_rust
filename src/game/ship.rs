use crate::data::commodities::{CargoHold, CommodityType};
use crate::data::ships::ShipType;
use crate::game::equipment::Loadout;
use serde::{Deserialize, Serialize};

/// Represents the player's ship with all its properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ship {
    /// Ship type
    pub ship_type: ShipType,
    /// Cargo hold
    pub cargo_hold: CargoHold,
    /// Current fuel level in light years
    pub fuel_current: u32,
    /// Hull points, up to the ship type's maximum
    pub hull_percentage: u32,
    /// Installed equipment
    #[serde(default)]
    pub equipment: Loadout,
}

impl Ship {
    /// Create a new Cobra Mk III (starting ship as per spec)
    pub fn new_cobra_mk3() -> Self {
        let ship_type = ShipType::starting();
        Self {
            cargo_hold: CargoHold::new(ship_type.cargo_capacity()),
            fuel_current: ship_type.fuel_capacity(),
            hull_percentage: ship_type.max_hull(),
            ship_type,
            equipment: Loadout::starting(),
        }
    }

    /// Get ship type
    pub fn ship_type(&self) -> &ShipType {
        &self.ship_type
    }

    /// Get ship type display name
    pub fn ship_type_name(&self) -> &str {
        self.ship_type.display_name()
    }

    /// Get cargo capacity
    pub fn cargo_capacity(&self) -> u32 {
        self.cargo_hold.capacity
    }

    /// Get current cargo used
    pub fn cargo_used(&self) -> u32 {
        self.cargo_hold.current_weight()
    }

    /// Get available cargo space
    pub fn cargo_available(&self) -> u32 {
        self.cargo_hold.remaining_capacity()
    }

    /// Check if ship has cargo space
    pub fn has_cargo_space(&self, amount: u32) -> bool {
        self.cargo_hold.can_add(amount)
    }

    /// Add cargo to ship
    /// Returns true if successful, false if insufficient space
    pub fn add_cargo(&mut self, commodity: CommodityType, amount: u32) -> bool {
        self.cargo_hold.add_cargo(commodity, amount)
    }

    /// Remove cargo from ship
    /// Returns true if successful, false if insufficient cargo
    pub fn remove_cargo(&mut self, commodity: &CommodityType, amount: u32) -> bool {
        self.cargo_hold.remove_cargo(commodity, amount)
    }

    /// Get cargo hold reference
    pub fn cargo_hold(&self) -> &CargoHold {
        &self.cargo_hold
    }

    /// Get cargo hold mutable reference
    pub fn cargo_hold_mut(&mut self) -> &mut CargoHold {
        &mut self.cargo_hold
    }

    /// Upgrade to a new ship type
    pub fn upgrade_to(&mut self, new_ship_type: ShipType) -> Result<(), String> {
        // Check if the new ship has enough cargo capacity for current cargo
        let current_cargo = self.cargo_hold.current_weight();
        let new_cargo_capacity = new_ship_type.cargo_capacity();

        if current_cargo > new_cargo_capacity {
            return Err(format!(
                "Cannot upgrade to {}: current cargo ({} tons) exceeds new ship capacity ({} tons)",
                new_ship_type.display_name(),
                current_cargo,
                new_cargo_capacity
            ));
        }

        // Transfer cargo to new ship
        let old_cargo_hold =
            std::mem::replace(&mut self.cargo_hold, CargoHold::new(new_cargo_capacity));

        // Copy cargo items to new ship
        for item in old_cargo_hold.get_items() {
            self.cargo_hold.add_cargo(item.commodity, item.quantity);
        }

        self.equipment = Loadout::starting();

        // Update ship type
        self.ship_type = new_ship_type;

        // Update hull to new ship's max hull
        self.hull_percentage = new_ship_type.max_hull();

        Ok(())
    }

    /// Replace the equipment loadout and resize the cargo hold to match.
    /// Fails, changing nothing, if the cargo held would not fit.
    pub fn set_equipment(&mut self, equipment: Loadout) -> Result<(), String> {
        let capacity =
            (self.ship_type.cargo_capacity() as f64 * equipment.cargo_multiplier()) as u32;
        let held = self.cargo_hold.current_weight();
        if held > capacity {
            return Err(format!(
                "Cargo hold would be too small: holding {} tons, capacity would be {} tons",
                held, capacity
            ));
        }
        self.cargo_hold.capacity = capacity;
        self.equipment = equipment;
        Ok(())
    }

    /// Get ship cost
    pub fn ship_cost(&self) -> u32 {
        self.ship_type.cost()
    }

    /// Get fuel capacity
    pub fn fuel_capacity(&self) -> u32 {
        self.ship_type.fuel_capacity()
    }

    /// Get current fuel level
    pub fn fuel_current(&self) -> u32 {
        self.fuel_current
    }

    /// Check if ship has sufficient fuel
    pub fn has_fuel(&self, amount: u32) -> bool {
        self.fuel_current >= amount
    }

    /// Consume fuel
    /// Returns true if successful, false if insufficient fuel
    pub fn consume_fuel(&mut self, amount: u32) -> bool {
        if self.has_fuel(amount) {
            self.fuel_current -= amount;
            true
        } else {
            false
        }
    }

    /// Refuel ship to full capacity
    pub fn refuel(&mut self) {
        self.fuel_current = self.fuel_capacity();
    }

    /// Hull as current/maximum points, e.g. `98/120`
    pub fn hull_display(&self) -> String {
        format!("{}/{}", self.hull_percentage, self.ship_type.max_hull())
    }

    /// Get hull percentage
    pub fn hull_percentage(&self) -> u32 {
        self.hull_percentage
    }

    /// Check if hull is damaged
    pub fn is_damaged(&self) -> bool {
        self.hull_percentage < self.ship_type.max_hull()
    }

    /// Hull points missing from the maximum
    pub fn damage(&self) -> u32 {
        self.ship_type
            .max_hull()
            .saturating_sub(self.hull_percentage)
    }

    /// Repair hull damage
    pub fn repair_hull(&mut self, amount: u32) {
        self.hull_percentage = (self.hull_percentage + amount).min(self.ship_type.max_hull());
    }

    /// Take hull damage
    pub fn take_damage(&mut self, amount: u32) {
        if amount >= self.hull_percentage {
            self.hull_percentage = 0;
        } else {
            self.hull_percentage -= amount;
        }
    }

    /// Check if ship is destroyed (hull at 0%)
    pub fn is_destroyed(&self) -> bool {
        self.hull_percentage == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ship_creation() {
        let ship = Ship::new_cobra_mk3();
        assert_eq!(ship.ship_type_name(), "Cobra Mk III");
        assert_eq!(ship.cargo_capacity(), 20);
        assert_eq!(ship.cargo_used(), 0);
        assert_eq!(ship.fuel_capacity(), 7);
        assert_eq!(ship.fuel_current(), 7);
        assert_eq!(ship.hull_percentage(), 100);
    }

    #[test]
    fn test_cargo_operations() {
        let mut ship = Ship::new_cobra_mk3();

        // Test cargo space
        assert_eq!(ship.cargo_available(), 20);
        assert!(ship.has_cargo_space(10));
        assert!(!ship.has_cargo_space(25));

        // Test adding cargo
        assert!(ship.add_cargo(CommodityType::Food, 10));
        assert_eq!(ship.cargo_used(), 10);
        assert_eq!(ship.cargo_available(), 10);

        // Test adding too much cargo
        assert!(!ship.add_cargo(CommodityType::Machinery, 15));
        assert_eq!(ship.cargo_used(), 10); // Unchanged

        // Test removing cargo
        assert!(ship.remove_cargo(&CommodityType::Food, 5));
        assert_eq!(ship.cargo_used(), 5);
        assert_eq!(ship.cargo_available(), 15);

        // Test removing too much cargo
        assert!(!ship.remove_cargo(&CommodityType::Food, 10));
        assert_eq!(ship.cargo_used(), 5); // Unchanged
    }

    #[test]
    fn test_fuel_operations() {
        let mut ship = Ship::new_cobra_mk3();

        // Test fuel checks
        assert!(ship.has_fuel(5));
        assert!(ship.has_fuel(7));
        assert!(!ship.has_fuel(10));

        // Test consuming fuel
        assert!(ship.consume_fuel(3));
        assert_eq!(ship.fuel_current(), 4);
        assert!(!ship.consume_fuel(10)); // Too much fuel

        // Test refueling
        ship.refuel();
        assert_eq!(ship.fuel_current(), 7);
    }

    #[test]
    fn hull_display_shows_points_against_the_ship_maximum() {
        let mut ship = Ship::new_cobra_mk3();
        ship.take_damage(2);
        assert_eq!(ship.hull_display(), "98/100");
        ship.ship_type = crate::data::ships::ShipType::from_string("python").unwrap();
        ship.hull_percentage = 150;
        assert_eq!(ship.hull_display(), "150/150");
    }

    #[test]
    fn test_hull_operations() {
        let mut ship = Ship::new_cobra_mk3();

        // Test initial state
        assert!(!ship.is_damaged());
        assert!(!ship.is_destroyed());

        // Test taking damage
        ship.take_damage(30);
        assert_eq!(ship.hull_percentage(), 70);
        assert!(ship.is_damaged());
        assert!(!ship.is_destroyed());

        // Test repair
        ship.repair_hull(20);
        assert_eq!(ship.hull_percentage(), 90);

        // Test over-repair (should cap at 100)
        ship.repair_hull(20);
        assert_eq!(ship.hull_percentage(), 100);

        // Test destruction
        ship.take_damage(100);
        assert_eq!(ship.hull_percentage(), 0);
        assert!(ship.is_destroyed());
    }
}
