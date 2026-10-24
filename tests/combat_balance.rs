use elitey_rust::game::combat::{Action, Combat, CombatError, Combatant, Outcome};
use elitey_rust::game::equipment::EquipmentHandler;
use elitey_rust::game::{Commander, Ship};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

const FIGHTS: u64 = 1_000;

struct Results {
    win_rate: f64,
    average_hull_lost: f64,
}

fn ship_with(commands: &[&str]) -> Ship {
    let mut ship = Ship::new_cobra_mk3();
    let mut commander = Commander::new("Test".to_string());
    commander.credits = 1_000_000;
    for command in commands {
        let args: Vec<String> = command.split_whitespace().map(String::from).collect();
        assert!(
            EquipmentHandler::handle(&args, &mut commander, &mut ship),
            "{}",
            command
        );
    }
    ship
}

/// Fight the Pirate Raider to the end, attacking every round
fn fight_many(ship: &Ship) -> Results {
    let (mut wins, mut hull_lost) = (0u64, 0u64);
    for seed in 0..FIGHTS {
        let mut combat = Combat::new(Combatant::from_ship(ship), Combatant::pirate_raider());
        let start = combat.player.hull;
        let mut rng = StdRng::seed_from_u64(seed);
        let mut roll = || rng.gen::<f64>();

        for _ in 0..500 {
            match combat.resolve_round(Action::Attack, &mut roll) {
                Ok(Outcome::Ongoing) => continue,
                Ok(Outcome::Victory) => wins += 1,
                _ => {}
            }
            break;
        }
        hull_lost += u64::from(start - combat.player.hull);
    }
    Results {
        win_rate: wins as f64 / FIGHTS as f64,
        average_hull_lost: hull_lost as f64 / FIGHTS as f64,
    }
}

#[test]
fn the_starting_loadout_usually_wins_and_loses_some_hull() {
    let results = fight_many(&ship_with(&[]));

    assert!(results.win_rate >= 0.9, "win rate {}", results.win_rate);
    assert!(
        (15.0..=45.0).contains(&results.average_hull_lost),
        "average hull lost {}",
        results.average_hull_lost
    );
}

#[test]
fn an_unarmed_ship_cannot_attack_and_so_cannot_win() {
    let ship = ship_with(&["sell pulse-laser"]);
    let mut combat = Combat::new(Combatant::from_ship(&ship), Combatant::pirate_raider());

    assert_eq!(
        combat.resolve_round(Action::Attack, &mut || 0.0),
        Err(CombatError::NoWeapons)
    );
}

#[test]
fn a_better_shield_loses_less_hull() {
    let start = fight_many(&ship_with(&[]));
    let standard = fight_many(&ship_with(&["buy standard-shield"]));
    let military = fight_many(&ship_with(&["buy military-shield"]));

    assert!(standard.average_hull_lost < start.average_hull_lost);
    assert!(military.average_hull_lost <= standard.average_hull_lost);
}

#[test]
fn better_weapons_lose_less_hull_and_win_at_least_as_often() {
    let start = fight_many(&ship_with(&[]));
    let two_lasers = fight_many(&ship_with(&["buy beam-laser"]));

    assert!(two_lasers.average_hull_lost < start.average_hull_lost);
    assert!(two_lasers.win_rate >= start.win_rate);
}

#[test]
fn going_without_a_shield_is_much_worse() {
    let start = fight_many(&ship_with(&[]));
    let bare = fight_many(&ship_with(&["sell basic-shield"]));

    assert!(bare.win_rate < start.win_rate);
    assert!(bare.average_hull_lost > start.average_hull_lost + 20.0);
}

#[test]
fn the_best_loadout_barely_scratches_the_hull() {
    let best = fight_many(&ship_with(&[
        "sell pulse-laser",
        "buy beam-laser",
        "buy plasma-cannon",
        "buy standard-shield",
    ]));

    assert!(best.win_rate >= 0.99);
    assert!(best.average_hull_lost < 10.0, "{}", best.average_hull_lost);
}
