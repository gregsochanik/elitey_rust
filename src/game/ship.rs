use serde::{Deserialize, Serialize};

/// Represents the player's ship with all its properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ship {
    /// Ship type name
    pub ship_type: String,
    /// Cargo capacity in tons
    pub cargo_capacity: u32,
    /// Current cargo used in tons
    pub cargo_used: u32,
    /// Maximum fuel capacity in light years
    pub fuel_capacity: u32,
    /// Current fuel level in light years
    pub fuel_current: u32,
    /// Hull integrity percentage (0-100)
    pub hull_percentage: u32,
}

#[allow(dead_code)]
impl Ship {
    /// Create a new Cobra Mk III (starting ship as per spec)
    pub fn new_cobra_mk3() -> Self {
        Self {
            ship_type: "Cobra Mk III".to_string(),
            cargo_capacity: 20, // Starting cargo capacity as per spec
            cargo_used: 0,      // Empty cargo hold as per spec
            fuel_capacity: 20,  // Fuel capacity in light years (increased for testing)
            fuel_current: 20,   // Full fuel tank
            hull_percentage: 100, // Full hull integrity
        }
    }

    /// Get ship type
    pub fn ship_type(&self) -> &str {
        &self.ship_type
    }

    /// Get cargo capacity
    pub fn cargo_capacity(&self) -> u32 {
        self.cargo_capacity
    }

    /// Get current cargo used
    pub fn cargo_used(&self) -> u32 {
        self.cargo_used
    }

    /// Get available cargo space
    pub fn cargo_available(&self) -> u32 {
        self.cargo_capacity - self.cargo_used
    }

    /// Check if ship has cargo space
    pub fn has_cargo_space(&self, amount: u32) -> bool {
        self.cargo_available() >= amount
    }

    /// Add cargo to ship
    /// Returns true if successful, false if insufficient space
    pub fn add_cargo(&mut self, amount: u32) -> bool {
        if self.has_cargo_space(amount) {
            self.cargo_used += amount;
            true
        } else {
            false
        }
    }

    /// Remove cargo from ship
    /// Returns true if successful, false if insufficient cargo
    pub fn remove_cargo(&mut self, amount: u32) -> bool {
        if self.cargo_used >= amount {
            self.cargo_used -= amount;
            true
        } else {
            false
        }
    }

    /// Get fuel capacity
    pub fn fuel_capacity(&self) -> u32 {
        self.fuel_capacity
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
        self.fuel_current = self.fuel_capacity;
    }

    /// Get hull percentage
    pub fn hull_percentage(&self) -> u32 {
        self.hull_percentage
    }

    /// Check if hull is damaged
    pub fn is_damaged(&self) -> bool {
        self.hull_percentage < 100
    }

    /// Repair hull damage
    pub fn repair_hull(&mut self, amount: u32) {
        self.hull_percentage = (self.hull_percentage + amount).min(100);
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
        assert_eq!(ship.ship_type(), "Cobra Mk III");
        assert_eq!(ship.cargo_capacity(), 20);
        assert_eq!(ship.cargo_used(), 0);
        assert_eq!(ship.fuel_capacity(), 20);
        assert_eq!(ship.fuel_current(), 20);
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
        assert!(ship.add_cargo(10));
        assert_eq!(ship.cargo_used(), 10);
        assert_eq!(ship.cargo_available(), 10);

        // Test adding too much cargo
        assert!(!ship.add_cargo(15));
        assert_eq!(ship.cargo_used(), 10); // Unchanged

        // Test removing cargo
        assert!(ship.remove_cargo(5));
        assert_eq!(ship.cargo_used(), 5);
        assert_eq!(ship.cargo_available(), 15);

        // Test removing too much cargo
        assert!(!ship.remove_cargo(10));
        assert_eq!(ship.cargo_used(), 5); // Unchanged
    }

    #[test]
    fn test_fuel_operations() {
        let mut ship = Ship::new_cobra_mk3();
        
        // Test fuel checks
        assert!(ship.has_fuel(5));
        assert!(ship.has_fuel(10));
        assert!(!ship.has_fuel(25));

        // Test consuming fuel
        assert!(ship.consume_fuel(3));
        assert_eq!(ship.fuel_current(), 17);
        assert!(!ship.consume_fuel(25)); // Too much fuel

        // Test refueling
        ship.refuel();
        assert_eq!(ship.fuel_current(), 20);
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
