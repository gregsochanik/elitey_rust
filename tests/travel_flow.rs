mod common;

use common::new_game;
use rand::SeedableRng;

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

#[test]
fn a_stranded_ship_can_refuel_and_jump_again() {
    let mut game = new_game(100);
    game.ship.fuel_current = 1;
    assert!(game.reachable_destinations().unwrap().is_empty());

    assert!(elitey_rust::cli::upkeep::refuel(&mut game, &[]));

    assert_eq!(game.ship.fuel_current, 7);
    assert_eq!(game.commander.credits(), 88);
    assert!(!game.reachable_destinations().unwrap().is_empty());
}

#[test]
fn a_ship_with_no_fuel_credits_or_cargo_is_rescued_and_can_move_again() {
    let mut game = new_game(0);
    game.ship.fuel_current = 1;
    assert!(game.reachable_destinations().unwrap().is_empty());

    assert!(elitey_rust::cli::upkeep::refuel(&mut game, &[]));

    assert!(!game.reachable_destinations().unwrap().is_empty());
    assert_eq!(game.commander.reputation, -2);
    assert_eq!(game.commander.credits(), 0);
}

#[test]
fn the_jobs_board_is_stable_while_docked_and_rerolls_after_a_jump() {
    let mut game = new_game(100);
    let first = game
        .jobs_here(&mut rand::rngs::StdRng::seed_from_u64(1))
        .unwrap();
    let again = game
        .jobs_here(&mut rand::rngs::StdRng::seed_from_u64(99))
        .unwrap();
    assert_eq!(first, again);
    assert!((3..=5).contains(&first.len()));
    assert!(first
        .iter()
        .all(|job| job.job.fuel_cost <= game.ship.fuel_capacity()));

    game.job_board.clear();
    let rerolled = game
        .jobs_here(&mut rand::rngs::StdRng::seed_from_u64(99))
        .unwrap();
    assert_ne!(first, rerolled);
}

#[test]
fn old_saves_without_a_job_board_load_with_an_empty_one() {
    let game = new_game(100);
    let mut value = serde_json::to_value(&game).unwrap();
    value.as_object_mut().unwrap().remove("job_board");

    let mut loaded: elitey_rust::game::GameState = serde_json::from_value(value).unwrap();
    assert!(
        loaded
            .jobs_here(&mut rand::rngs::StdRng::seed_from_u64(3))
            .unwrap()
            .len()
            >= 3
    );
}
