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

    // Market stock and prices continue where they left off
    assert!(loaded.handle_buy_command(&args("food 1")));
    assert_eq!(
        loaded.trade_log.recent(1)[0].unit_price,
        elitey_rust::game::commander::Standing::Trusted
            .buy_price(game.trade_log.recent(1)[0].unit_price)
    );
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
