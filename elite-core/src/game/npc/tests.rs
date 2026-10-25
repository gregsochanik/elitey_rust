use super::*;
use crate::data::enemies::Behaviour;

fn fighter(hull: u32, max_hull: u32, shield: u32, weapons: &[(u32, f64)]) -> Combatant {
    Combatant {
        name: "Test".to_string(),
        hull,
        max_hull,
        shield,
        max_shield: shield,
        recharge: 0,
        weapons: weapons
            .iter()
            .map(|&(damage, accuracy)| Weapon { damage, accuracy })
            .collect(),
    }
}

fn player() -> Combatant {
    fighter(100, 100, 25, &[(14, 0.65)])
}

fn timid() -> Behaviour {
    Behaviour {
        aggression: 0.0,
        flee_below_hull: 0.3,
        flee_odds: 0.0,
        flee_chance: 0.5,
        surrender_below_hull: 0.1,
        surrender_chance: 0.5,
    }
}

fn decide_with(behaviour: &Behaviour, npc: &Combatant, rolls: &[f64]) -> (NpcAction, usize) {
    let mut taken = 0;
    let mut roll = || {
        let value = rolls[taken];
        taken += 1;
        value
    };
    let action = decide(behaviour, npc, &player(), &mut roll);
    (action, taken)
}

#[test]
fn a_healthy_enemy_attacks_without_rolling() {
    let npc = fighter(70, 70, 25, &[(14, 0.65)]);
    assert_eq!(decide_with(&timid(), &npc, &[]), (NpcAction::Attack, 0));
}

#[test]
fn an_enemy_that_never_retreats_attacks_however_hurt() {
    let npc = fighter(1, 70, 0, &[(14, 0.65)]);
    assert_eq!(
        decide_with(&Behaviour::NEVER_RETREATS, &npc, &[]),
        (NpcAction::Attack, 0)
    );
}

#[test]
fn flight_starts_at_the_hull_threshold() {
    let at = fighter(21, 70, 0, &[(14, 0.65)]);
    let above = fighter(22, 70, 0, &[(14, 0.65)]);
    assert_eq!(decide_with(&timid(), &at, &[0.4]), (NpcAction::Flee, 1));
    assert_eq!(decide_with(&timid(), &above, &[]), (NpcAction::Attack, 0));
}

#[test]
fn flight_depends_on_the_roll_against_flee_chance() {
    let npc = fighter(20, 70, 0, &[(14, 0.65)]);
    assert_eq!(decide_with(&timid(), &npc, &[0.49]).0, NpcAction::Flee);
    assert_eq!(decide_with(&timid(), &npc, &[0.5]).0, NpcAction::Attack);
}

#[test]
fn surrender_is_tried_before_flight() {
    let npc = fighter(7, 70, 0, &[(14, 0.65)]);
    assert_eq!(
        decide_with(&timid(), &npc, &[0.2]),
        (NpcAction::Surrender, 1)
    );
    assert_eq!(
        decide_with(&timid(), &npc, &[0.9, 0.2]),
        (NpcAction::Flee, 2)
    );
    assert_eq!(
        decide_with(&timid(), &npc, &[0.9, 0.9]),
        (NpcAction::Attack, 2)
    );
}

#[test]
fn surrender_starts_at_its_own_hull_threshold() {
    let at = fighter(7, 70, 0, &[(14, 0.65)]);
    let above = fighter(8, 70, 0, &[(14, 0.65)]);
    assert_eq!(decide_with(&timid(), &at, &[0.0]).0, NpcAction::Surrender);
    assert_eq!(decide_with(&timid(), &above, &[0.0]).0, NpcAction::Flee);
}

#[test]
fn an_enemy_with_no_weapons_may_surrender_at_full_hull() {
    let npc = fighter(70, 70, 25, &[]);
    assert_eq!(
        decide_with(&Behaviour::NEVER_RETREATS, &npc, &[0.1]),
        (NpcAction::Surrender, 1)
    );
}

#[test]
fn an_outmatched_enemy_flees_by_the_odds() {
    let behaviour = Behaviour {
        aggression: 0.0,
        flee_below_hull: 0.0,
        flee_odds: 0.5,
        flee_chance: 1.0,
        surrender_below_hull: 0.0,
        surrender_chance: 0.0,
    };
    let weak = fighter(20, 70, 0, &[(5, 0.5)]);
    let strong = fighter(70, 70, 25, &[(20, 0.9)]);
    assert_eq!(decide_with(&behaviour, &weak, &[0.0]), (NpcAction::Flee, 1));
    assert_eq!(
        decide_with(&behaviour, &strong, &[]),
        (NpcAction::Attack, 0)
    );
}

#[test]
fn aggression_lowers_the_flee_thresholds() {
    let npc = fighter(20, 70, 0, &[(14, 0.65)]);
    let bold = Behaviour {
        aggression: 0.5,
        ..timid()
    };
    let bolder = Behaviour {
        aggression: 0.8,
        ..timid()
    };
    assert_eq!(decide_with(&bold, &npc, &[0.0, 0.0]).0, NpcAction::Attack);
    assert_eq!(decide_with(&bolder, &npc, &[]), (NpcAction::Attack, 0));
    assert_eq!(decide_with(&timid(), &npc, &[0.0]).0, NpcAction::Flee);
}

#[test]
fn an_unarmed_player_makes_the_odds_infinite() {
    let behaviour = Behaviour {
        flee_odds: 5.0,
        aggression: 0.0,
        flee_below_hull: 0.0,
        surrender_below_hull: 0.0,
        ..timid()
    };
    let npc = fighter(20, 70, 0, &[(5, 0.5)]);
    let unarmed = fighter(100, 100, 0, &[]);
    let mut roll = || panic!("no roll expected");
    assert_eq!(
        decide(&behaviour, &npc, &unarmed, &mut roll),
        NpcAction::Attack
    );
}

#[test]
fn the_shipped_enemies_match_the_original_raider() {
    let raider = Combatant::pirate_raider();
    assert_eq!(
        (
            raider.name.as_str(),
            raider.hull,
            raider.max_hull,
            raider.shield,
            raider.recharge
        ),
        ("Pirate Raider", 70, 70, 25, 4)
    );
    assert_eq!(
        raider.weapons,
        vec![Weapon {
            damage: 14,
            accuracy: 0.65
        }]
    );
    assert_eq!(Combatant::patrol_vessel().name, "Patrol Vessel");
}
