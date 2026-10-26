use crate::data::commodities::CommodityType;
use crate::data::systems::StarSystem;
use crate::game::events::{apply_cargo_gain, CargoGain};
use crate::game::{Commander, Ship};
use rand::seq::SliceRandom;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::OnceLock;

/// The share of stars, the farthest from Sol, that count as the frontier
const FRONTIER_SHARE: f64 = 0.12;

/// A find at a frontier star, as logged on the commander
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Discovery {
    pub system: String,
    pub id: String,
    pub title: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct CargoFind {
    commodity: String,
    tons: [u32; 2],
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Find {
    id: String,
    title: String,
    text: String,
    weight: u32,
    credits: Option<[u32; 2]>,
    cargo: Option<CargoFind>,
    #[serde(default)]
    refuel: bool,
    #[serde(default)]
    chart: usize,
}

fn finds() -> &'static [Find] {
    static FINDS: OnceLock<Vec<Find>> = OnceLock::new();
    FINDS.get_or_init(|| {
        serde_json::from_str(include_str!("../../data/discoveries.json"))
            .expect("data/discoveries.json is valid")
    })
}

/// The farthest stars from Sol (or from the middle of the map when there is no Sol)
pub fn frontier_systems(systems: &[&StarSystem]) -> HashSet<String> {
    let centre = systems
        .iter()
        .find(|system| system.name == "Sol")
        .map(|system| (f64::from(system.x), f64::from(system.y)))
        .unwrap_or_else(|| {
            let n = systems.len().max(1) as f64;
            (
                systems.iter().map(|s| f64::from(s.x)).sum::<f64>() / n,
                systems.iter().map(|s| f64::from(s.y)).sum::<f64>() / n,
            )
        });
    let mut by_distance: Vec<(&str, f64)> = systems
        .iter()
        .map(|system| {
            (
                system.name.as_str(),
                (f64::from(system.x) - centre.0).hypot(f64::from(system.y) - centre.1),
            )
        })
        .collect();
    by_distance.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(b.0)));
    let count = (systems.len() as f64 * FRONTIER_SHARE).ceil() as usize;
    by_distance
        .into_iter()
        .take(count)
        .map(|(name, _)| name.to_string())
        .collect()
}

/// What the commander finds on first reaching a frontier star: applied at once, with the
/// messages to show. `nearby` lists the other stars, nearest first, for charts.
pub fn discover(
    system: &str,
    nearby: &[String],
    commander: &mut Commander,
    ship: &mut Ship,
    rng: &mut impl Rng,
) -> Vec<String> {
    let find = finds()
        .choose_weighted(rng, |find| find.weight)
        .expect("there are discoveries");
    let mut messages = vec![format!("🔭 Discovery: {}", find.title), find.text.clone()];

    if let Some([low, high]) = find.credits {
        let credits = rng.gen_range(low..=high);
        commander.add_credits(credits);
        messages.push(format!("You earn {} credits.", credits));
    }
    if let Some(cargo) = &find.cargo {
        let commodity = CommodityType::from_string(&cargo.commodity)
            .expect("discoveries name real commodities");
        let tons = rng.gen_range(cargo.tons[0]..=cargo.tons[1]);
        let gain = CargoGain { commodity, tons };
        match apply_cargo_gain(ship, &gain) {
            0 => messages.push("There is no room in your hold to take it.".to_string()),
            taken => messages.push(format!(
                "You take {} tons of {}.",
                taken,
                gain.commodity.name()
            )),
        }
    }
    if find.refuel {
        ship.fuel_current = ship.fuel_capacity();
    }
    if find.chart > 0 {
        let charted: Vec<&String> = nearby
            .iter()
            .filter(|name| !commander.has_visited(name.as_str()))
            .take(find.chart)
            .collect();
        for name in &charted {
            commander.visit(name.as_str());
        }
        let names: Vec<&str> = charted.iter().map(|name| name.as_str()).collect();
        messages.push(format!("Charted: {}.", names.join(", ")));
    }

    commander.discoveries.push(Discovery {
        system: system.to_string(),
        id: find.id.clone(),
        title: find.title.clone(),
    });
    messages
}

#[cfg(test)]
mod tests;
