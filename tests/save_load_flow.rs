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
        game.trade_log.recent(1)[0].unit_price
    );
}
