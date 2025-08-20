use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Economy types for star systems
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EconomyType {
    Agricultural,
    Industrial,
    HighTech,
    Mining,
    Tourism,
    Military,
    Service,
}

impl EconomyType {
    /// Get a human-readable name for the economy type
    pub fn display_name(&self) -> &'static str {
        match self {
            EconomyType::Agricultural => "Agricultural",
            EconomyType::Industrial => "Industrial",
            EconomyType::HighTech => "High-Tech",
            EconomyType::Mining => "Mining",
            EconomyType::Tourism => "Tourism",
            EconomyType::Military => "Military",
            EconomyType::Service => "Service",
        }
    }
}

/// Represents a star system with all its properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StarSystem {
    /// System name
    pub name: String,
    /// Unique identifier
    pub id: String,
    /// Economy type
    pub economy: EconomyType,
    /// Distance from Lave in light years
    pub distance_from_lave: f64,
    /// Available commodities (simplified for now)
    pub available_commodities: Vec<String>,
    /// System description
    pub description: String,
}

impl StarSystem {
    /// Create a new star system
    pub fn new(
        name: String,
        id: String,
        economy: EconomyType,
        distance_from_lave: f64,
        available_commodities: Vec<String>,
        description: String,
    ) -> Self {
        Self {
            name,
            id,
            economy,
            distance_from_lave,
            available_commodities,
            description,
        }
    }

    /// Get the system name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the economy type
    pub fn economy(&self) -> &EconomyType {
        &self.economy
    }

    /// Get distance from Lave
    pub fn distance_from_lave(&self) -> f64 {
        self.distance_from_lave
    }

    /// Get available commodities
    pub fn available_commodities(&self) -> &[String] {
        &self.available_commodities
    }

    /// Get system description
    pub fn description(&self) -> &str {
        &self.description
    }
}

/// Manages all star systems and travel mechanics
#[derive(Debug, Clone)]
pub struct SystemManager {
    systems: HashMap<String, StarSystem>,
}

impl Default for SystemManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemManager {
    /// Create a new system manager with predefined systems
    pub fn new() -> Self {
        let mut systems = HashMap::new();
        
        // Add predefined systems as per Elite lore
        let lave = StarSystem::new(
            "Lave".to_string(),
            "LAVE".to_string(),
            EconomyType::Agricultural,
            0.0,
            vec!["Food".to_string(), "Textiles".to_string(), "Liquor".to_string()],
            "A peaceful agricultural world known for its fine wines and textiles.".to_string(),
        );
        
        let leesti = StarSystem::new(
            "Leesti".to_string(),
            "LEESTI".to_string(),
            EconomyType::Industrial,
            15.2,
            vec!["Machinery".to_string(), "Computers".to_string(), "Alloys".to_string()],
            "An industrial powerhouse producing advanced machinery and computers.".to_string(),
        );
        
        let diso = StarSystem::new(
            "Diso".to_string(),
            "DISO".to_string(),
            EconomyType::Agricultural,
            8.7,
            vec!["Food".to_string(), "Livestock".to_string(), "Grain".to_string()],
            "A fertile agricultural world with vast farmlands and livestock.".to_string(),
        );
        
        let zaonce = StarSystem::new(
            "Zaonce".to_string(),
            "ZAONCE".to_string(),
            EconomyType::HighTech,
            12.3,
            vec!["Computers".to_string(), "Electronics".to_string(), "Robots".to_string()],
            "A high-tech research center specializing in advanced electronics.".to_string(),
        );
        
        let orrere = StarSystem::new(
            "Orrere".to_string(),
            "ORRERE".to_string(),
            EconomyType::Mining,
            18.9,
            vec!["Minerals".to_string(), "Ore".to_string(), "Precious Metals".to_string()],
            "A mining colony extracting valuable minerals from asteroid belts.".to_string(),
        );

        systems.insert(lave.name.clone(), lave);
        systems.insert(leesti.name.clone(), leesti);
        systems.insert(diso.name.clone(), diso);
        systems.insert(zaonce.name.clone(), zaonce);
        systems.insert(orrere.name.clone(), orrere);

        Self { systems }
    }

    /// Get a system by name
    pub fn get_system(&self, name: &str) -> Option<&StarSystem> {
        self.systems.get(name)
    }

    /// Get all available systems
    pub fn get_all_systems(&self) -> Vec<&StarSystem> {
        self.systems.values().collect()
    }

    /// Calculate distance between two systems
    pub fn calculate_distance(&self, from: &str, to: &str) -> Option<f64> {
        let from_system = self.get_system(from)?;
        let to_system = self.get_system(to)?;
        
        // For now, use distance from Lave as a simple approximation
        // In a full implementation, this would use actual 3D coordinates
        let distance = (from_system.distance_from_lave - to_system.distance_from_lave).abs();
        Some(distance)
    }

    /// Check if travel is possible (fuel check)
    pub fn can_travel(&self, from: &str, to: &str, current_fuel: f64) -> bool {
        if let Some(distance) = self.calculate_distance(from, to) {
            distance <= current_fuel
        } else {
            false
        }
    }

    /// Calculate fuel cost for travel
    pub fn calculate_fuel_cost(&self, from: &str, to: &str) -> Option<f64> {
        self.calculate_distance(from, to)
    }

    /// Calculate travel time in seconds (simplified)
    #[allow(dead_code)]
    pub fn calculate_travel_time(&self, from: &str, to: &str) -> Option<u64> {
        self.calculate_distance(from, to).map(|distance| (distance * 10.0) as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_creation() {
        let system = StarSystem::new(
            "Test System".to_string(),
            "TEST".to_string(),
            EconomyType::HighTech,
            10.0,
            vec!["Computers".to_string()],
            "A test system.".to_string(),
        );
        
        assert_eq!(system.name(), "Test System");
        assert_eq!(system.economy(), &EconomyType::HighTech);
        assert_eq!(system.distance_from_lave(), 10.0);
    }

    #[test]
    fn test_system_manager() {
        let manager = SystemManager::new();
        
        // Test getting a system
        let lave = manager.get_system("Lave");
        assert!(lave.is_some());
        assert_eq!(lave.unwrap().name(), "Lave");
        
        // Test getting all systems
        let all_systems = manager.get_all_systems();
        assert_eq!(all_systems.len(), 5);
    }

    #[test]
    fn test_distance_calculation() {
        let manager = SystemManager::new();
        
        let distance = manager.calculate_distance("Lave", "Leesti");
        assert!(distance.is_some());
        assert_eq!(distance.unwrap(), 15.2);
    }

    #[test]
    fn test_travel_validation() {
        let manager = SystemManager::new();
        
        // Test travel with sufficient fuel
        assert!(manager.can_travel("Lave", "Leesti", 20.0));
        
        // Test travel with insufficient fuel
        assert!(!manager.can_travel("Lave", "Leesti", 10.0));
    }
}
