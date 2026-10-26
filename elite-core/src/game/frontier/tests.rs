use super::*;
use crate::data::systems::SystemManager;

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
