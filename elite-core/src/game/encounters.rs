use crate::data::systems::{EconomyType, GovernmentType, StarSystem};
use crate::game::commander::Standing;
use crate::game::events::EventType;
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

#[derive(Deserialize)]
struct Table {
    rules: Vec<Rule>,
}

fn rules() -> &'static [Rule] {
    static RULES: OnceLock<Vec<Rule>> = OnceLock::new();
    RULES.get_or_init(|| {
        let table: Table = serde_json::from_str(include_str!("../../data/encounters.json"))
            .expect("data/encounters.json is valid");
        table.rules
    })
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
