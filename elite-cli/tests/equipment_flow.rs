mod common;

use common::{args, new_game};
use elite_core::data::commodities::CommodityType;

#[test]
fn racks_raise_capacity_and_a_full_hold_blocks_selling_them() {
    let mut game = new_game(500);

    assert!(elite_cli::cli::equipment::command(
        &mut game,
        &args("buy expanded-racks")
    ));
    assert_eq!(game.ship.cargo_capacity(), 25);

    assert!(game.ship.add_cargo(CommodityType::Food, 24));
    assert!(!elite_cli::cli::equipment::command(
        &mut game,
        &args("sell expanded-racks")
    ));
    assert_eq!(game.ship.cargo_capacity(), 25);
}

#[test]
fn credits_follow_buys_replacements_and_sales() {
    let mut game = new_game(500);

    assert!(elite_cli::cli::equipment::command(
        &mut game,
        &args("buy expanded-racks")
    )); // -30
    assert!(elite_cli::cli::equipment::command(
        &mut game,
        &args("buy military-racks")
    )); // -80, +15
    assert!(elite_cli::cli::equipment::command(
        &mut game,
        &args("sell military-racks")
    )); // +40

    assert_eq!(game.commander.credits(), 500 - 30 - 80 + 15 + 40);
    assert_eq!(game.ship.cargo_capacity(), 20);
}

#[test]
fn changing_ship_sells_equipment_and_resizes_the_hold() {
    let mut game = new_game(10_000);
    assert!(elite_cli::cli::equipment::command(
        &mut game,
        &args("buy elite-racks")
    )); // 150, hold 40
    let before = game.commander.credits();

    assert!(elite_cli::cli::upgrades::buy(
        &mut game,
        &args("buy cobra4")
    ));

    let cost = elite_core::data::ships::ShipUpgradeManager::new().calculate_upgrade_cost(
        &elite_core::data::ships::ShipType::starting(),
        &elite_core::data::ships::ShipType::from_string("cobra4").unwrap(),
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
            elite_cli::cli::upgrades::buy(&mut game, &args(command)),
            "{}",
            command
        );
        assert_eq!(
            game.ship.ship_type,
            elite_core::data::ships::ShipType::from_string("cobra4").unwrap(),
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
            !elite_cli::cli::upgrades::buy(&mut game, &args(command)),
            "{}",
            command
        );
        assert_eq!(game.commander.credits(), 10_000);
    }
}
