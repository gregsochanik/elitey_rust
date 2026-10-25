use crate::data::commodities::CommodityType;
use crate::data::systems::{EconomyType, GovernmentType, StarSystem};
use crate::game::commander::Standing;
use crate::game::encounters;
use crate::game::{Commander, Ship};
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
    TraderConvoy,
    BountyHunter,
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

/// Where an event drops the ship instead of the planned destination
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Redirect {
    /// Any system other than the origin and the planned destination
    Anywhere,
    /// One of the few systems closest to the planned destination
    NearDestination,
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
    #[serde(default)]
    pub redirect: Option<Redirect>,
    pub message: String,
}

/// What applying an event outcome did, beyond the credits, hull and fuel it states
#[derive(Debug, Clone)]
pub struct EventReport {
    pub outcome: EventOutcome,
    /// Cargo taken from the hold (empty when there was nothing to take)
    pub cargo_lost: Option<Vec<(CommodityType, u32)>>,
    pub cargo_gained: Option<(CommodityType, u32)>,
    pub no_room_for_cargo: bool,
    pub new_standing: Option<Standing>,
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
fn event_weight(event_type: &EventType, destination: &StarSystem, standing: Standing) -> f64 {
    encounters::weight(event_type, destination, standing)
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
            ship.cargo_hold.release(remaining);
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

const MIN_CREDITS_ROLL: f64 = 0.6;
const MAX_CREDITS_ROLL: f64 = 1.4;
const WINDFALL_BASE: i32 = 200;
const WINDFALL_CAP_FLOOR: u32 = 100;

/// Roll an outcome's credits and cargo tons within their ranges and fill in the message
pub(crate) fn roll_outcome<R: Rng>(
    outcome: &EventOutcome,
    commander_credits: u32,
    rng: &mut R,
) -> EventOutcome {
    let mut rolled = outcome.clone();

    if outcome.credits_change != 0 {
        let roll = rng.gen_range(MIN_CREDITS_ROLL..=MAX_CREDITS_ROLL);
        let mut credits = (f64::from(outcome.credits_change) * roll).round() as i32;
        if outcome.credits_change >= WINDFALL_BASE {
            let cap = WINDFALL_CAP_FLOOR + commander_credits / 2;
            credits = credits.min(cap as i32);
        }
        rolled.credits_change = credits;
    }

    let mut tons = 0;
    if let Some(gain) = &mut rolled.cargo_gain {
        gain.tons = roll_tons(gain.tons, rng);
        tons = gain.tons;
    }
    if let Some(CargoLoss::Tons(base)) = &mut rolled.cargo_loss {
        *base = roll_tons(*base, rng);
        tons = *base;
    }

    rolled.message = outcome
        .message
        .replace(
            "{credits}",
            &rolled.credits_change.unsigned_abs().to_string(),
        )
        .replace("{tons}", &tons.to_string());
    rolled
}

fn roll_tons<R: Rng>(base: u32, rng: &mut R) -> u32 {
    rng.gen_range(base.saturating_sub(1).max(1)..=base + 1)
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
        events.insert(EventType::TraderConvoy, Self::create_trader_convoy_event());
        events.insert(EventType::BountyHunter, Self::create_bounty_hunter_event());

        Self { events }
    }

    /// Chance that an event triggers in this phase at this destination
    pub fn trigger_probability(&self, phase: &EventPhase, destination: &StarSystem) -> f64 {
        let base_probability = match phase {
            EventPhase::Jump => 0.10,     // 10% base chance for jump events
            EventPhase::Approach => 0.12, // 12% base chance for approach events
        };

        base_probability * self.calculate_contextual_modifier(destination)
    }

    /// Check if an event should trigger for a given phase
    pub fn should_trigger_event(&self, phase: &EventPhase, destination: &StarSystem) -> bool {
        self.should_trigger_event_with(phase, destination, &mut rand::thread_rng())
    }

    pub fn should_trigger_event_with<R: Rng>(
        &self,
        phase: &EventPhase,
        destination: &StarSystem,
        rng: &mut R,
    ) -> bool {
        rng.gen_bool(self.trigger_probability(phase, destination))
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
        standing: Standing,
    ) -> Option<&Event> {
        self.pick_event(phase, destination, standing, &mut rand::thread_rng())
    }

    pub fn pick_event<R: Rng>(
        &self,
        phase: &EventPhase,
        destination: &StarSystem,
        standing: Standing,
        rng: &mut R,
    ) -> Option<&Event> {
        let mut available_events = self.get_available_events(phase);
        available_events.sort_by_key(|event| format!("{:?}", event.event_type));

        let weights: Vec<f64> = available_events
            .iter()
            .map(|event| event_weight(&event.event_type, destination, standing))
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
        self.process_event_with(
            event,
            commander,
            ship,
            choice_index,
            &mut rand::thread_rng(),
        )
    }

    pub fn process_event_with<R: Rng>(
        &self,
        event: &Event,
        commander: &mut Commander,
        ship: &mut Ship,
        choice_index: usize,
        rng: &mut R,
    ) -> Result<EventOutcome, String> {
        self.process_event_reported(event, commander, ship, choice_index, rng)
            .map(|report| report.outcome)
    }

    /// Apply a choice's rolled outcome and report what happened to the cargo and standing
    pub fn process_event_reported<R: Rng>(
        &self,
        event: &Event,
        commander: &mut Commander,
        ship: &mut Ship,
        choice_index: usize,
        rng: &mut R,
    ) -> Result<EventReport, String> {
        if choice_index >= event.choices.len() {
            return Err("Invalid choice index".to_string());
        }

        let choice = &event.choices[choice_index];
        let rolled = roll_outcome(&choice.outcome, commander.credits, rng);
        let outcome = &rolled;
        let mut report = EventReport {
            outcome: rolled.clone(),
            cargo_lost: None,
            cargo_gained: None,
            no_room_for_cargo: false,
            new_standing: None,
        };

        commander.credits = (commander.credits as i32 + outcome.credits_change).max(0) as u32;

        if let Some(loss) = &outcome.cargo_loss {
            report.cargo_lost = Some(apply_cargo_loss(ship, loss));
        }

        if let Some(gain) = &outcome.cargo_gain {
            match apply_cargo_gain(ship, gain) {
                0 => report.no_room_for_cargo = true,
                added => report.cargo_gained = Some((gain.commodity.clone(), added)),
            }
        }

        report.new_standing = commander
            .change_reputation(outcome.reputation_change)
            .new_standing;

        ship.take_damage(outcome.hull_damage.unsigned_abs());
        ship.fuel_current = (ship.fuel_current as i32 + outcome.fuel_change).max(0) as u32;

        Ok(report)
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
                        redirect: Some(Redirect::Anywhere),
                        message: "The wormhole spits you out far from your course!".to_string(),
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
                        redirect: None,
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
                        credits_change: 200,
                        cargo_loss: None,
                        cargo_gain: Some(CargoGain {
                            commodity: CommodityType::Computers,
                            tons: 2,
                        }),
                        hull_damage: -5,
                        fuel_change: -1,
                        reputation_change: 10,
                        starts_combat: false,
                        redirect: None,
                        message: "You discover valuable materials! +{credits} credits, +{tons} tons of Computers, +10 reputation"
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
                        redirect: None,
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
                        redirect: None,
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
                        redirect: Some(Redirect::NearDestination),
                        message: "You drift off course and use more fuel than expected."
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
                        credits_change: 100,
                        cargo_loss: None,
                        cargo_gain: Some(CargoGain {
                            commodity: CommodityType::Minerals,
                            tons: 3,
                        }),
                        hull_damage: -2,
                        fuel_change: 0,
                        reputation_change: 0,
                        starts_combat: false,
                        redirect: None,
                        message: "You salvage valuable materials! +{credits} credits, +{tons} tons of Minerals, minor hull damage."
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
                        redirect: None,
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
                        credits_change: 150,
                        cargo_loss: None,
                        cargo_gain: None,
                        hull_damage: 0,
                        fuel_change: 0,
                        reputation_change: 15,
                        starts_combat: false,
                        redirect: None,
                        message: "The aliens are friendly! +{credits} credits, +15 reputation."
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
                        redirect: None,
                        message: "You escape but take some damage and use extra fuel.".to_string(),
                    },
                },
                EventChoice {
                    description: "Fight".to_string(),
                    outcome: EventOutcome {
                        description: "You engage in combat with the aliens".to_string(),
                        credits_change: 200,
                        cargo_loss: None,
                        cargo_gain: None,
                        hull_damage: -8,
                        fuel_change: 0,
                        reputation_change: -10,
                        starts_combat: false,
                        redirect: None,
                        message:
                            "You win the battle! +{credits} credits, but significant hull damage."
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
                        redirect: None,
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
                        redirect: None,
                        message: "You escape but lose up to {tons} tons of cargo and take damage.".to_string(),
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
                        redirect: None,
                        message: "You surrender and lose up to {tons} tons of cargo and {credits} credits, but avoid damage."
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
                        credits_change: 200,
                        cargo_loss: None,
                        cargo_gain: Some(CargoGain {
                            commodity: CommodityType::Luxury,
                            tons: 2,
                        }),
                        hull_damage: 0,
                        fuel_change: 0,
                        reputation_change: 20,
                        starts_combat: false,
                        redirect: None,
                        message: "You rescue the crew! +{credits} credits, +{tons} tons of Luxury as thanks, +20 reputation.".to_string(),
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
                        redirect: None,
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
                        redirect: None,
                        message:
                            "Inspection completed. -{credits} credits for fees, +10 reputation."
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
                        redirect: None,
                        message: "Bribe successful. -{credits} credits, minor reputation loss."
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
                        redirect: None,
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
                        redirect: None,
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
                        redirect: None,
                        message: "You evade but take damage and use extra fuel.".to_string(),
                    },
                },
                EventChoice {
                    description: "Engage the patrol".to_string(),
                    outcome: EventOutcome {
                        description: "You engage in combat with the military patrol".to_string(),
                        credits_change: 0,
                        cargo_loss: None,
                        cargo_gain: None,
                        hull_damage: 0,
                        fuel_change: 0,
                        reputation_change: -20,
                        starts_combat: true,
                        redirect: None,
                        message: "You open fire on the patrol. Reputation loss.".to_string(),
                    },
                },
            ],
        }
    }

    fn create_trader_convoy_event() -> Event {
        let quiet = |description: &str, message: &str| EventOutcome {
            description: description.to_string(),
            credits_change: 0,
            cargo_loss: None,
            cargo_gain: None,
            hull_damage: 0,
            fuel_change: 0,
            reputation_change: 0,
            starts_combat: false,
            redirect: None,
            message: message.to_string(),
        };
        Event {
            event_type: EventType::TraderConvoy,
            phase: EventPhase::Approach,
            title: "💼 Trader Convoy".to_string(),
            description: "A merchant convoy is hailing you, offering to trade.".to_string(),
            choices: vec![
                EventChoice {
                    description: "Trade with the convoy".to_string(),
                    outcome: EventOutcome {
                        credits_change: -50,
                        cargo_gain: Some(CargoGain {
                            commodity: CommodityType::Textiles,
                            tons: 3,
                        }),
                        ..quiet(
                            "You swap credits for textiles",
                            "You pay 50 credits for 3 tons of textiles.",
                        )
                    },
                },
                EventChoice {
                    description: "Buy fuel from the convoy".to_string(),
                    outcome: EventOutcome {
                        credits_change: -20,
                        fuel_change: 2,
                        ..quiet(
                            "You top up your tank",
                            "You pay 20 credits for 2 light years of fuel.",
                        )
                    },
                },
                EventChoice {
                    description: "Rob the convoy".to_string(),
                    outcome: EventOutcome {
                        reputation_change: -10,
                        starts_combat: true,
                        ..quiet(
                            "You draw your weapons on the convoy",
                            "You open fire on the convoy. Reputation loss.",
                        )
                    },
                },
                EventChoice {
                    description: "Wave and move on".to_string(),
                    outcome: quiet(
                        "You wave and fly on",
                        "The convoy wishes you a safe journey.",
                    ),
                },
            ],
        }
    }

    fn create_bounty_hunter_event() -> Event {
        let outcome = |description: &str, message: &str| EventOutcome {
            description: description.to_string(),
            credits_change: 0,
            cargo_loss: None,
            cargo_gain: None,
            hull_damage: 0,
            fuel_change: 0,
            reputation_change: 0,
            starts_combat: false,
            redirect: None,
            message: message.to_string(),
        };
        Event {
            event_type: EventType::BountyHunter,
            phase: EventPhase::Approach,
            title: "🎯 Bounty Hunter".to_string(),
            description:
                "A bounty hunter has locked onto your ship. Your reputation has caught up with you."
                    .to_string(),
            choices: vec![
                EventChoice {
                    description: "Fight the hunter".to_string(),
                    outcome: EventOutcome {
                        starts_combat: true,
                        ..outcome("You turn to face the hunter", "You turn and open fire.")
                    },
                },
                EventChoice {
                    description: "Pay them to look the other way".to_string(),
                    outcome: EventOutcome {
                        credits_change: -250,
                        ..outcome(
                            "You pay the hunter off",
                            "The hunter takes {credits} credits and lets you go.",
                        )
                    },
                },
                EventChoice {
                    description: "Try to outrun them".to_string(),
                    outcome: EventOutcome {
                        hull_damage: -10,
                        fuel_change: -2,
                        ..outcome(
                            "You push the engines",
                            "You shake the hunter but take hits and burn extra fuel.",
                        )
                    },
                },
            ],
        }
    }
}

#[cfg(test)]
mod tests;
