use crate::data::systems::{EconomyType, GovernmentType, StarSystem};
use crate::game::commander::{CombatRating, Standing};
use crate::game::events::EventType;
use rand::Rng;
use serde::Deserialize;
use std::sync::OnceLock;

/// One line of `data/encounters.json`: every rule whose conditions all hold multiplies the
/// chance of its events
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Rule {
    events: Vec<EventType>,
    government: Option<Vec<GovernmentType>>,
    economy: Option<Vec<EconomyType>>,
    standing: Option<Vec<Standing>>,
    lawful: Option<bool>,
    weight: f64,
}

/// A chance of a further enemy joining the fight when the conditions hold
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct GroupRule {
    events: Vec<EventType>,
    government: Option<Vec<GovernmentType>>,
    economy: Option<Vec<EconomyType>>,
    extra_chance: f64,
}

/// Tougher but more nervous enemies as the commander's combat rating rises
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct DifficultyRule {
    enemies: Vec<String>,
    health_per_rating: f64,
    nerve_per_rating: f64,
}

#[derive(Deserialize)]
struct Table {
    rules: Vec<Rule>,
    groups: Vec<GroupRule>,
    difficulty: DifficultyRule,
}

fn table() -> &'static Table {
    static TABLE: OnceLock<Table> = OnceLock::new();
    TABLE.get_or_init(|| {
        serde_json::from_str(include_str!("../../data/encounters.json"))
            .expect("data/encounters.json is valid")
    })
}

fn rules() -> &'static [Rule] {
    &table().rules
}

/// The most enemies beyond the first that can join one encounter
const MAX_EXTRA_FOES: u32 = 2;

/// How many more enemies follow the first: each further one is half as likely as the one before
pub fn extra_foes(event_type: &EventType, destination: &StarSystem, rng: &mut impl Rng) -> u32 {
    let chance: f64 = table()
        .groups
        .iter()
        .filter(|rule| rule.events.contains(event_type))
        .filter(|rule| {
            rule.government
                .as_ref()
                .is_none_or(|list| list.contains(&destination.government))
        })
        .filter(|rule| {
            rule.economy
                .as_ref()
                .is_none_or(|list| list.contains(&destination.economy))
        })
        .map(|rule| rule.extra_chance)
        .sum::<f64>()
        .min(0.8);
    let mut extra = 0;
    let mut next = chance;
    while extra < MAX_EXTRA_FOES && rng.gen_bool(next) {
        extra += 1;
        next /= 2.0;
    }
    extra
}

/// How an enemy is toughened for a commander of this rating
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Difficulty {
    /// Multiplies the enemy's hull and shield
    pub health: f64,
    /// Subtracted from the enemy's aggression, so it breaks off sooner
    pub nerve: f64,
}

impl Difficulty {
    pub const NONE: Difficulty = Difficulty {
        health: 1.0,
        nerve: 0.0,
    };
}

pub fn difficulty(enemy_id: &str, rating: CombatRating) -> Difficulty {
    let rule = &table().difficulty;
    if !rule.enemies.iter().any(|id| id == enemy_id) {
        return Difficulty::NONE;
    }
    let level = f64::from(rating as u32);
    Difficulty {
        health: 1.0 + rule.health_per_rating * level,
        nerve: rule.nerve_per_rating * level,
    }
}

/// How much more or less likely an event is at this destination for this commander
/// (1.0 is normal, 0.0 never)
pub fn weight(event_type: &EventType, destination: &StarSystem, standing: Standing) -> f64 {
    rules()
        .iter()
        .filter(|rule| rule.events.contains(event_type))
        .filter(|rule| {
            rule.government
                .as_ref()
                .is_none_or(|list| list.contains(&destination.government))
        })
        .filter(|rule| {
            rule.economy
                .as_ref()
                .is_none_or(|list| list.contains(&destination.economy))
        })
        .filter(|rule| {
            rule.standing
                .as_ref()
                .is_none_or(|list| list.contains(&standing))
        })
        .filter(|rule| {
            rule.lawful
                .is_none_or(|lawful| lawful == destination.government.is_lawful())
        })
        .map(|rule| rule.weight)
        .product()
}

#[cfg(test)]
mod tests;
