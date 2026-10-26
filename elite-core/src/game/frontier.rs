use crate::data::systems::StarSystem;
use std::collections::HashSet;

/// The share of stars, the farthest from Sol, that count as the frontier
const FRONTIER_SHARE: f64 = 0.12;

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

#[cfg(test)]
mod tests;
