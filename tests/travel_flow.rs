mod common;

use common::new_game;

#[test]
fn reachable_systems_list_the_fuel_each_jump_costs() {
    let game = new_game(100);
    let destinations = game.reachable_destinations().unwrap();

    let diso = destinations.iter().find(|d| d.name == "DISO").unwrap();
    assert!(diso.distance > 2.0 && diso.distance < 3.0);
    assert_eq!(diso.fuel, 3);
    assert!(destinations
        .windows(2)
        .all(|pair| pair[0].distance <= pair[1].distance));
}

#[test]
fn only_systems_within_fuel_range_are_listed() {
    let mut game = new_game(100);
    game.ship.fuel_current = 3;

    let destinations = game.reachable_destinations().unwrap();

    assert_eq!(destinations.len(), 1);
    assert_eq!(destinations[0].name, "DISO");
    assert!(destinations.iter().all(|d| d.fuel <= 3));
}
