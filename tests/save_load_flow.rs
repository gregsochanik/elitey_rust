mod common;

use common::{args, new_game};
use elitey_rust::data::commodities::CommodityType;
use elitey_rust::data::save_load::SaveLoadManager;

#[test]
fn a_played_game_survives_saving_to_disk_and_loading_back() {
    let mut game = new_game(500);
    assert!(game.handle_buy_command(&args("food 3")));
    assert!(game.handle_equipment_command(&args("buy expanded-racks")));
    game.commander.reputation = 12;

    let manager = SaveLoadManager::new().unwrap();
    manager.save_game(&game, "flow").unwrap();
    let mut loaded = manager.load_game("flow").unwrap();

    assert_eq!(loaded.commander.credits(), game.commander.credits());
    assert_eq!(loaded.commander.reputation, 12);
    assert_eq!(loaded.commander.location(), game.commander.location());
    assert_eq!(loaded.ship.cargo_hold.get_quantity(&CommodityType::Food), 3);
    assert_eq!(loaded.ship.cargo_capacity(), 25);
    assert_eq!(loaded.ship.equipment, game.ship.equipment);
    assert_eq!(loaded.trade_log.recent(10).len(), 1);

    // Market stock, prices and the player's impact on them continue where they left off
    let expected = game.market_prices().unwrap();
    let restored = loaded.market_prices().unwrap();
    assert_eq!(restored.buy_prices, expected.buy_prices);
    assert_eq!(restored.sell_prices, expected.sell_prices);
    assert_eq!(restored.availability, expected.availability);
    assert!(loaded.handle_buy_command(&args("food 1")));
}

#[test]
fn autosaves_are_kept_per_commander() {
    use elitey_rust::game::SaveLoadHandler;

    common::isolate_home();
    let handler = SaveLoadHandler::new().unwrap();
    let mut first = new_game(111);
    first.commander.name = "Ada Lovelace".to_string();
    let mut second = new_game(222);
    second.commander.name = "Grace".to_string();

    assert!(handler.auto_save(&first));
    assert!(handler.auto_save(&second));

    assert_eq!(
        handler
            .find_autosave("ada lovelace")
            .unwrap()
            .unwrap()
            .commander
            .credits(),
        111
    );
    assert_eq!(
        handler
            .find_autosave("Grace")
            .unwrap()
            .unwrap()
            .commander
            .credits(),
        222
    );
    assert!(handler.find_autosave("Nobody").unwrap().is_none());

    assert_ne!(
        SaveLoadHandler::autosave_slot("Ada Lovelace"),
        SaveLoadHandler::autosave_slot("Ada_Lovelace")
    );
    assert!(handler.find_autosave("Ada_Lovelace").unwrap().is_none());

    let long_name = "x".repeat(300);
    assert!(SaveLoadHandler::autosave_slot(&long_name).len() < 100);
}

#[test]
fn a_save_from_before_game_data_moved_to_json_still_loads() {
    let json = include_str!("fixtures/save_before_json_data.json");
    let envelope: serde_json::Value = serde_json::from_str(json).unwrap();
    let mut state = envelope["game_state"].clone();
    assert_eq!(state["ship"]["ship_type"], "CobraMkIII");

    let loaded: elitey_rust::game::GameState = serde_json::from_value(state.clone()).unwrap();
    assert_eq!(loaded.ship.ship_type.id(), "cobra-mk-iii");
    assert_eq!(loaded.ship.cargo_capacity(), 20);
    assert_eq!(loaded.world_clock, 0);
    assert_eq!(loaded.ship.cargo_hold.get_quantity(&CommodityType::Food), 2);
    assert_eq!(loaded.trade_log.recent(10).len(), 2);
    let commodities: Vec<_> = loaded
        .trade_log
        .recent(10)
        .iter()
        .map(|trade| trade.commodity.clone())
        .collect();
    assert!(commodities.iter().all(|c| *c == CommodityType::Food));

    state["ship"]["ship_type"] = serde_json::json!("Python");
    let python: elitey_rust::game::GameState = serde_json::from_value(state).unwrap();
    assert_eq!(python.ship.ship_type.id(), "python");
    assert_eq!(python.ship.ship_type.max_hull(), 150);
}

#[test]
fn saves_write_ships_by_id() {
    let game = new_game(100);
    let value = serde_json::to_value(&game).unwrap();
    assert_eq!(value["ship"]["ship_type"], "cobra-mk-iii");
}

#[test]
fn saves_do_not_store_event_definitions() {
    let game = new_game(500);
    let json = serde_json::to_value(&game).unwrap();
    assert!(json.get("event_manager").is_none());
}

#[test]
fn an_old_save_uses_the_current_event_definitions() {
    use elitey_rust::game::events::{EventType, Redirect};
    use elitey_rust::game::EventPhase;

    let json = include_str!("fixtures/save_before_json_data.json");
    let envelope: serde_json::Value = serde_json::from_str(json).unwrap();
    let loaded: elitey_rust::game::GameState =
        serde_json::from_value(envelope["game_state"].clone()).unwrap();

    let wormhole = loaded
        .event_manager()
        .get_available_events(&EventPhase::Jump)
        .into_iter()
        .find(|event| event.event_type == EventType::Wormhole)
        .unwrap();
    assert!(wormhole
        .choices
        .iter()
        .any(|choice| choice.outcome.redirect == Some(Redirect::Anywhere)));
}
