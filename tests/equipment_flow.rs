mod common;

use common::{args, new_game};
use elitey_rust::data::commodities::CommodityType;

#[test]
fn racks_raise_capacity_and_a_full_hold_blocks_selling_them() {
    let mut game = new_game(500);

    assert!(game.handle_equipment_command(&args("buy expanded-racks")));
    assert_eq!(game.ship.cargo_capacity(), 25);

    assert!(game.ship.add_cargo(CommodityType::Food, 24));
    assert!(!game.handle_equipment_command(&args("sell expanded-racks")));
    assert_eq!(game.ship.cargo_capacity(), 25);
}

#[test]
fn credits_follow_buys_replacements_and_sales() {
    let mut game = new_game(500);

    assert!(game.handle_equipment_command(&args("buy expanded-racks"))); // -30
    assert!(game.handle_equipment_command(&args("buy military-racks"))); // -80, +15
    assert!(game.handle_equipment_command(&args("sell military-racks"))); // +40

    assert_eq!(game.commander.credits(), 500 - 30 - 80 + 15 + 40);
    assert_eq!(game.ship.cargo_capacity(), 20);
}

#[test]
fn changing_ship_sells_equipment_and_resizes_the_hold() {
    let mut game = new_game(10_000);
    assert!(game.handle_equipment_command(&args("buy elite-racks"))); // 150, hold 40
    let before = game.commander.credits();

    assert!(game.handle_upgrade_purchase_command(&args("buy cobra4")));

    let cost = elitey_rust::data::ships::ShipUpgradeManager::new().calculate_upgrade_cost(
        &elitey_rust::data::ships::ShipType::CobraMkIII,
        &elitey_rust::data::ships::ShipType::CobraMkIV,
    );
    assert_eq!(game.commander.credits(), before - cost + 75);
    assert_eq!(game.ship.cargo_capacity(), 35);
}

#[test]
fn upgrade_accepts_the_ship_name_in_every_written_form() {
    for command in [
        "cobra4",
        "buy cobra4",
        "Cobra Mk IV",
        "\"Cobra Mk IV\"",
        "cobra-mk-iv",
        "COBRA MK 4",
    ] {
        let mut game = new_game(10_000);
        assert!(
            game.handle_upgrade_purchase_command(&args(command)),
            "{}",
            command
        );
        assert_eq!(
            game.ship.ship_type,
            elitey_rust::data::ships::ShipType::CobraMkIV,
            "{}",
            command
        );
    }
}

#[test]
fn upgrade_with_no_ship_or_an_unknown_one_changes_nothing() {
    for command in ["", "buy", "cobra9", "\"Elite Mk X\""] {
        let mut game = new_game(10_000);
        assert!(
            !game.handle_upgrade_purchase_command(&args(command)),
            "{}",
            command
        );
        assert_eq!(game.commander.credits(), 10_000);
    }
}
