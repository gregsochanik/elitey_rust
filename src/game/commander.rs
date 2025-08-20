use serde::{Deserialize, Serialize};

/// Represents the player commander with all their data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commander {
    /// Commander's name
    pub name: String,
    /// Current credits
    pub credits: u32,
    /// Current location (system name)
    pub location: String,
}

#[allow(dead_code)]
impl Commander {
    /// Create a new commander with starting conditions
    pub fn new(name: String) -> Self {
        Self {
            name,
            credits: 100, // Starting credits as per spec
            location: "LAVE".to_string(), // Starting location as per spec
        }
    }

    /// Get the commander's name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get current credits
    pub fn credits(&self) -> u32 {
        self.credits
    }

    /// Get current location
    pub fn location(&self) -> &str {
        &self.location
    }

    /// Add credits to the commander's account
    pub fn add_credits(&mut self, amount: u32) {
        self.credits += amount;
    }

    /// Remove credits from the commander's account
    /// Returns true if successful, false if insufficient funds
    pub fn remove_credits(&mut self, amount: u32) -> bool {
        if self.credits >= amount {
            self.credits -= amount;
            true
        } else {
            false
        }
    }

    /// Check if commander has sufficient credits
    pub fn has_credits(&self, amount: u32) -> bool {
        self.credits >= amount
    }

    /// Update the commander's location
    pub fn set_location(&mut self, location: String) {
        self.location = location;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commander_creation() {
        let commander = Commander::new("Test Commander".to_string());
        assert_eq!(commander.name(), "Test Commander");
        assert_eq!(commander.credits(), 100);
        assert_eq!(commander.location(), "LAVE");
    }

    #[test]
    fn test_add_credits() {
        let mut commander = Commander::new("Test".to_string());
        commander.add_credits(50);
        assert_eq!(commander.credits(), 150);
    }

    #[test]
    fn test_remove_credits_success() {
        let mut commander = Commander::new("Test".to_string());
        let result = commander.remove_credits(30);
        assert!(result);
        assert_eq!(commander.credits(), 70);
    }

    #[test]
    fn test_remove_credits_insufficient() {
        let mut commander = Commander::new("Test".to_string());
        let result = commander.remove_credits(150);
        assert!(!result);
        assert_eq!(commander.credits(), 100); // Credits unchanged
    }

    #[test]
    fn test_has_credits() {
        let commander = Commander::new("Test".to_string());
        assert!(commander.has_credits(50));
        assert!(commander.has_credits(100));
        assert!(!commander.has_credits(150));
    }

    #[test]
    fn test_set_location() {
        let mut commander = Commander::new("Test".to_string());
        commander.set_location("Leesti".to_string());
        assert_eq!(commander.location(), "Leesti");
    }
}
