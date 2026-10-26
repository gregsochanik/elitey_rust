use super::*;
use crate::data::systems::{EconomyType, GovernmentType, SystemManager};

fn star(name: &str, x: u16, y: u16) -> StarSystem {
    StarSystem {
        name: name.to_string(),
        x,
        y,
        economy: EconomyType::AverageIndustrial,
        government: GovernmentType::Democracy,
        tech_level: 5,
        turnover: 0,
        radius: 0,
        population: String::new(),
        description: String::new(),
        star_class: String::new(),
    }
}

/// A line of stars 8 grid units (2 ly) apart, with a detour and a gap
fn chain() -> Vec<StarSystem> {
    vec![
        star("A", 0, 0),
        star("B", 8, 0),
        star("C", 16, 0),
        star("D", 24, 0),
        star("Far", 200, 200),
    ]
}

fn refs(systems: &[StarSystem]) -> Vec<&StarSystem> {
    systems.iter().collect()
}

#[test]
fn the_route_takes_the_cheapest_fuel() {
    let systems = chain();

    let route = plan_route(&refs(&systems), "A", "D", 7, 7).unwrap();

    let names: Vec<_> = route.hops.iter().map(|hop| hop.to.as_str()).collect();
    assert_eq!(route.hops[0].from, "A");
    assert_eq!(
        route.total_fuel,
        route.hops.iter().map(|hop| hop.fuel).sum::<u32>()
    );
    assert!(names.ends_with(&["D"]));
    // stepping through B and C costs 2+2+2; a direct 6 ly jump costs 6, so the tie goes to fewer hops
    assert_eq!(route.total_fuel, 6);
    assert_eq!(route.hops.len(), 1);
}

#[test]
fn a_short_tank_forces_the_stepping_stones() {
    let systems = chain();

    let route = plan_route(&refs(&systems), "A", "D", 3, 3).unwrap();

    let names: Vec<_> = route.hops.iter().map(|hop| hop.to.as_str()).collect();
    assert_eq!(names, ["B", "C", "D"]);
    assert_eq!(route.total_fuel, 6);
}

#[test]
fn refuelling_is_flagged_where_the_tank_would_run_dry() {
    let systems = chain();

    let route = plan_route(&refs(&systems), "A", "D", 3, 3).unwrap();
    assert_eq!(route.refuel_at, ["B".to_string(), "C".to_string()]);

    let full = plan_route(&refs(&systems), "A", "D", 6, 6).unwrap();
    assert!(full.refuel_at.is_empty());

    let low = plan_route(&refs(&systems), "A", "D", 3, 1).unwrap();
    assert_eq!(low.refuel_at.first().map(String::as_str), Some("A"));
}

#[test]
fn unreachable_unknown_and_same_systems_are_refused() {
    let systems = chain();

    assert_eq!(
        plan_route(&refs(&systems), "A", "Far", 7, 7),
        Err(RouteError::NoRoute("Far".to_string()))
    );
    assert_eq!(
        plan_route(&refs(&systems), "A", "Nowhere", 7, 7),
        Err(RouteError::UnknownSystem("Nowhere".to_string()))
    );
    assert_eq!(
        plan_route(&refs(&systems), "b", "B", 7, 7),
        Err(RouteError::AlreadyThere("B".to_string()))
    );
}

#[test]
fn every_star_in_the_galaxy_can_be_reached_from_sol_on_the_starting_tank() {
    let manager = SystemManager::new().unwrap();
    let systems = manager.get_all_systems();

    for system in &systems {
        if system.name == "Sol" {
            continue;
        }
        let route = plan_route(&systems, "Sol", &system.name, 7, 7)
            .unwrap_or_else(|error| panic!("{}: {error}", system.name));
        assert!(route.hops.iter().all(|hop| hop.fuel <= 7));
        assert_eq!(route.hops.last().unwrap().to, system.name);
    }
}
