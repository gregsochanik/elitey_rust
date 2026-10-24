mod common;

use common::{args, new_game};
use elitey_rust::data::save_load::SaveLoadManager;

#[test]
fn the_world_clock_counts_jumps_and_is_saved() {
    let mut game = new_game(100);
    assert_eq!(game.world_clock, 0);

    game.travel_to_system("DISO").unwrap();
    game.travel_to_system("LAVE").unwrap();
    assert_eq!(game.world_clock, 2);

    let manager = SaveLoadManager::new().unwrap();
    manager.save_game(&game, "market_flow").unwrap();
    assert_eq!(manager.load_game("market_flow").unwrap().world_clock, 2);
}

#[test]
fn old_saves_without_a_clock_start_at_zero() {
    let game = new_game(100);
    let mut value = serde_json::to_value(&game).unwrap();
    value.as_object_mut().unwrap().remove("world_clock");

    let loaded: elitey_rust::game::GameState = serde_json::from_value(value).unwrap();

    assert_eq!(loaded.world_clock, 0);
}

#[test]
fn markets_still_trade_after_the_state_change() {
    let mut game = new_game(500);
    assert!(game.handle_buy_command(&args("food 2")));
    assert!(game.handle_sell_command(&args("food 2")));
}
