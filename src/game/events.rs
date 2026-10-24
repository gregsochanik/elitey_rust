use crate::data::commodities::CommodityType;
use crate::data::systems::{EconomyType, GovernmentType, StarSystem};
use crate::game::{Commander, Ship};
use crate::utils::display::show_info;
use rand::Rng;
use std::collections::HashMap;

/// Represents the phase of travel where an event can occur
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum EventPhase {
    Jump,     // During hyperspace jump
    Approach, // After jump, before reaching station
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

/// Cargo an event takes from the hold
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum CargoLoss {
    /// Up to this many tons, taken from the largest cargo first
    Tons(u32),
    /// Everything of one commodity
    All(CommodityType),
}

/// Cargo an event adds to the hold, as much as fits
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CargoGain {
    pub commodity: CommodityType,
    pub tons: u32,
}

/// Represents the outcome of an event choice
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EventOutcome {
    pub description: String,
    pub credits_change: i32,
    pub cargo_loss: Option<CargoLoss>,
    pub cargo_gain: Option<CargoGain>,
    pub hull_damage: i32,
    pub fuel_change: i32,
    pub reputation_change: i32,
    pub starts_combat: bool,
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

/// How much more or less likely an event is at this destination (1.0 is normal)
fn event_weight(event_type: &EventType, destination: &StarSystem) -> f64 {
    government_weight(event_type, &destination.government)
        * economy_weight(event_type, &destination.economy)
}

fn government_weight(event_type: &EventType, government: &GovernmentType) -> f64 {
    match (government, event_type) {
        (GovernmentType::Anarchy, EventType::PirateAttack) => 2.5,
        (GovernmentType::Anarchy, EventType::GovernmentInspection) => 0.2,
        (GovernmentType::Anarchy, EventType::MilitaryPatrol) => 0.3,
        (GovernmentType::Communist, EventType::GovernmentInspection) => 2.5,
        (GovernmentType::Communist, EventType::MilitaryPatrol) => 1.5,
        (GovernmentType::Communist, EventType::PirateAttack) => 0.6,
        (GovernmentType::Dictatorship, EventType::MilitaryPatrol) => 2.5,
        (GovernmentType::Dictatorship, EventType::GovernmentInspection) => 1.5,
        (GovernmentType::Dictatorship, EventType::PirateAttack) => 0.7,
        (GovernmentType::CorporateState, EventType::GovernmentInspection) => 1.5,
        (GovernmentType::CorporateState, EventType::PirateAttack) => 0.5,
        _ => 1.0,
    }
}

fn economy_weight(event_type: &EventType, economy: &EconomyType) -> f64 {
    match (economy, event_type) {
        (EconomyType::PoorAgricultural | EconomyType::PoorIndustrial, EventType::PirateAttack) => {
            1.5
        }
        (EconomyType::PoorAgricultural | EconomyType::PoorIndustrial, EventType::RescueSignal) => {
            1.5
        }
        (EconomyType::RichIndustrial | EconomyType::RichAgricultural, EventType::PirateAttack) => {
            0.6
        }
        (EconomyType::MainlyIndustrial, EventType::SpaceDebris) => 1.5,
        (EconomyType::MainlyAgricultural, EventType::RescueSignal) => 1.3,
        _ => 1.0,
    }
}

/// Remove cargo from the hold and return what was taken
pub(crate) fn apply_cargo_loss(ship: &mut Ship, loss: &CargoLoss) -> Vec<(CommodityType, u32)> {
    match loss {
        CargoLoss::All(commodity) => {
            let held = ship.cargo_hold.get_quantity(commodity);
            if held > 0 {
                ship.cargo_hold.remove_cargo(commodity, held);
                vec![(commodity.clone(), held)]
            } else {
                Vec::new()
            }
        }
        CargoLoss::Tons(tons) => {
            let mut items = ship.cargo_hold.get_items();
            items.sort_by(|a, b| {
                b.quantity
                    .cmp(&a.quantity)
                    .then_with(|| a.commodity.name().cmp(b.commodity.name()))
            });

            let mut remaining = *tons;
            let mut taken = Vec::new();
            for item in items {
                if remaining == 0 {
                    break;
                }
                let amount = item.quantity.min(remaining);
                ship.cargo_hold.remove_cargo(&item.commodity, amount);
                remaining -= amount;
                taken.push((item.commodity, amount));
            }
            taken
        }
    }
}

/// Add as much of the gain as fits and return the tons added
pub(crate) fn apply_cargo_gain(ship: &mut Ship, gain: &CargoGain) -> u32 {
    let tons = gain.tons.min(ship.cargo_hold.remaining_capacity());
    if tons > 0 {
        ship.cargo_hold.add_cargo(gain.commodity.clone(), tons);
    }
    tons
}

impl EventManager {
    pub fn new() -> Self {
        let mut events = HashMap::new();

        // Initialize all events
        events.insert(EventType::Wormhole, Self::create_wormhole_event());
        events.insert(
            EventType::HyperspaceAnomaly,
            Self::create_hyperspace_anomaly_event(),
        );
        events.insert(
            EventType::NavigationError,
            Self::create_navigation_error_event(),
        );
        events.insert(EventType::SpaceDebris, Self::create_space_debris_event());
        events.insert(
            EventType::AlienEncounter,
            Self::create_alien_encounter_event(),
        );
        events.insert(EventType::PirateAttack, Self::create_pirate_attack_event());
        events.insert(EventType::RescueSignal, Self::create_rescue_signal_event());
        events.insert(
            EventType::GovernmentInspection,
            Self::create_government_inspection_event(),
        );
        events.insert(
            EventType::MilitaryPatrol,
            Self::create_military_patrol_event(),
        );

        Self { events }
    }

    /// Check if an event should trigger for a given phase
    pub fn should_trigger_event(&self, phase: &EventPhase, destination: &StarSystem) -> bool {
        let base_probability = match phase {
            EventPhase::Jump => 0.10,     // 10% base chance for jump events
            EventPhase::Approach => 0.12, // 12% base chance for approach events
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
            GovernmentType::Anarchy => modifier *= 1.3, // More events in lawless space
            GovernmentType::Communist => modifier *= 1.2, // More inspections
            GovernmentType::CorporateState => modifier *= 0.9, // Fewer events in corporate space
            GovernmentType::Democracy => modifier *= 1.0, // Standard
            GovernmentType::Dictatorship => modifier *= 1.1, // More military presence
            GovernmentType::Feudal => modifier *= 1.0,  // Standard
            GovernmentType::MultiGovernment => modifier *= 1.0, // Standard
            GovernmentType::Confederacy => modifier *= 1.0, // Standard
        }

        // Economy-based modifiers
        match destination.economy {
            EconomyType::RichIndustrial => modifier *= 0.8, // Fewer events in rich space
            EconomyType::PoorAgricultural => modifier *= 1.4, // More desperate encounters
            EconomyType::MainlyAgricultural => modifier *= 1.1, // Slightly more agricultural events
            EconomyType::MainlyIndustrial => modifier *= 1.1, // Slightly more industrial events
            EconomyType::PoorIndustrial => modifier *= 1.2, // More industrial accidents
            EconomyType::AverageIndustrial => modifier *= 1.0, // Standard
            EconomyType::RichAgricultural => modifier *= 0.9, // Fewer events in rich agricultural
            EconomyType::AverageAgricultural => modifier *= 1.0, // Standard
        }

        modifier
    }

    /// Get available events for a given phase
    pub fn get_available_events(&self, phase: &EventPhase) -> Vec<&Event> {
        self.events
            .values()
            .filter(|event| event.phase == *phase)
            .collect()
    }

    /// Select a random event for a given phase, weighted by the destination's government and economy
    pub fn select_random_event(
        &self,
        phase: &EventPhase,
        destination: &StarSystem,
    ) -> Option<&Event> {
        self.pick_event(phase, destination, &mut rand::thread_rng())
    }

    fn pick_event<R: Rng>(
        &self,
        phase: &EventPhase,
        destination: &StarSystem,
        rng: &mut R,
    ) -> Option<&Event> {
        let mut available_events = self.get_available_events(phase);
        available_events.sort_by_key(|event| format!("{:?}", event.event_type));

        let weights: Vec<f64> = available_events
            .iter()
            .map(|event| event_weight(&event.event_type, destination))
            .collect();
        let total: f64 = weights.iter().sum();
        if available_events.is_empty() || total <= 0.0 {
            return None;
        }

        let mut roll = rng.gen_range(0.0..total);
        for (event, weight) in available_events.iter().zip(&weights) {
            if roll < *weight {
                return Some(event);
            }
            roll -= weight;
        }
        available_events.last().copied()
    }

    /// Process an event and return the outcome
    pub fn process_event(
        &self,
        event: &Event,
        commander: &mut Commander,
        ship: &mut Ship,
        choice_index: usize,
    ) -> Result<EventOutcome, String> {
        if choice_index >= event.choices.len() {
            return Err("Invalid choice index".to_string());
        }

        let choice = &event.choices[choice_index];
        let outcome = &choice.outcome;

        // Apply outcome effects
        commander.credits = (commander.credits as i32 + outcome.credits_change).max(0) as u32;

        if let Some(loss) = &outcome.cargo_loss {
            let taken = apply_cargo_loss(ship, loss);
            if taken.is_empty() {
                show_info("You had no cargo to lose.");
            }
            for (commodity, tons) in taken {
                show_info(&format!("Lost {} tons of {}", tons, commodity.name()));
            }
        }

        if let Some(gain) = &outcome.cargo_gain {
            let added = apply_cargo_gain(ship, gain);
            if added == 0 {
                show_info("No room in your hold for the cargo.");
            } else {
                show_info(&format!(
                    "Gained {} tons of {}",
                    added,
                    gain.commodity.name()
                ));
            }
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
            description:
                "A mysterious wormhole has appeared in your path. It could lead anywhere..."
                    .to_string(),
            choices: vec![
                EventChoice {
                    description: "Enter the wormhole".to_string(),
                    outcome: EventOutcome {
                        description: "You enter the wormhole and emerge in an unknown system"
                            .to_string(),
                        credits_change: 0,
                        cargo_loss: None,
                        cargo_gain: None,
                        hull_damage: 0,
                        fuel_change: -2,
                        reputation_change: 0,
                        starts_combat: false,
                        new_location: Some("UNKNOWN".to_string()),
                        message: "The wormhole has transported you to an unknown location!"
                            .to_string(),
                    },
                },
                EventChoice {
                    description: "Avoid the wormhole".to_string(),
                    outcome: EventOutcome {
                        description: "You carefully navigate around the wormhole".to_string(),
                        credits_change: 0,
                        cargo_loss: None,
                        cargo_gain: None,
                        hull_damage: 0,
                        fuel_change: -1,
                        reputation_change: 0,
                        starts_combat: false,
                        new_location: None,
                        message: "You safely avoid the wormhole and continue your journey."
                            .to_string(),
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
            description: "Your sensors detect an unusual energy signature in hyperspace."
                .to_string(),
            choices: vec![
                EventChoice {
                    description: "Investigate the anomaly".to_string(),
                    outcome: EventOutcome {
                        description: "You investigate and find rare materials".to_string(),
                        credits_change: 500,
                        cargo_loss: None,
                        cargo_gain: Some(CargoGain {
                            commodity: CommodityType::Computers,
                            tons: 2,
                        }),
                        hull_damage: -5,
                        fuel_change: -1,
                        reputation_change: 10,
                        starts_combat: false,
                        new_location: None,
                        message: "You discover valuable materials! +500 credits, +2 tons of Computers, +10 reputation"
                            .to_string(),
                    },
                },
                EventChoice {
                    description: "Avoid the anomaly".to_string(),
                    outcome: EventOutcome {
                        description: "You safely avoid the anomaly".to_string(),
                        credits_change: 0,
                        cargo_loss: None,
                        cargo_gain: None,
                        hull_damage: 0,
                        fuel_change: 0,
                        reputation_change: 0,
                        starts_combat: false,
                        new_location: None,
                        message: "You safely avoid the anomaly and continue your journey."
                            .to_string(),
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
                        cargo_gain: None,
                        hull_damage: 0,
                        fuel_change: -2,
                        reputation_change: 0,
                        starts_combat: false,
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
                        cargo_gain: None,
                        hull_damage: 0,
                        fuel_change: -3,
                        reputation_change: -5,
                        starts_combat: false,
                        new_location: None,
                        message: "You arrive at your destination but used more fuel than expected."
                            .to_string(),
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
                        cargo_gain: Some(CargoGain {
                            commodity: CommodityType::Minerals,
                            tons: 3,
                        }),
                        hull_damage: -2,
                        fuel_change: 0,
                        reputation_change: 0,
                        starts_combat: false,
                        new_location: None,
                        message: "You salvage valuable materials! +200 credits, +3 tons of Minerals, minor hull damage."
                            .to_string(),
                    },
                },
                EventChoice {
                    description: "Avoid the debris".to_string(),
                    outcome: EventOutcome {
                        description: "You carefully navigate around the debris".to_string(),
                        credits_change: 0,
                        cargo_loss: None,
                        cargo_gain: None,
                        hull_damage: 0,
                        fuel_change: 0,
                        reputation_change: 0,
                        starts_combat: false,
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
                        cargo_gain: None,
                        hull_damage: 0,
                        fuel_change: 0,
                        reputation_change: 15,
                        starts_combat: false,
                        new_location: None,
                        message: "The aliens are friendly! +300 credits, +15 reputation."
                            .to_string(),
                    },
                },
                EventChoice {
                    description: "Flee".to_string(),
                    outcome: EventOutcome {
                        description: "You flee from the alien vessel".to_string(),
                        credits_change: 0,
                        cargo_loss: None,
                        cargo_gain: None,
                        hull_damage: -3,
                        fuel_change: -2,
                        reputation_change: -5,
                        starts_combat: false,
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
                        cargo_gain: None,
                        hull_damage: -8,
                        fuel_change: 0,
                        reputation_change: -10,
                        starts_combat: false,
                        new_location: None,
                        message: "You win the battle! +400 credits, but significant hull damage."
                            .to_string(),
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
                        credits_change: 0,
                        cargo_loss: None,
                        cargo_gain: None,
                        hull_damage: 0,
                        fuel_change: 0,
                        reputation_change: 0,
                        starts_combat: true,
                        new_location: None,
                        message: "The pirates open fire!"
                            .to_string(),
                    },
                },
                EventChoice {
                    description: "Flee".to_string(),
                    outcome: EventOutcome {
                        description: "You attempt to flee from the pirates".to_string(),
                        credits_change: 0,
                        cargo_loss: Some(CargoLoss::Tons(3)),
                        cargo_gain: None,
                        hull_damage: -5,
                        fuel_change: -3,
                        reputation_change: -5,
                        starts_combat: false,
                        new_location: None,
                        message: "You escape but lose up to 3 tons of cargo and take damage.".to_string(),
                    },
                },
                EventChoice {
                    description: "Surrender".to_string(),
                    outcome: EventOutcome {
                        description: "You surrender to the pirates".to_string(),
                        credits_change: -200,
                        cargo_loss: Some(CargoLoss::Tons(5)),
                        cargo_gain: None,
                        hull_damage: 0,
                        fuel_change: 0,
                        reputation_change: -10,
                        starts_combat: false,
                        new_location: None,
                        message: "You surrender and lose up to 5 tons of cargo and 200 credits, but avoid damage."
                            .to_string(),
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
                        cargo_gain: Some(CargoGain {
                            commodity: CommodityType::Luxury,
                            tons: 2,
                        }),
                        hull_damage: 0,
                        fuel_change: 0,
                        reputation_change: 20,
                        starts_combat: false,
                        new_location: None,
                        message: "You rescue the crew! +400 credits, +2 tons of Luxury as thanks, +20 reputation.".to_string(),
                    },
                },
                EventChoice {
                    description: "Ignore the signal".to_string(),
                    outcome: EventOutcome {
                        description: "You ignore the distress signal".to_string(),
                        credits_change: 0,
                        cargo_loss: None,
                        cargo_gain: None,
                        hull_damage: 0,
                        fuel_change: 0,
                        reputation_change: -5,
                        starts_combat: false,
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
            description: "Government officials are requesting an inspection of your vessel."
                .to_string(),
            choices: vec![
                EventChoice {
                    description: "Comply with inspection".to_string(),
                    outcome: EventOutcome {
                        description: "You comply with the government inspection".to_string(),
                        credits_change: -50,
                        cargo_loss: None,
                        cargo_gain: None,
                        hull_damage: 0,
                        fuel_change: 0,
                        reputation_change: 10,
                        starts_combat: false,
                        new_location: None,
                        message: "Inspection completed. -50 credits for fees, +10 reputation."
                            .to_string(),
                    },
                },
                EventChoice {
                    description: "Attempt to bribe".to_string(),
                    outcome: EventOutcome {
                        description: "You attempt to bribe the officials".to_string(),
                        credits_change: -100,
                        cargo_loss: None,
                        cargo_gain: None,
                        hull_damage: 0,
                        fuel_change: 0,
                        reputation_change: -5,
                        starts_combat: false,
                        new_location: None,
                        message: "Bribe successful. -100 credits, minor reputation loss."
                            .to_string(),
                    },
                },
                EventChoice {
                    description: "Resist inspection".to_string(),
                    outcome: EventOutcome {
                        description: "You resist the government inspection".to_string(),
                        credits_change: 0,
                        cargo_loss: Some(CargoLoss::All(CommodityType::Narcotics)),
                        cargo_gain: None,
                        hull_damage: 0,
                        fuel_change: 0,
                        reputation_change: -15,
                        starts_combat: false,
                        new_location: None,
                        message: "You resist but lose any illegal cargo and reputation."
                            .to_string(),
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
                        cargo_gain: None,
                        hull_damage: 0,
                        fuel_change: 0,
                        reputation_change: 5,
                        starts_combat: false,
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
                        cargo_gain: None,
                        hull_damage: -3,
                        fuel_change: -2,
                        reputation_change: -10,
                        starts_combat: false,
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
                        cargo_gain: None,
                        hull_damage: -15,
                        fuel_change: 0,
                        reputation_change: -20,
                        starts_combat: false,
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
    use crate::data::systems::{EconomyType, GovernmentType, StarSystem};

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
        manager
            .process_event(event, &mut commander, &mut ship, 0)
            .unwrap();

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
            manager
                .process_event(event, &mut commander, &mut ship, index)
                .unwrap();
            if commander.reputation < before {
                return;
            }
        }
        panic!("no military patrol choice lowers reputation");
    }

    fn ship_with(cargo: &[(CommodityType, u32)]) -> Ship {
        let mut ship = Ship::new_cobra_mk3();
        for (commodity, tons) in cargo {
            assert!(ship.cargo_hold.add_cargo(commodity.clone(), *tons));
        }
        ship
    }

    #[test]
    fn losing_tons_takes_from_the_largest_cargo_first() {
        let mut ship = ship_with(&[(CommodityType::Food, 2), (CommodityType::Minerals, 8)]);

        let taken = apply_cargo_loss(&mut ship, &CargoLoss::Tons(5));

        assert_eq!(taken, vec![(CommodityType::Minerals, 5)]);
        assert_eq!(ship.cargo_hold.get_quantity(&CommodityType::Minerals), 3);
        assert_eq!(ship.cargo_hold.get_quantity(&CommodityType::Food), 2);
    }

    #[test]
    fn losing_more_than_held_takes_everything_across_commodities() {
        let mut ship = ship_with(&[(CommodityType::Food, 2), (CommodityType::Minerals, 3)]);

        let taken = apply_cargo_loss(&mut ship, &CargoLoss::Tons(10));

        assert_eq!(taken.iter().map(|(_, tons)| tons).sum::<u32>(), 5);
        assert!(ship.cargo_hold.is_empty());
    }

    #[test]
    fn losing_cargo_from_an_empty_hold_takes_nothing() {
        let mut ship = ship_with(&[]);
        assert!(apply_cargo_loss(&mut ship, &CargoLoss::Tons(3)).is_empty());
        assert!(apply_cargo_loss(&mut ship, &CargoLoss::All(CommodityType::Narcotics)).is_empty());
    }

    #[test]
    fn losing_all_of_a_commodity_leaves_other_cargo() {
        let mut ship = ship_with(&[(CommodityType::Narcotics, 4), (CommodityType::Food, 2)]);

        let taken = apply_cargo_loss(&mut ship, &CargoLoss::All(CommodityType::Narcotics));

        assert_eq!(taken, vec![(CommodityType::Narcotics, 4)]);
        assert_eq!(ship.cargo_hold.get_quantity(&CommodityType::Narcotics), 0);
        assert_eq!(ship.cargo_hold.get_quantity(&CommodityType::Food), 2);
    }

    #[test]
    fn cargo_gain_is_limited_by_free_space() {
        let mut ship = ship_with(&[(CommodityType::Food, 19)]);
        let gain = CargoGain {
            commodity: CommodityType::Luxury,
            tons: 2,
        };

        assert_eq!(apply_cargo_gain(&mut ship, &gain), 1);
        assert_eq!(ship.cargo_hold.get_quantity(&CommodityType::Luxury), 1);
        assert_eq!(apply_cargo_gain(&mut ship, &gain), 0);
    }

    #[test]
    fn surrendering_to_pirates_costs_cargo_and_credits() {
        let manager = EventManager::new();
        let mut commander = Commander::new("Test".to_string());
        commander.credits = 500;
        let mut ship = ship_with(&[(CommodityType::Computers, 8)]);

        let event = manager.events.get(&EventType::PirateAttack).unwrap();
        let surrender = event
            .choices
            .iter()
            .position(|c| c.description == "Surrender")
            .unwrap();
        manager
            .process_event(event, &mut commander, &mut ship, surrender)
            .unwrap();

        assert_eq!(ship.cargo_hold.get_quantity(&CommodityType::Computers), 3);
        assert_eq!(commander.credits, 300);
    }

    #[test]
    fn resisting_inspection_confiscates_narcotics_only() {
        let manager = EventManager::new();
        let mut commander = Commander::new("Test".to_string());
        let mut ship = ship_with(&[(CommodityType::Narcotics, 3), (CommodityType::Food, 5)]);

        let event = manager
            .events
            .get(&EventType::GovernmentInspection)
            .unwrap();
        let resist = event
            .choices
            .iter()
            .position(|c| c.description.starts_with("Resist"))
            .unwrap();
        manager
            .process_event(event, &mut commander, &mut ship, resist)
            .unwrap();

        assert_eq!(ship.cargo_hold.get_quantity(&CommodityType::Narcotics), 0);
        assert_eq!(ship.cargo_hold.get_quantity(&CommodityType::Food), 5);
    }

    #[test]
    fn rescuing_the_crew_rewards_cargo() {
        let manager = EventManager::new();
        let mut commander = Commander::new("Test".to_string());
        let mut ship = ship_with(&[]);

        let event = manager.events.get(&EventType::RescueSignal).unwrap();
        manager
            .process_event(event, &mut commander, &mut ship, 0)
            .unwrap();

        assert_eq!(ship.cargo_hold.get_quantity(&CommodityType::Luxury), 2);
    }

    fn system(government: GovernmentType, economy: EconomyType) -> StarSystem {
        StarSystem {
            name: "Test".to_string(),
            x: 0,
            y: 0,
            economy,
            government,
            tech_level: 5,
            turnover: 1000,
            radius: 100,
            population: "1000000".to_string(),
            description: "Test system".to_string(),
        }
    }

    fn share_of(event_type: EventType, phase: EventPhase, destination: &StarSystem) -> f64 {
        use rand::rngs::StdRng;
        use rand::SeedableRng;

        let manager = EventManager::new();
        let mut rng = StdRng::seed_from_u64(7);
        let draws = 20_000;
        let hits = (0..draws)
            .filter(|_| {
                manager
                    .pick_event(&phase, destination, &mut rng)
                    .unwrap()
                    .event_type
                    == event_type
            })
            .count();
        hits as f64 / f64::from(draws)
    }

    #[test]
    fn anarchy_attracts_pirates_more_than_democracy() {
        let anarchy = system(GovernmentType::Anarchy, EconomyType::AverageAgricultural);
        let democracy = system(GovernmentType::Democracy, EconomyType::AverageAgricultural);

        let lawless = share_of(EventType::PirateAttack, EventPhase::Approach, &anarchy);
        let ordered = share_of(EventType::PirateAttack, EventPhase::Approach, &democracy);

        assert!(lawless > ordered * 1.8, "{} vs {}", lawless, ordered);
    }

    #[test]
    fn communist_systems_inspect_more_often() {
        let communist = system(GovernmentType::Communist, EconomyType::AverageAgricultural);
        let democracy = system(GovernmentType::Democracy, EconomyType::AverageAgricultural);

        let strict = share_of(
            EventType::GovernmentInspection,
            EventPhase::Approach,
            &communist,
        );
        let normal = share_of(
            EventType::GovernmentInspection,
            EventPhase::Approach,
            &democracy,
        );

        assert!(strict > normal * 1.8, "{} vs {}", strict, normal);
    }

    #[test]
    fn dictatorships_send_more_patrols() {
        let dictatorship = system(
            GovernmentType::Dictatorship,
            EconomyType::AverageAgricultural,
        );
        let democracy = system(GovernmentType::Democracy, EconomyType::AverageAgricultural);

        let patrolled = share_of(
            EventType::MilitaryPatrol,
            EventPhase::Approach,
            &dictatorship,
        );
        let normal = share_of(EventType::MilitaryPatrol, EventPhase::Approach, &democracy);

        assert!(patrolled > normal * 1.8, "{} vs {}", patrolled, normal);
    }

    #[test]
    fn poor_economies_have_more_pirates_and_rich_ones_fewer() {
        let poor = system(GovernmentType::Democracy, EconomyType::PoorAgricultural);
        let average = system(GovernmentType::Democracy, EconomyType::AverageAgricultural);
        let rich = system(GovernmentType::Democracy, EconomyType::RichIndustrial);

        let poor_share = share_of(EventType::PirateAttack, EventPhase::Approach, &poor);
        let average_share = share_of(EventType::PirateAttack, EventPhase::Approach, &average);
        let rich_share = share_of(EventType::PirateAttack, EventPhase::Approach, &rich);

        assert!(poor_share > average_share && average_share > rich_share);
    }

    #[test]
    fn mainly_industrial_systems_have_more_debris() {
        let industrial = system(GovernmentType::Democracy, EconomyType::MainlyIndustrial);
        let average = system(GovernmentType::Democracy, EconomyType::AverageAgricultural);

        let debris = share_of(EventType::SpaceDebris, EventPhase::Jump, &industrial);
        let normal = share_of(EventType::SpaceDebris, EventPhase::Jump, &average);

        assert!(debris > normal);
    }

    #[test]
    fn every_event_stays_possible_everywhere() {
        let manager = EventManager::new();
        for government in [
            GovernmentType::Anarchy,
            GovernmentType::Communist,
            GovernmentType::CorporateState,
            GovernmentType::Democracy,
            GovernmentType::Dictatorship,
            GovernmentType::Feudal,
            GovernmentType::MultiGovernment,
            GovernmentType::Confederacy,
        ] {
            let destination = system(government, EconomyType::RichIndustrial);
            for phase in [EventPhase::Jump, EventPhase::Approach] {
                for event in manager.get_available_events(&phase) {
                    assert!(event_weight(&event.event_type, &destination) > 0.0);
                }
            }
        }
    }

    #[test]
    fn picking_an_event_always_matches_the_phase() {
        let manager = EventManager::new();
        let destination = system(GovernmentType::Anarchy, EconomyType::PoorAgricultural);
        for _ in 0..200 {
            let event = manager
                .select_random_event(&EventPhase::Jump, &destination)
                .unwrap();
            assert_eq!(event.phase, EventPhase::Jump);
        }
    }

    #[test]
    fn only_fighting_pirates_starts_combat() {
        let manager = EventManager::new();
        let mut starters = Vec::new();
        for (event_type, event) in &manager.events {
            for choice in &event.choices {
                if choice.outcome.starts_combat {
                    starters.push((event_type.clone(), choice.description.clone()));
                }
            }
        }

        assert_eq!(
            starters,
            vec![(EventType::PirateAttack, "Fight the pirates".to_string())]
        );
    }

    #[test]
    fn fighting_pirates_has_no_instant_effects() {
        let manager = EventManager::new();
        let mut commander = Commander::new("Test".to_string());
        let mut ship = ship_with(&[]);
        let event = manager.events.get(&EventType::PirateAttack).unwrap();

        let outcome = manager
            .process_event(event, &mut commander, &mut ship, 0)
            .unwrap();

        assert!(outcome.starts_combat);
        assert_eq!(commander.credits, 100);
        assert_eq!(commander.reputation, 0);
        assert_eq!(ship.hull_percentage, 100);
    }
}
