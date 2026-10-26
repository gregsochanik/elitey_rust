use super::*;
use crate::data::systems::SystemManager;
use crate::game::GameState;
use rand::rngs::StdRng;
use rand::SeedableRng;

#[test]
fn the_frontier_is_the_farthest_dozen_percent_from_sol() {
    let manager = SystemManager::new().unwrap();
    let systems = manager.get_all_systems();

    let frontier = frontier_systems(&systems);

    assert_eq!(frontier.len(), 31);
    assert!(!frontier.contains("Sol"));
    let sol = manager.get_system("Sol").unwrap();
    let nearest_frontier = systems
        .iter()
        .filter(|system| frontier.contains(&system.name))
        .map(|system| sol.distance_to(system))
        .fold(f64::INFINITY, f64::min);
    let farthest_inside = systems
        .iter()
        .filter(|system| !frontier.contains(&system.name))
        .map(|system| sol.distance_to(system))
        .fold(0.0, f64::max);
    assert!(nearest_frontier >= farthest_inside);
}

fn find_with(id: &str) -> (GameState, Vec<String>) {
    for seed in 0..500 {
        let mut game = GameState::new("Test".to_string());
        game.ship.fuel_current = 1;
        let nearby = game.nearest_systems("Sol");
        let mut rng = StdRng::seed_from_u64(seed);
        let messages = discover(
            "Far Star",
            &nearby,
            &mut game.commander,
            &mut game.ship,
            &mut rng,
        );
        if game
            .commander
            .discoveries
            .last()
            .is_some_and(|d| d.id == id)
        {
            return (game, messages);
        }
    }
    panic!("no seed found {id}");
}

#[test]
fn every_find_does_what_it_says() {
    let (game, messages) = find_with("derelict");
    assert!(game.commander.credits() >= 100 + 80);
    assert!((2..=4).contains(&game.ship.cargo_used()));
    assert!(messages[0].contains("Derelict freighter"));

    let (game, _) = find_with("survey");
    assert!((250..=450).contains(&game.commander.credits()));

    let (game, _) = find_with("beacon");
    assert_eq!(game.exploration().visited, 4);

    let (game, _) = find_with("fuel-cache");
    assert_eq!(game.ship.fuel_current(), game.ship.fuel_capacity());

    let (game, _) = find_with("artefact");
    assert_eq!(
        game.ship.cargo_hold.get_quantity(&CommodityType::Artefacts),
        1
    );
}

#[test]
fn a_discovery_is_logged_with_where_it_was_found() {
    let (game, _) = find_with("survey");

    assert_eq!(
        game.commander.discoveries,
        vec![Discovery {
            system: "Far Star".to_string(),
            id: "survey".to_string(),
            title: "Uncharted world".to_string(),
        }]
    );
}

#[test]
fn common_finds_turn_up_more_than_rare_ones() {
    let mut game = GameState::new("Test".to_string());
    let mut rng = StdRng::seed_from_u64(9);
    let mut counts = std::collections::HashMap::new();
    for _ in 0..2_000 {
        game.ship = crate::game::Ship::new_cobra_mk3();
        game.commander.discoveries.clear();
        discover("X", &[], &mut game.commander, &mut game.ship, &mut rng);
        *counts
            .entry(game.commander.discoveries[0].id.clone())
            .or_insert(0) += 1;
    }

    assert!(counts["derelict"] > counts["survey"]);
    assert!(counts["survey"] > counts["artefact"]);
    assert!(counts["artefact"] > 50);
}

#[test]
fn artefacts_show_in_the_market_only_while_held_and_can_be_sold() {
    let (mut game, _) = find_with("artefact");
    let view = game.market_overview().unwrap();
    assert!(view
        .rows
        .iter()
        .any(|row| row.commodity == CommodityType::Artefacts && row.held == 1));
    let before = game.commander.credits();

    game.sell(CommodityType::Artefacts, 1).unwrap();

    assert!(game.commander.credits() > before + 150);
    let view = game.market_overview().unwrap();
    assert!(!view
        .rows
        .iter()
        .any(|row| row.commodity == CommodityType::Artefacts));
    assert!(game.buy(CommodityType::Artefacts, 1).is_err());
}

#[test]
fn the_game_knows_which_stars_are_on_the_frontier() {
    let game = crate::game::GameState::new("Test".to_string());
    let systems = game.travel_handler.get_available_systems();
    let frontier = frontier_systems(&systems);

    assert!(!game.is_frontier("Sol"));
    assert!(!game.is_frontier("Proxima Centauri"));
    let far = frontier.iter().min().unwrap();
    assert!(game.is_frontier(far));
    assert!(game.system_info(far).unwrap().frontier);
    assert!(!game.system_info("Sol").unwrap().frontier);
}
