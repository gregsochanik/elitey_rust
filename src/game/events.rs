use crate::data::systems::{StarSystem, EconomyType, GovernmentType};
use crate::game::{Commander, Ship};
use crate::utils::display::show_info;
use rand::Rng;
use std::collections::HashMap;

/// Represents the phase of travel where an event can occur
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum EventPhase {
    Jump,      // During hyperspace jump
    Approach,  // After jump, before reaching station
}

/// Represents different types of events that can occur during travel
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum EventType {
    // Jump Events
    Wormhole,
    HyperspaceAnomaly,
    NavigationError,
    SpaceDebris,
    
    // Approach Events
    AlienEncounter,
    PirateAttack,
    RescueSignal,
    GovernmentInspection,
    MilitaryPatrol,
}

/// Represents the outcome of an event choice
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EventOutcome {
    pub description: String,
    pub credits_change: i32,
    pub cargo_loss: Option<String>, // Commodity type to lose
    pub hull_damage: i32,
    pub fuel_change: i32,
    pub reputation_change: i32,
    pub new_location: Option<String>, // For wormhole events
    pub message: String,
}

/// Represents a choice the player can make during an event
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EventChoice {
    pub description: String,
    pub outcome: EventOutcome,
}

/// Represents a complete event with choices and outcomes
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Event {
    pub event_type: EventType,
    pub phase: EventPhase,
    pub title: String,
    pub description: String,
    pub choices: Vec<EventChoice>,
}

/// Manages the event system and handles event triggering
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EventManager {
    events: HashMap<EventType, Event>,
}

impl Default for EventManager {
    fn default() -> Self {
        Self::new()
    }
}

impl EventManager {
    pub fn new() -> Self {
        let mut events = HashMap::new();
        
        // Initialize all events
        events.insert(EventType::Wormhole, Self::create_wormhole_event());
        events.insert(EventType::HyperspaceAnomaly, Self::create_hyperspace_anomaly_event());
        events.insert(EventType::NavigationError, Self::create_navigation_error_event());
        events.insert(EventType::SpaceDebris, Self::create_space_debris_event());
        events.insert(EventType::AlienEncounter, Self::create_alien_encounter_event());
        events.insert(EventType::PirateAttack, Self::create_pirate_attack_event());
        events.insert(EventType::RescueSignal, Self::create_rescue_signal_event());
        events.insert(EventType::GovernmentInspection, Self::create_government_inspection_event());
        events.insert(EventType::MilitaryPatrol, Self::create_military_patrol_event());
        
        Self { events }
    }
    
    /// Check if an event should trigger for a given phase
    pub fn should_trigger_event(&self, phase: &EventPhase, destination: &StarSystem) -> bool {
        let base_probability = match phase {
            EventPhase::Jump => 0.10,      // 10% base chance for jump events
            EventPhase::Approach => 0.12,  // 12% base chance for approach events
        };
        
        let modifier = self.calculate_contextual_modifier(destination);
        let final_probability = base_probability * modifier;
        
        let mut rng = rand::thread_rng();
        rng.gen_bool(final_probability)
    }
    
    /// Calculate contextual modifier based on destination system characteristics
    fn calculate_contextual_modifier(&self, destination: &StarSystem) -> f64 {
        let mut modifier = 1.0;
        
        // Government-based modifiers
        match destination.government {
            GovernmentType::Anarchy => modifier *= 1.3,        // More events in lawless space
            GovernmentType::Communist => modifier *= 1.2,      // More inspections
            GovernmentType::CorporateState => modifier *= 0.9, // Fewer events in corporate space
            GovernmentType::Democracy => modifier *= 1.0,      // Standard
            GovernmentType::Dictatorship => modifier *= 1.1,   // More military presence
            GovernmentType::Feudal => modifier *= 1.0,         // Standard
            GovernmentType::MultiGovernment => modifier *= 1.0, // Standard
            GovernmentType::Confederacy => modifier *= 1.0,    // Standard
        }
        
        // Economy-based modifiers
        match destination.economy {
            EconomyType::RichIndustrial => modifier *= 0.8,    // Fewer events in rich space
            EconomyType::PoorAgricultural => modifier *= 1.4,  // More desperate encounters
            EconomyType::MainlyAgricultural => modifier *= 1.1, // Slightly more agricultural events
            EconomyType::MainlyIndustrial => modifier *= 1.1,  // Slightly more industrial events
            EconomyType::PoorIndustrial => modifier *= 1.2,    // More industrial accidents
            EconomyType::AverageIndustrial => modifier *= 1.0, // Standard
            EconomyType::RichAgricultural => modifier *= 0.9,  // Fewer events in rich agricultural
            EconomyType::AverageAgricultural => modifier *= 1.0, // Standard
        }
        
        modifier
    }
    
    /// Get available events for a given phase
    pub fn get_available_events(&self, phase: &EventPhase) -> Vec<&Event> {
        self.events.values()
            .filter(|event| event.phase == *phase)
            .collect()
    }
    
    /// Select a random event for a given phase, considering destination context
    pub fn select_random_event(&self, phase: &EventPhase, _destination: &StarSystem) -> Option<&Event> {
        let available_events = self.get_available_events(phase);
        if available_events.is_empty() {
            return None;
        }
        
        // For now, select randomly. In the future, we could weight based on destination
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..available_events.len());
        Some(available_events[index])
    }
    
    /// Process an event and return the outcome
    pub fn process_event(&self, event: &Event, commander: &mut Commander, ship: &mut Ship, choice_index: usize) -> Result<EventOutcome, String> {
        if choice_index >= event.choices.len() {
            return Err("Invalid choice index".to_string());
        }
        
        let choice = &event.choices[choice_index];
        let outcome = &choice.outcome;
        
        // Apply outcome effects
        commander.credits = (commander.credits as i32 + outcome.credits_change).max(0) as u32;
        
        if let Some(cargo_type) = &outcome.cargo_loss {
            // Remove cargo if specified - for now, just show the message
            show_info(&format!("Lost 1 cargo: {}", cargo_type));
            // TODO: Implement proper cargo removal when we have string-to-commodity conversion
        }
        
        commander.reputation += outcome.reputation_change;

        ship.take_damage(outcome.hull_damage.unsigned_abs());
        ship.fuel_current = (ship.fuel_current as i32 + outcome.fuel_change).max(0) as u32;
        
        // Update location if specified (for wormhole events)
        if let Some(new_location) = &outcome.new_location {
            commander.location = new_location.clone();
        }
        
        Ok(outcome.clone())
    }
    
    // Event creation methods
    fn create_wormhole_event() -> Event {
        Event {
            event_type: EventType::Wormhole,
            phase: EventPhase::Jump,
            title: "🌌 Wormhole Detected".to_string(),
            description: "A mysterious wormhole has appeared in your path. It could lead anywhere...".to_string(),
            choices: vec![
                EventChoice {
                    description: "Enter the wormhole".to_string(),
                    outcome: EventOutcome {
                        description: "You enter the wormhole and emerge in an unknown system".to_string(),
                        credits_change: 0,
                        cargo_loss: None,
                        hull_damage: 0,
                        fuel_change: -2,
                        reputation_change: 0,
                        new_location: Some("UNKNOWN".to_string()),
                        message: "The wormhole has transported you to an unknown location!".to_string(),
                    },
                },
                EventChoice {
                    description: "Avoid the wormhole".to_string(),
                    outcome: EventOutcome {
                        description: "You carefully navigate around the wormhole".to_string(),
                        credits_change: 0,
                        cargo_loss: None,
                        hull_damage: 0,
                        fuel_change: -1,
                        reputation_change: 0,
                        new_location: None,
                        message: "You safely avoid the wormhole and continue your journey.".to_string(),
                    },
                },
            ],
        }
    }
    
    fn create_hyperspace_anomaly_event() -> Event {
        Event {
            event_type: EventType::HyperspaceAnomaly,
            phase: EventPhase::Jump,
            title: "⚡ Hyperspace Anomaly".to_string(),
            description: "Your sensors detect an unusual energy signature in hyperspace.".to_string(),
            choices: vec![
                EventChoice {
                    description: "Investigate the anomaly".to_string(),
                    outcome: EventOutcome {
                        description: "You investigate and find rare materials".to_string(),
                        credits_change: 500,
                        cargo_loss: None,
                        hull_damage: -5,
                        fuel_change: -1,
                        reputation_change: 10,
                        new_location: None,
                        message: "You discover valuable materials! +500 credits, +10 reputation".to_string(),
                    },
                },
                EventChoice {
                    description: "Avoid the anomaly".to_string(),
                    outcome: EventOutcome {
                        description: "You safely avoid the anomaly".to_string(),
                        credits_change: 0,
                        cargo_loss: None,
                        hull_damage: 0,
                        fuel_change: 0,
                        reputation_change: 0,
                        new_location: None,
                        message: "You safely avoid the anomaly and continue your journey.".to_string(),
                    },
                },
            ],
        }
    }
    
    fn create_navigation_error_event() -> Event {
        Event {
            event_type: EventType::NavigationError,
            phase: EventPhase::Jump,
            title: "🧭 Navigation Error".to_string(),
            description: "Your navigation systems are behaving erratically.".to_string(),
            choices: vec![
                EventChoice {
                    description: "Correct course".to_string(),
                    outcome: EventOutcome {
                        description: "You correct the navigation error".to_string(),
                        credits_change: 0,
                        cargo_loss: None,
                        hull_damage: 0,
                        fuel_change: -2,
                        reputation_change: 0,
                        new_location: None,
                        message: "Navigation corrected, but you used extra fuel.".to_string(),
                    },
                },
                EventChoice {
                    description: "Continue as planned".to_string(),
                    outcome: EventOutcome {
                        description: "You continue despite the error".to_string(),
                        credits_change: 0,
                        cargo_loss: None,
                        hull_damage: 0,
                        fuel_change: -3,
                        reputation_change: -5,
                        new_location: None,
                        message: "You arrive at your destination but used more fuel than expected.".to_string(),
                    },
                },
            ],
        }
    }
    
    fn create_space_debris_event() -> Event {
        Event {
            event_type: EventType::SpaceDebris,
            phase: EventPhase::Jump,
            title: "🗑️ Space Debris Field".to_string(),
            description: "You encounter a field of space debris from an old battle.".to_string(),
            choices: vec![
                EventChoice {
                    description: "Salvage the debris".to_string(),
                    outcome: EventOutcome {
                        description: "You salvage valuable materials".to_string(),
                        credits_change: 200,
                        cargo_loss: None,
                        hull_damage: -2,
                        fuel_change: 0,
                        reputation_change: 0,
                        new_location: None,
                        message: "You salvage valuable materials! +200 credits, minor hull damage.".to_string(),
                    },
                },
                EventChoice {
                    description: "Avoid the debris".to_string(),
                    outcome: EventOutcome {
                        description: "You carefully navigate around the debris".to_string(),
                        credits_change: 0,
                        cargo_loss: None,
                        hull_damage: 0,
                        fuel_change: 0,
                        reputation_change: 0,
                        new_location: None,
                        message: "You safely avoid the debris field.".to_string(),
                    },
                },
            ],
        }
    }
    
    fn create_alien_encounter_event() -> Event {
        Event {
            event_type: EventType::AlienEncounter,
            phase: EventPhase::Approach,
            title: "👽 Alien Encounter".to_string(),
            description: "An alien vessel approaches your ship.".to_string(),
            choices: vec![
                EventChoice {
                    description: "Attempt communication".to_string(),
                    outcome: EventOutcome {
                        description: "You successfully communicate with the aliens".to_string(),
                        credits_change: 300,
                        cargo_loss: None,
                        hull_damage: 0,
                        fuel_change: 0,
                        reputation_change: 15,
                        new_location: None,
                        message: "The aliens are friendly! +300 credits, +15 reputation.".to_string(),
                    },
                },
                EventChoice {
                    description: "Flee".to_string(),
                    outcome: EventOutcome {
                        description: "You flee from the alien vessel".to_string(),
                        credits_change: 0,
                        cargo_loss: None,
                        hull_damage: -3,
                        fuel_change: -2,
                        reputation_change: -5,
                        new_location: None,
                        message: "You escape but take some damage and use extra fuel.".to_string(),
                    },
                },
                EventChoice {
                    description: "Fight".to_string(),
                    outcome: EventOutcome {
                        description: "You engage in combat with the aliens".to_string(),
                        credits_change: 400,
                        cargo_loss: None,
                        hull_damage: -8,
                        fuel_change: 0,
                        reputation_change: -10,
                        new_location: None,
                        message: "You win the battle! +400 credits, but significant hull damage.".to_string(),
                    },
                },
            ],
        }
    }
    
    fn create_pirate_attack_event() -> Event {
        Event {
            event_type: EventType::PirateAttack,
            phase: EventPhase::Approach,
            title: "🏴‍☠️ Pirate Attack".to_string(),
            description: "Pirates are attacking your ship!".to_string(),
            choices: vec![
                EventChoice {
                    description: "Fight the pirates".to_string(),
                    outcome: EventOutcome {
                        description: "You engage in combat with the pirates".to_string(),
                        credits_change: 600,
                        cargo_loss: None,
                        hull_damage: -10,
                        fuel_change: 0,
                        reputation_change: 5,
                        new_location: None,
                        message: "You defeat the pirates! +600 credits, but heavy hull damage.".to_string(),
                    },
                },
                EventChoice {
                    description: "Flee".to_string(),
                    outcome: EventOutcome {
                        description: "You attempt to flee from the pirates".to_string(),
                        credits_change: 0,
                        cargo_loss: Some("Food".to_string()),
                        hull_damage: -5,
                        fuel_change: -3,
                        reputation_change: -5,
                        new_location: None,
                        message: "You escape but lose some cargo and take damage.".to_string(),
                    },
                },
                EventChoice {
                    description: "Surrender".to_string(),
                    outcome: EventOutcome {
                        description: "You surrender to the pirates".to_string(),
                        credits_change: -200,
                        cargo_loss: Some("Food".to_string()),
                        hull_damage: 0,
                        fuel_change: 0,
                        reputation_change: -10,
                        new_location: None,
                        message: "You surrender and lose cargo and credits, but avoid damage.".to_string(),
                    },
                },
            ],
        }
    }
    
    fn create_rescue_signal_event() -> Event {
        Event {
            event_type: EventType::RescueSignal,
            phase: EventPhase::Approach,
            title: "🛸 Rescue Signal".to_string(),
            description: "You detect a distress signal from a nearby vessel.".to_string(),
            choices: vec![
                EventChoice {
                    description: "Investigate the signal".to_string(),
                    outcome: EventOutcome {
                        description: "You investigate and find a genuine distress call".to_string(),
                        credits_change: 400,
                        cargo_loss: None,
                        hull_damage: 0,
                        fuel_change: 0,
                        reputation_change: 20,
                        new_location: None,
                        message: "You rescue the crew! +400 credits, +20 reputation.".to_string(),
                    },
                },
                EventChoice {
                    description: "Ignore the signal".to_string(),
                    outcome: EventOutcome {
                        description: "You ignore the distress signal".to_string(),
                        credits_change: 0,
                        cargo_loss: None,
                        hull_damage: 0,
                        fuel_change: 0,
                        reputation_change: -5,
                        new_location: None,
                        message: "You ignore the signal and continue your journey.".to_string(),
                    },
                },
            ],
        }
    }
    
    fn create_government_inspection_event() -> Event {
        Event {
            event_type: EventType::GovernmentInspection,
            phase: EventPhase::Approach,
            title: "🛡️ Government Inspection".to_string(),
            description: "Government officials are requesting an inspection of your vessel.".to_string(),
            choices: vec![
                EventChoice {
                    description: "Comply with inspection".to_string(),
                    outcome: EventOutcome {
                        description: "You comply with the government inspection".to_string(),
                        credits_change: -50,
                        cargo_loss: None,
                        hull_damage: 0,
                        fuel_change: 0,
                        reputation_change: 10,
                        new_location: None,
                        message: "Inspection completed. -50 credits for fees, +10 reputation.".to_string(),
                    },
                },
                EventChoice {
                    description: "Attempt to bribe".to_string(),
                    outcome: EventOutcome {
                        description: "You attempt to bribe the officials".to_string(),
                        credits_change: -100,
                        cargo_loss: None,
                        hull_damage: 0,
                        fuel_change: 0,
                        reputation_change: -5,
                        new_location: None,
                        message: "Bribe successful. -100 credits, minor reputation loss.".to_string(),
                    },
                },
                EventChoice {
                    description: "Resist inspection".to_string(),
                    outcome: EventOutcome {
                        description: "You resist the government inspection".to_string(),
                        credits_change: 0,
                        cargo_loss: Some("Narcotics".to_string()),
                        hull_damage: 0,
                        fuel_change: 0,
                        reputation_change: -15,
                        new_location: None,
                        message: "You resist but lose any illegal cargo and reputation.".to_string(),
                    },
                },
            ],
        }
    }
    
    fn create_military_patrol_event() -> Event {
        Event {
            event_type: EventType::MilitaryPatrol,
            phase: EventPhase::Approach,
            title: "🚨 Military Patrol".to_string(),
            description: "A military patrol vessel is approaching your ship.".to_string(),
            choices: vec![
                EventChoice {
                    description: "Identify yourself".to_string(),
                    outcome: EventOutcome {
                        description: "You properly identify yourself to the military".to_string(),
                        credits_change: 0,
                        cargo_loss: None,
                        hull_damage: 0,
                        fuel_change: 0,
                        reputation_change: 5,
                        new_location: None,
                        message: "Standard procedure completed. +5 reputation.".to_string(),
                    },
                },
                EventChoice {
                    description: "Attempt to evade".to_string(),
                    outcome: EventOutcome {
                        description: "You attempt to evade the military patrol".to_string(),
                        credits_change: 0,
                        cargo_loss: None,
                        hull_damage: -3,
                        fuel_change: -2,
                        reputation_change: -10,
                        new_location: None,
                        message: "You evade but take damage and use extra fuel.".to_string(),
                    },
                },
                EventChoice {
                    description: "Engage the patrol".to_string(),
                    outcome: EventOutcome {
                        description: "You engage in combat with the military patrol".to_string(),
                        credits_change: 800,
                        cargo_loss: None,
                        hull_damage: -15,
                        fuel_change: 0,
                        reputation_change: -20,
                        new_location: None,
                        message: "You defeat the patrol! +800 credits, but heavy damage and reputation loss.".to_string(),
                    },
                },
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::systems::{StarSystem, EconomyType, GovernmentType};
    
    #[test]
    fn test_event_manager_creation() {
        let manager = EventManager::new();
        assert_eq!(manager.events.len(), 9); // 9 event types
    }
    
    #[test]
    fn test_phase_event_filtering() {
        let manager = EventManager::new();
        
        let jump_events = manager.get_available_events(&EventPhase::Jump);
        let approach_events = manager.get_available_events(&EventPhase::Approach);
        
        assert_eq!(jump_events.len(), 4); // 4 jump events
        assert_eq!(approach_events.len(), 5); // 5 approach events
    }
    
    #[test]
    fn test_contextual_modifier_calculation() {
        let manager = EventManager::new();
        
        // Test anarchy system (should have higher modifier)
        let anarchy_system = StarSystem {
            name: "Test".to_string(),
            x: 0,
            y: 0,
            economy: EconomyType::MainlyAgricultural,
            government: GovernmentType::Anarchy,
            tech_level: 5,
            turnover: 1000,
            radius: 100,
            population: "1000000".to_string(),
            description: "Test system".to_string(),
        };
        
        let modifier = manager.calculate_contextual_modifier(&anarchy_system);
        assert!(modifier > 1.0); // Should be higher than base
    }
    
    #[test]
    fn test_event_processing() {
        let manager = EventManager::new();
        let mut commander = Commander::new("Test".to_string());
        let mut ship = Ship::new_cobra_mk3();
        
        let event = manager.events.get(&EventType::Wormhole).unwrap();
        let initial_credits = commander.credits;
        let initial_fuel = ship.fuel_current;
        
        let result = manager.process_event(event, &mut commander, &mut ship, 0);
        assert!(result.is_ok());
        
        // Check that effects were applied
        assert_eq!(commander.credits, initial_credits);
        assert_eq!(ship.fuel_current, initial_fuel - 2);
    }

    #[test]
    fn event_outcome_changes_commander_reputation() {
        let manager = EventManager::new();
        let mut commander = Commander::new("Test".to_string());
        let mut ship = Ship::new_cobra_mk3();

        let event = manager.events.get(&EventType::AlienEncounter).unwrap();
        manager.process_event(event, &mut commander, &mut ship, 0).unwrap();

        assert_eq!(commander.reputation, 15);
    }

    #[test]
    fn reputation_can_go_negative() {
        let manager = EventManager::new();
        let mut commander = Commander::new("Test".to_string());
        let mut ship = Ship::new_cobra_mk3();

        let event = manager.events.get(&EventType::MilitaryPatrol).unwrap();
        for index in 0..event.choices.len() {
            let before = commander.reputation;
            manager.process_event(event, &mut commander, &mut ship, index).unwrap();
            if commander.reputation < before {
                return;
            }
        }
        panic!("no military patrol choice lowers reputation");
    }
}
