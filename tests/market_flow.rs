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

#[test]
fn hopping_between_systems_does_not_reroll_prices_or_stock() {
    let mut game = new_game(500);
    let before = game.market_prices().unwrap();

    game.travel_to_system("DISO").unwrap();
    game.ship.refuel();
    game.travel_to_system("LAVE").unwrap();
    let after = game.market_prices().unwrap();

    assert_eq!(before.buy_prices, after.buy_prices);
    assert_eq!(before.sell_prices, after.sell_prices);
    assert_eq!(before.availability, after.availability);
}

#[test]
fn stock_bought_recovers_slowly_and_prices_refresh_after_the_period() {
    let mut game = new_game(500);
    let before = game.market_prices().unwrap();
    assert!(game.handle_buy_command(&args("food 3")));
    let stock = before.availability[&elitey_rust::data::commodities::CommodityType::Food];

    game.travel_to_system("DISO").unwrap();
    game.ship.refuel();
    game.travel_to_system("LAVE").unwrap();
    let after = game.market_prices().unwrap();
    assert_eq!(
        after.availability[&elitey_rust::data::commodities::CommodityType::Food],
        stock - 2
    );

    for _ in 0..9 {
        game.ship.refuel();
        game.travel_to_system("DISO").unwrap();
        game.travel_to_system("LAVE").unwrap();
    }
    let refreshed = game.market_prices().unwrap();
    assert_eq!(game.world_clock, 20);
    assert_ne!(refreshed.buy_prices, before.buy_prices);
}

#[test]
fn selling_pushes_the_price_down_and_it_recovers_over_the_next_jumps() {
    use elitey_rust::data::commodities::CommodityType;

    let mut game = new_game(2_000);
    assert!(game.handle_buy_command(&args("luxury 5")));
    let undisturbed = game.market_prices().unwrap().sell_prices[&CommodityType::Luxury];

    assert!(game.handle_sell_command(&args("luxury 5")));
    let pushed = game.market_prices().unwrap().sell_prices[&CommodityType::Luxury];
    assert!(pushed < undisturbed);

    game.travel_to_system("DISO").unwrap();
    game.ship.refuel();
    game.travel_to_system("LAVE").unwrap();
    let recovering = game.market_prices().unwrap().sell_prices[&CommodityType::Luxury];
    assert!(recovering > pushed && recovering <= undisturbed);
}

#[test]
fn buying_raises_the_buy_price_for_the_next_purchase() {
    use elitey_rust::data::commodities::CommodityType;

    let mut game = new_game(2_000);
    let before = game.market_prices().unwrap().buy_prices[&CommodityType::Food];
    assert!(game.handle_buy_command(&args("food 5")));
    let after = game.market_prices().unwrap().buy_prices[&CommodityType::Food];

    assert!(after > before);
}
