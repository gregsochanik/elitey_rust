mod common;

use common::new_game;
use elitey_rust::game::aftermath::settle;
use elitey_rust::game::combat::{Combat, Combatant, Outcome};
use elitey_rust::game::fight::run_fight;
use rand::rngs::StdRng;
use rand::SeedableRng;

fn fight(game: &mut elitey_rust::game::GameState, typed: &str, dice: f64) -> Outcome {
    let mut combat = Combat::new(Combatant::from_ship(&game.ship), Combatant::pirate_raider());
    let mut roll = || dice;
    let outcome = run_fight(
        &mut combat,
        &mut typed.as_bytes(),
        &mut std::io::sink(),
        &mut roll,
    )
    .unwrap();
    settle(
        outcome,
        combat.player.hull,
        &mut game.commander,
        &mut game.ship,
        &mut StdRng::seed_from_u64(1),
    );
    outcome
}

#[test]
fn winning_a_fight_pays_out_and_keeps_the_damage() {
    let mut game = new_game(100);

    // every shot hits, so the starting loadout wins and the raider lands hits too
    let outcome = fight(&mut game, &"a\n".repeat(20), 0.0);

    assert_eq!(outcome, Outcome::Victory);
    assert!(game.commander.credits() >= 300);
    assert_eq!(game.commander.reputation, 5);
    assert!(game.ship.cargo_used() >= 1);
    assert!(game.ship.hull_percentage < 100);
}

#[test]
fn losing_a_fight_strips_the_ship_but_the_game_goes_on() {
    let mut game = new_game(400);
    game.handle_equipment_command(&["sell".to_string(), "pulse-laser".to_string()]);
    game.ship.hull_percentage = 12;
    game.ship
        .cargo_hold
        .add_cargo(elitey_rust::data::commodities::CommodityType::Food, 5);

    // no weapons to attack with: fleeing fails every time, so the raider wins
    let outcome = fight(&mut game, &"f\n".repeat(30), 0.55);

    assert_eq!(outcome, Outcome::Defeat);
    assert_eq!(game.ship.cargo_used(), 0);
    assert_eq!(game.ship.hull_percentage, 10);
    assert!(game.is_running());
}

#[test]
fn surrendering_costs_cargo_and_credits_only() {
    let mut game = new_game(500);
    game.ship
        .cargo_hold
        .add_cargo(elitey_rust::data::commodities::CommodityType::Food, 8);

    let outcome = fight(&mut game, "s\n", 0.5);

    assert_eq!(outcome, Outcome::Surrendered);
    assert_eq!(game.ship.cargo_used(), 3);
    assert_eq!(game.commander.credits(), 300);
    assert_eq!(game.ship.hull_percentage, 100);
}
