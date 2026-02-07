use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use anyhow::Result;

/// Economy types for star systems (matching galaxy1.json)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EconomyType {
    #[serde(rename = "Rich Ind")]
    RichIndustrial,
    #[serde(rename = "Poor Ind")]
    PoorIndustrial,
    #[serde(rename = "Mainly Ind")]
    MainlyIndustrial,
    #[serde(rename = "Average Ind")]
    AverageIndustrial,
    #[serde(rename = "Rich Agri")]
    RichAgricultural,
    #[serde(rename = "Poor Agri")]
    PoorAgricultural,
    #[serde(rename = "Mainly Agri")]
    MainlyAgricultural,
    #[serde(rename = "Average Agri")]
    AverageAgricultural,
}

impl EconomyType {
    /// Get a human-readable name for the economy type
    #[allow(dead_code)]
    pub fn display_name(&self) -> &'static str {
        match self {
            EconomyType::RichIndustrial => "Rich Industrial",
            EconomyType::PoorIndustrial => "Poor Industrial",
            EconomyType::MainlyIndustrial => "Mainly Industrial",
            EconomyType::AverageIndustrial => "Average Industrial",
            EconomyType::RichAgricultural => "Rich Agricultural",
            EconomyType::PoorAgricultural => "Poor Agricultural",
            EconomyType::MainlyAgricultural => "Mainly Agricultural",
            EconomyType::AverageAgricultural => "Average Agricultural",
        }
    }
}

/// Government types for star systems (matching galaxy1.json)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GovernmentType {
    #[serde(rename = "Anarchy")]
    Anarchy,
    #[serde(rename = "Feudal")]
    Feudal,
    #[serde(rename = "Multi-Government")]
    MultiGovernment,
    #[serde(rename = "Dictatorship")]
    Dictatorship,
    #[serde(rename = "Communist")]
    Communist,
    #[serde(rename = "Confederacy")]
    Confederacy,
    #[serde(rename = "Democracy")]
    Democracy,
    #[serde(rename = "Corporate State")]
    CorporateState,
}

impl GovernmentType {
    /// Get a human-readable name for the government type
    #[allow(dead_code)]
    pub fn display_name(&self) -> &'static str {
        match self {
            GovernmentType::Anarchy => "Anarchy",
            GovernmentType::Feudal => "Feudal",
            GovernmentType::MultiGovernment => "Multi-Government",
            GovernmentType::Dictatorship => "Dictatorship",
            GovernmentType::Communist => "Communist",
            GovernmentType::Confederacy => "Confederacy",
            GovernmentType::Democracy => "Democracy",
            GovernmentType::CorporateState => "Corporate State",
        }
    }
}

/// JSON structure for economy data from galaxy1.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomyData {
    level: u8,
    name: String,
}

/// JSON structure for government data from galaxy1.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernmentData {
    level: u8,
    name: String,
}

/// JSON structure for a star system from galaxy1.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalaxySystemData {
    name: String,
    x: u16,
    y: u16,
    economy: EconomyData,
    government: GovernmentData,
    techlevel: u8,
    turnover: u32,
    radius: u32,
    population: String,
    description: String,
}

/// Represents a star system with all its properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StarSystem {
    /// System name
    pub name: String,
    /// X coordinate
    pub x: u16,
    /// Y coordinate
    pub y: u16,
    /// Economy type
    pub economy: EconomyType,
    /// Government type
    pub government: GovernmentType,
    /// Technology level
    pub tech_level: u8,
    /// Economic turnover
    pub turnover: u32,
    /// Planet radius
    pub radius: u32,
    /// Population
    pub population: String,
    /// System description
    pub description: String,
}

impl StarSystem {
    /// Create a new star system from galaxy data
    pub fn from_galaxy_data(data: GalaxySystemData) -> Result<Self, serde_json::Error> {
        // Parse economy type from the name string
        let economy = serde_json::from_str(&format!("\"{}\"", data.economy.name))?;
        
        // Parse government type from the name string
        let government = serde_json::from_str(&format!("\"{}\"", data.government.name))?;
        
        Ok(Self {
            name: data.name,
            x: data.x,
            y: data.y,
            economy,
            government,
            tech_level: data.techlevel,
            turnover: data.turnover,
            radius: data.radius,
            population: data.population,
            description: data.description,
        })
    }

    /// Get the system name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the economy type
    #[allow(dead_code)]
    pub fn economy(&self) -> &EconomyType {
        &self.economy
    }

    /// Get the government type
    #[allow(dead_code)]
    pub fn government(&self) -> &GovernmentType {
        &self.government
    }

    /// Get technology level
    #[allow(dead_code)]
    pub fn tech_level(&self) -> u8 {
        self.tech_level
    }

    /// Get economic turnover
    #[allow(dead_code)]
    pub fn turnover(&self) -> u32 {
        self.turnover
    }

    /// Get population
    #[allow(dead_code)]
    pub fn population(&self) -> &str {
        &self.population
    }

    /// Get system description
    #[allow(dead_code)]
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Calculate distance to another system using Elite's distance formula
    pub fn distance_to(&self, other: &StarSystem) -> f64 {
        let dx = self.x as i32 - other.x as i32;
        let dy = self.y as i32 - other.y as i32;
        let distance_squared = (dx * dx) + (dy * dy);
        (distance_squared as f64).sqrt() / 4.0
    }
}

/// Manages all star systems and travel mechanics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SystemManager {
    systems: HashMap<String, StarSystem>,
}

impl Default for SystemManager {
    fn default() -> Self {
        Self::new().expect("Failed to load galaxy data")
    }
}

impl SystemManager {
    /// Create a new system manager by loading from galaxy1.json
    pub fn new() -> Result<Self> {
        let mut systems = HashMap::new();
        
        // Load galaxy data from JSON file
        let galaxy_data = fs::read_to_string("galaxies/galaxy1.json")?;
        let galaxy_systems: HashMap<String, GalaxySystemData> = serde_json::from_str(&galaxy_data)?;
        
        // Convert each system from JSON data to StarSystem
        for (system_id, system_data) in galaxy_systems {
            match StarSystem::from_galaxy_data(system_data) {
                Ok(system) => {
                    systems.insert(system.name.clone(), system);
                }
                Err(e) => {
                    eprintln!("Failed to parse system {}: {}", system_id, e);
                }
            }
        }
        
        Ok(Self { systems })
    }

    /// Get a system by name
    pub fn get_system(&self, name: &str) -> Option<&StarSystem> {
        self.systems.get(name)
    }

    /// Get all available systems
    pub fn get_all_systems(&self) -> Vec<&StarSystem> {
        self.systems.values().collect()
    }

    /// Calculate distance between two systems using 3D coordinates
    pub fn calculate_distance(&self, from: &str, to: &str) -> Option<f64> {
        let from_system = self.get_system(from)?;
        let to_system = self.get_system(to)?;
        
        Some(from_system.distance_to(to_system))
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
        let test_data = GalaxySystemData {
            name: "Test System".to_string(),
            x: 10,
            y: 20,
            economy: EconomyData {
                level: 0,
                name: "Rich Ind".to_string(),
            },
            government: GovernmentData {
                level: 2,
                name: "Multi-Government".to_string(),
            },
            techlevel: 10,
            turnover: 20000,
            radius: 5000,
            population: "40 Billion".to_string(),
            description: "A test system.".to_string(),
        };
        
        let system = StarSystem::from_galaxy_data(test_data).unwrap();
        
        assert_eq!(system.name(), "Test System");
        assert_eq!(system.economy(), &EconomyType::RichIndustrial);
        assert_eq!(system.government(), &GovernmentType::MultiGovernment);
        assert_eq!(system.tech_level(), 10);
    }

    #[test]
    fn test_system_manager() {
        let manager = SystemManager::new().unwrap();
        
        // Test getting a system
        let lave = manager.get_system("LAVE");
        assert!(lave.is_some());
        assert_eq!(lave.unwrap().name(), "LAVE");
        
        // Test getting all systems
        let all_systems = manager.get_all_systems();
        assert!(all_systems.len() > 100); // Should have hundreds of systems
    }

    #[test]
    fn test_distance_calculation() {
        let manager = SystemManager::new().unwrap();
        
        let distance = manager.calculate_distance("LAVE", "LEESTI");
        assert!(distance.is_some());
        assert!(distance.unwrap() > 0.0); // Should be a positive distance
    }

    #[test]
    fn test_travel_validation() {
        let manager = SystemManager::new().unwrap();
        
        // Test travel with sufficient fuel
        assert!(manager.can_travel("LAVE", "LEESTI", 20.0));
        
        // Test travel with insufficient fuel
        assert!(!manager.can_travel("LAVE", "LEESTI", 1.0));
    }
}
