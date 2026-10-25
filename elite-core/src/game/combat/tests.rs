use super::*;
use crate::game::encounters::Difficulty;
use crate::game::npc::NpcAction;

fn fighter(name: &str, hull: u32, shield: u32, recharge: u32, weapons: &[(u32, f64)]) -> Combatant {
    Combatant {
        name: name.to_string(),
        hull,
        max_hull: hull,
        shield,
        max_shield: shield,
        recharge,
        weapons: weapons
            .iter()
            .map(|(damage, accuracy)| Weapon {
                damage: *damage,
                accuracy: *accuracy,
            })
            .collect(),
    }
}

/// Dice that return the given values in order and fail the test if asked for more
fn rolls(values: &[f64]) -> impl FnMut() -> f64 + '_ {
    let mut remaining = values.iter();
    move || *remaining.next().expect("more rolls than expected")
}

#[test]
fn a_shot_hits_when_the_roll_is_below_accuracy_and_misses_otherwise() {
    let mut combat = Combat::new(
        fighter("You", 100, 0, 0, &[(15, 0.9)]),
        fighter("Raider", 60, 0, 0, &[(10, 0.6)]),
    );

    // player hits, raider misses
    combat
        .resolve_round(Action::Attack, &mut rolls(&[0.5, 0.7]))
        .unwrap();
    assert_eq!(combat.enemy.hull, 45);
    assert_eq!(combat.player.hull, 100);

    // player misses, raider hits
    combat
        .resolve_round(Action::Attack, &mut rolls(&[0.95, 0.1]))
        .unwrap();
    assert_eq!(combat.enemy.hull, 45);
    assert_eq!(combat.player.hull, 90);
}

#[test]
fn shields_absorb_damage_before_the_hull() {
    let mut combat = Combat::new(
        fighter("You", 100, 0, 0, &[(15, 1.0)]),
        fighter("Raider", 60, 20, 0, &[(10, 0.0)]),
    );

    combat
        .resolve_round(Action::Attack, &mut rolls(&[0.0, 0.99]))
        .unwrap();

    assert_eq!(combat.enemy.shield, 5);
    assert_eq!(combat.enemy.hull, 60);
}

#[test]
fn damage_beyond_the_shield_carries_over_to_the_hull() {
    let mut combat = Combat::new(
        fighter("You", 100, 0, 0, &[(15, 1.0)]),
        fighter("Raider", 60, 10, 0, &[(10, 0.0)]),
    );

    combat
        .resolve_round(Action::Attack, &mut rolls(&[0.0, 0.99]))
        .unwrap();

    assert_eq!(combat.enemy.shield, 0);
    assert_eq!(combat.enemy.hull, 55);
}

#[test]
fn shields_recharge_each_round_but_never_above_the_maximum() {
    let mut combat = Combat::new(
        fighter("You", 100, 20, 5, &[(1, 0.0)]),
        fighter("Raider", 60, 20, 3, &[(8, 1.0)]),
    );

    // raider hits for 8: shield 20 -> 12, then recharges to 17
    combat
        .resolve_round(Action::Attack, &mut rolls(&[0.99, 0.0]))
        .unwrap();
    assert_eq!(combat.player.shield, 17);

    // raider hits for 8 again: 17 -> 9 -> 14
    combat
        .resolve_round(Action::Attack, &mut rolls(&[0.99, 0.0]))
        .unwrap();
    assert_eq!(combat.player.shield, 14);

    // an undamaged shield stays at its maximum
    assert_eq!(combat.enemy.shield, 20);
}

#[test]
fn every_installed_weapon_fires_each_round() {
    let mut combat = Combat::new(
        fighter("You", 100, 0, 0, &[(10, 1.0), (7, 1.0)]),
        fighter("Raider", 60, 0, 0, &[(1, 0.0)]),
    );

    combat
        .resolve_round(Action::Attack, &mut rolls(&[0.0, 0.0, 0.99]))
        .unwrap();

    assert_eq!(combat.enemy.hull, 43);
}

#[test]
fn a_destroyed_enemy_does_not_fire_back() {
    let mut combat = Combat::new(
        fighter("You", 100, 0, 0, &[(80, 1.0)]),
        fighter("Raider", 60, 0, 0, &[(10, 1.0)]),
    );

    // only one roll is available: the player's shot
    let outcome = combat
        .resolve_round(Action::Attack, &mut rolls(&[0.0]))
        .unwrap();

    assert_eq!(outcome, Outcome::Victory);
    assert_eq!(combat.player.hull, 100);
}

#[test]
fn the_player_is_defeated_when_the_hull_reaches_zero() {
    let mut combat = Combat::new(
        fighter("You", 8, 0, 0, &[(1, 0.0)]),
        fighter("Raider", 60, 0, 0, &[(10, 1.0)]),
    );

    let outcome = combat
        .resolve_round(Action::Attack, &mut rolls(&[0.99, 0.0]))
        .unwrap();

    assert_eq!(outcome, Outcome::Defeat);
    assert_eq!(combat.player.hull, 0);
}

#[test]
fn fleeing_succeeds_below_the_flee_chance_and_ends_the_fight() {
    let mut combat = Combat::new(
        fighter("You", 100, 0, 0, &[(10, 1.0)]),
        fighter("Raider", 60, 0, 0, &[(10, 1.0)]),
    );

    let outcome = combat
        .resolve_round(Action::Flee, &mut rolls(&[0.3]))
        .unwrap();

    assert_eq!(outcome, Outcome::Escaped);
    assert_eq!(combat.player.hull, 100);
    assert_eq!(combat.enemy.hull, 60);
}

#[test]
fn a_failed_escape_gives_the_enemy_a_free_attack() {
    let mut combat = Combat::new(
        fighter("You", 100, 0, 0, &[(10, 1.0)]),
        fighter("Raider", 60, 0, 0, &[(10, 1.0)]),
    );

    let outcome = combat
        .resolve_round(Action::Flee, &mut rolls(&[0.7, 0.0]))
        .unwrap();

    assert_eq!(outcome, Outcome::Ongoing);
    assert_eq!(combat.player.hull, 90);
    assert_eq!(combat.enemy.hull, 60);
}

#[test]
fn surrendering_ends_the_fight_without_damage() {
    let mut combat = Combat::new(
        fighter("You", 100, 0, 0, &[(10, 1.0)]),
        fighter("Raider", 60, 0, 0, &[(10, 1.0)]),
    );

    let outcome = combat
        .resolve_round(Action::Surrender, &mut rolls(&[]))
        .unwrap();

    assert_eq!(outcome, Outcome::Surrendered);
    assert_eq!(combat.player.hull, 100);
}

#[test]
fn attacking_without_weapons_is_refused_and_takes_no_turn() {
    let mut combat = Combat::new(
        fighter("You", 100, 20, 5, &[]),
        fighter("Raider", 60, 20, 3, &[(10, 1.0)]),
    );
    let before = (combat.player.clone(), combat.enemy.clone());

    let result = combat.resolve_round(Action::Attack, &mut rolls(&[]));

    assert_eq!(result, Err(CombatError::NoWeapons));
    assert_eq!((combat.player.clone(), combat.enemy.clone()), before);
    assert_eq!(combat.outcome(), Outcome::Ongoing);
}

#[test]
fn an_unarmed_ship_can_still_flee_or_surrender() {
    let mut combat = Combat::new(
        fighter("You", 100, 0, 0, &[]),
        fighter("Raider", 60, 0, 0, &[(10, 1.0)]),
    );
    assert_eq!(
        combat
            .resolve_round(Action::Flee, &mut rolls(&[0.1]))
            .unwrap(),
        Outcome::Escaped
    );

    let mut combat = Combat::new(
        fighter("You", 100, 0, 0, &[]),
        fighter("Raider", 60, 0, 0, &[(10, 1.0)]),
    );
    assert_eq!(
        combat
            .resolve_round(Action::Surrender, &mut rolls(&[]))
            .unwrap(),
        Outcome::Surrendered
    );
}

#[test]
fn no_more_rounds_can_be_played_after_the_fight_ends() {
    let mut combat = Combat::new(
        fighter("You", 100, 0, 0, &[(10, 1.0)]),
        fighter("Raider", 60, 0, 0, &[(10, 1.0)]),
    );
    combat
        .resolve_round(Action::Surrender, &mut rolls(&[]))
        .unwrap();

    assert_eq!(
        combat.resolve_round(Action::Attack, &mut rolls(&[0.0, 0.0])),
        Err(CombatError::Finished)
    );
}

#[test]
fn the_log_records_hits_misses_and_the_ending() {
    let mut combat = Combat::new(
        fighter("You", 100, 0, 0, &[(80, 1.0)]),
        fighter("Raider", 60, 0, 0, &[(10, 1.0)]),
    );
    combat
        .resolve_round(Action::Attack, &mut rolls(&[0.0]))
        .unwrap();

    assert_eq!(combat.log.len(), 2);
    assert!(combat.log[0].contains("hits"));
    assert!(combat.log[1].contains("destroyed"));
}

#[test]
fn the_pirate_raider_has_the_specified_stats() {
    let raider = Combatant::pirate_raider();
    assert_eq!((raider.hull, raider.shield, raider.recharge), (70, 25, 4));
    assert_eq!(
        raider.weapons,
        vec![Weapon {
            damage: 14,
            accuracy: 0.65
        }]
    );
}

fn fitted(ship: &mut Ship, buy: &[&str], sell: &[&str]) {
    let mut commander = crate::game::Commander::new("Test".to_string());
    commander.credits = 10_000;
    for id in buy {
        let item = crate::game::equipment::find_item(id).unwrap();
        assert!(crate::game::equipment::buy(&mut commander, ship, item).is_ok());
    }
    for id in sell {
        let item = crate::game::equipment::find_item(id).unwrap();
        assert!(crate::game::equipment::sell(&mut commander, ship, item).is_ok());
    }
}

#[test]
fn a_new_ship_fights_with_its_starting_equipment() {
    let player = Combatant::from_ship(&Ship::new_cobra_mk3());

    assert_eq!(player.hull, 100);
    assert_eq!(
        (player.shield, player.max_shield, player.recharge),
        (20, 20, 5)
    );
    assert_eq!(
        player.weapons,
        vec![Weapon {
            damage: 15,
            accuracy: 0.9
        }]
    );
}

#[test]
fn every_installed_weapon_becomes_a_weapon() {
    let mut ship = Ship::new_cobra_mk3();
    fitted(&mut ship, &["beam-laser"], &[]);

    let player = Combatant::from_ship(&ship);

    assert_eq!(player.weapons.len(), 2);
    assert!(player.weapons.contains(&Weapon {
        damage: 25,
        accuracy: 0.8
    }));
}

#[test]
fn a_better_shield_changes_protection_and_recharge() {
    let mut ship = Ship::new_cobra_mk3();
    fitted(&mut ship, &["elite-shield"], &[]);

    let player = Combatant::from_ship(&ship);

    assert_eq!((player.shield, player.recharge), (100, 15));
}

#[test]
fn no_shield_means_no_shield_points() {
    let mut ship = Ship::new_cobra_mk3();
    fitted(&mut ship, &[], &["basic-shield"]);

    let player = Combatant::from_ship(&ship);

    assert_eq!(
        (player.shield, player.max_shield, player.recharge),
        (0, 0, 0)
    );
}

#[test]
fn no_weapons_means_nothing_to_fire() {
    let mut ship = Ship::new_cobra_mk3();
    fitted(&mut ship, &[], &["pulse-laser"]);

    assert!(Combatant::from_ship(&ship).weapons.is_empty());
}

#[test]
fn hull_follows_the_ships_damage() {
    let mut ship = Ship::new_cobra_mk3();
    ship.take_damage(30);

    assert_eq!(Combatant::from_ship(&ship).hull, 70);
}

#[test]
fn non_combat_equipment_does_not_change_the_fighter() {
    let mut ship = Ship::new_cobra_mk3();
    let before = Combatant::from_ship(&ship);
    fitted(
        &mut ship,
        &["elite-racks", "performance-drive", "military-scanner"],
        &[],
    );

    assert_eq!(Combatant::from_ship(&ship), before);
}

fn timid() -> Behaviour {
    Behaviour {
        aggression: 0.0,
        flee_below_hull: 0.5,
        flee_odds: 0.0,
        flee_chance: 1.0,
        surrender_below_hull: 0.0,
        surrender_chance: 0.0,
    }
}

fn wounded_raider(hull: u32) -> Combatant {
    Combatant {
        hull,
        ..fighter("Raider", 60, 0, 0, &[(10, 1.0)])
    }
}

#[test]
fn a_wounded_enemy_can_get_away_and_the_fight_ends() {
    let mut combat = Combat::with_behaviour(
        fighter("You", 100, 0, 0, &[(5, 0.1)]),
        wounded_raider(20),
        timid(),
    );

    // player misses, the raider decides to flee, then slips away
    let outcome = combat
        .resolve_round(Action::Attack, &mut rolls(&[0.9, 0.0, 0.1]))
        .unwrap();

    assert_eq!(outcome, Outcome::EnemyFled);
    assert_eq!(combat.last_npc_action, Some(NpcAction::Flee));
    assert_eq!(combat.player.hull, 100);
}

#[test]
fn an_enemy_that_fails_to_flee_loses_its_turn() {
    let mut combat = Combat::with_behaviour(
        fighter("You", 100, 0, 0, &[(5, 0.1)]),
        wounded_raider(20),
        timid(),
    );

    let outcome = combat
        .resolve_round(Action::Attack, &mut rolls(&[0.9, 0.0, 0.9]))
        .unwrap();

    assert_eq!(outcome, Outcome::Ongoing);
    assert_eq!(combat.player.hull, 100);
    assert!(combat.log.iter().any(|line| line.contains("fails")));
}

#[test]
fn a_beaten_enemy_can_surrender() {
    let behaviour = Behaviour {
        surrender_below_hull: 0.5,
        surrender_chance: 1.0,
        ..timid()
    };
    let mut combat = Combat::with_behaviour(
        fighter("You", 100, 0, 0, &[(5, 0.1)]),
        wounded_raider(20),
        behaviour,
    );

    let outcome = combat
        .resolve_round(Action::Attack, &mut rolls(&[0.9, 0.0]))
        .unwrap();

    assert_eq!(outcome, Outcome::EnemySurrendered);
    assert_eq!(combat.last_npc_action, Some(NpcAction::Surrender));
}

#[test]
fn an_enemy_that_never_retreats_uses_no_extra_rolls() {
    let mut combat = Combat::new(fighter("You", 100, 0, 0, &[(5, 0.1)]), wounded_raider(5));

    combat
        .resolve_round(Action::Attack, &mut rolls(&[0.9, 0.0]))
        .unwrap();

    assert_eq!(combat.last_npc_action, Some(NpcAction::Attack));
}

#[test]
fn an_unarmed_trader_may_give_in_at_the_first_shot() {
    let trader = crate::data::enemies::find_enemy("trader").unwrap();
    let mut combat = Combat::against(fighter("You", 100, 0, 0, &[(5, 1.0)]), trader);

    let outcome = combat
        .resolve_round(Action::Attack, &mut rolls(&[0.0, 0.1]))
        .unwrap();

    assert_eq!(outcome, Outcome::EnemySurrendered);
    assert_eq!(combat.player.hull, 100);
}

#[test]
fn a_scaled_enemy_has_more_health_and_less_aggression() {
    let raider = crate::data::enemies::find_enemy("pirate-raider").unwrap();
    let difficulty = Difficulty {
        health: 1.5,
        nerve: 0.3,
    };

    let combat = Combat::against_scaled(fighter("You", 100, 0, 0, &[(5, 1.0)]), raider, difficulty);

    assert_eq!((combat.enemy.hull, combat.enemy.max_hull), (105, 105));
    assert_eq!((combat.enemy.shield, combat.enemy.max_shield), (38, 38));
}
