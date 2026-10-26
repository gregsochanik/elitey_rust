use crate::data::systems::StarSystem;
use crate::game::travel::fuel_for_distance;
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};
use thiserror::Error;

/// One jump of a route
#[derive(Debug, Clone, PartialEq)]
pub struct Hop {
    pub from: String,
    pub to: String,
    pub distance: f64,
    pub fuel: u32,
}

/// A path through the galaxy that never needs more fuel in one jump than the tank holds
#[derive(Debug, Clone, PartialEq)]
pub struct Route {
    pub hops: Vec<Hop>,
    pub total_fuel: u32,
    pub total_distance: f64,
    /// Systems where the tank must be topped up to carry on
    pub refuel_at: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RouteError {
    #[error("System '{0}' not found")]
    UnknownSystem(String),
    #[error("You are already at {0}")]
    AlreadyThere(String),
    #[error("No route to {0} within your fuel range")]
    NoRoute(String),
}

/// The route from `from` to `to` that burns the least fuel, using jumps no longer than the
/// tank can hold. `fuel_now` decides where refuelling is needed along the way.
pub fn plan_route(
    systems: &[&StarSystem],
    from: &str,
    to: &str,
    fuel_capacity: u32,
    fuel_now: u32,
) -> Result<Route, RouteError> {
    let index_of = |name: &str| {
        systems
            .iter()
            .position(|system| system.name.eq_ignore_ascii_case(name))
    };
    let start = index_of(from).ok_or_else(|| RouteError::UnknownSystem(from.to_string()))?;
    let goal = index_of(to).ok_or_else(|| RouteError::UnknownSystem(to.to_string()))?;
    if start == goal {
        return Err(RouteError::AlreadyThere(systems[goal].name.clone()));
    }

    let mut best: HashMap<usize, (u32, u32)> = HashMap::from([(start, (0, 0))]);
    let mut came_from: HashMap<usize, usize> = HashMap::new();
    let mut queue = BinaryHeap::from([Reverse((0u32, 0u32, start))]);

    while let Some(Reverse((fuel, hops, here))) = queue.pop() {
        if here == goal {
            break;
        }
        if best.get(&here).is_some_and(|&known| known < (fuel, hops)) {
            continue;
        }
        for (next, system) in systems.iter().enumerate() {
            if next == here {
                continue;
            }
            let cost = fuel_for_distance(systems[here].distance_to(system));
            if cost > fuel_capacity {
                continue;
            }
            let candidate = (fuel + cost, hops + 1);
            if best.get(&next).is_none_or(|&known| candidate < known) {
                best.insert(next, candidate);
                came_from.insert(next, here);
                queue.push(Reverse((candidate.0, candidate.1, next)));
            }
        }
    }

    if !came_from.contains_key(&goal) {
        return Err(RouteError::NoRoute(systems[goal].name.clone()));
    }
    let mut path = vec![goal];
    while let Some(&previous) = came_from.get(path.last().expect("path is not empty")) {
        path.push(previous);
        if previous == start {
            break;
        }
    }
    path.reverse();

    let mut hops = Vec::new();
    let mut refuel_at = Vec::new();
    let mut fuel_left = fuel_now.min(fuel_capacity);
    for pair in path.windows(2) {
        let (a, b) = (systems[pair[0]], systems[pair[1]]);
        let distance = a.distance_to(b);
        let fuel = fuel_for_distance(distance);
        if fuel > fuel_left {
            refuel_at.push(a.name.clone());
            fuel_left = fuel_capacity;
        }
        fuel_left -= fuel;
        hops.push(Hop {
            from: a.name.clone(),
            to: b.name.clone(),
            distance,
            fuel,
        });
    }
    Ok(Route {
        total_fuel: hops.iter().map(|hop| hop.fuel).sum(),
        total_distance: hops.iter().map(|hop| hop.distance).sum(),
        hops,
        refuel_at,
    })
}

#[cfg(test)]
mod tests;
