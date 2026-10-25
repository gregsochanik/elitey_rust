use super::*;
use crate::data::systems::GovernmentType;
use crate::game::combat::{Action, Weapon};
use crate::game::commander::CombatRating;
use crate::game::events::EventType;
use crate::game::jobs::{Job, JobKind};
use crate::game::smuggling;
use rand::rngs::StdRng;
use rand::SeedableRng;

fn game() -> GameState {
    let mut game = GameState::new("Test".to_string());
    game.commander.credits = 1_000;
    game
}

fn first_step(seed: u64) -> (GameState, Journey, Step) {
    let mut game = game();
    let plan = game.plan_jump("Proxima Centauri").unwrap();
    let (journey, step) = game.start_jump(plan, &mut StdRng::seed_from_u64(seed));
    (game, journey, step)
}

fn seed_where(wanted: impl Fn(&Step) -> bool) -> u64 {
    (0..2_000)
        .find(|seed| wanted(&first_step(*seed).2))
        .expect("a seed that gives the wanted first step")
}

fn won_fight(mut combat: Combat) -> Combat {
    combat.enemy.hull = 1;
    combat.player.weapons = vec![Weapon {
        damage: 50,
        accuracy: 1.0,
    }];
    combat
        .resolve_round(Action::Attack, &mut || 0.0)
        .expect("the attack lands");
    assert_eq!(combat.outcome(), Outcome::Victory);
    combat
}

#[test]
fn a_quiet_jump_costs_fuel_counts_a_jump_and_arrives() {
    let seed = seed_where(|step| matches!(step, Step::Arrived { .. }));
    let (game, mut journey, step) = first_step(seed);

    let Step::Arrived {
        arrived_at,
        planned,
        eventful,
    } = step
    else {
        panic!("expected arrival");
    };
    assert_eq!(
        (arrived_at.as_str(), planned.as_str()),
        ("Proxima Centauri", "Proxima Centauri")
    );
    assert!(!eventful);
    assert_eq!(game.commander.location, "Proxima Centauri");
    assert_eq!(game.ship.fuel_current, 7 - 3);
    assert_eq!(game.world_clock, 1);
    assert!(journey.take_notices().contains(&Notice::Success(
        "Arrived safely at Proxima Centauri.".to_string()
    )));
}

#[test]
fn an_event_waits_for_a_choice_then_the_journey_finishes() {
    let seed = seed_where(|step| matches!(step, Step::Event(_)));
    let (mut game, mut journey, step) = first_step(seed);
    let Step::Event(event) = step else {
        panic!("expected an event");
    };
    assert!(!event.choices.is_empty());
    assert_eq!(game.commander.location, "Sol");

    let mut rng = StdRng::seed_from_u64(seed + 1);
    assert_eq!(
        game.advance_journey(&mut journey, Input::Choice(99), &mut rng)
            .unwrap_err(),
        JourneyError::InvalidChoice
    );
    assert_eq!(
        game.advance_journey(&mut journey, Input::Continue, &mut rng)
            .unwrap_err(),
        JourneyError::WrongInput
    );

    let step = game
        .advance_journey(&mut journey, Input::Choice(0), &mut rng)
        .unwrap();
    let step = game.finish_journey_automatically(&mut journey, step, &mut rng);
    assert!(matches!(step, Step::Arrived { .. }));
    assert_eq!(game.world_clock, 1);
}

#[test]
fn entering_a_wormhole_lands_somewhere_else_and_the_jump_still_costs_its_fuel() {
    let seed = seed_where(
        |step| matches!(step, Step::Event(event) if event.event_type == EventType::Wormhole),
    );
    let (mut game, mut journey, _) = first_step(seed);
    let mut rng = StdRng::seed_from_u64(seed + 1);

    let step = game
        .advance_journey(&mut journey, Input::Choice(0), &mut rng)
        .unwrap();
    let Step::Arrived {
        arrived_at,
        planned,
        ..
    } = game.finish_journey_automatically(&mut journey, step, &mut rng)
    else {
        panic!("expected arrival");
    };

    assert_eq!(planned, "Proxima Centauri");
    assert!(arrived_at != "Sol" && arrived_at != "Proxima Centauri");
    assert_eq!(game.commander.location, arrived_at);
    assert!(game.ship.fuel_current <= 7 - 3 - 2);
    assert!(journey.take_notices().contains(&Notice::Info(format!(
        "You emerge at {}, not Proxima Centauri.",
        arrived_at
    ))));
}

#[test]
fn a_bounty_at_the_destination_needs_a_fight_and_pays_on_victory() {
    let seed = seed_where(|step| matches!(step, Step::Arrived { .. }));
    let mut game = game();
    game.active_jobs.push(ActiveJob {
        job: Job {
            kind: JobKind::Bounty,
            tons: 0,
            commodity: None,
            destination: "Proxima Centauri".to_string(),
            distance: 2.3,
            fuel_cost: 3,
            reward: 300,
            deadline_jumps: 3,
            min_rating: CombatRating::Harmless,
        },
        jumps_left: 3,
    });
    let credits = game.commander.credits();
    let plan = game.plan_jump("Proxima Centauri").unwrap();
    let mut rng = StdRng::seed_from_u64(seed);

    let (mut journey, step) = game.start_jump(plan, &mut rng);
    let Step::Fight(combat) = step else {
        panic!("expected a fight, got {:?}", step);
    };
    assert_eq!(combat.enemy.name, "Pirate Raider");

    let step = game
        .advance_journey(&mut journey, Input::FightEnded(won_fight(combat)), &mut rng)
        .unwrap();

    assert!(matches!(step, Step::Arrived { .. }));
    assert_eq!(game.commander.credits(), credits + 300);
    assert_eq!(game.commander.kills, 1);
    assert!(game.active_jobs.is_empty());
}

#[test]
fn an_interrupted_bounty_fight_fails_the_job() {
    let seed = seed_where(|step| matches!(step, Step::Arrived { .. }));
    let mut game = game();
    game.active_jobs.push(ActiveJob {
        job: Job {
            kind: JobKind::Bounty,
            tons: 0,
            commodity: None,
            destination: "Proxima Centauri".to_string(),
            distance: 2.3,
            fuel_cost: 3,
            reward: 300,
            deadline_jumps: 3,
            min_rating: CombatRating::Harmless,
        },
        jumps_left: 3,
    });
    let plan = game.plan_jump("Proxima Centauri").unwrap();
    let mut rng = StdRng::seed_from_u64(seed);
    let (mut journey, _) = game.start_jump(plan, &mut rng);

    game.advance_journey(
        &mut journey,
        Input::FightInterrupted("input closed".to_string()),
        &mut rng,
    )
    .unwrap();

    assert!(game.active_jobs.is_empty());
    assert_eq!(game.commander.reputation, -3);
    assert!(journey.take_notices().contains(&Notice::Text(
        "The bounty near Proxima Centauri failed.".to_string()
    )));
}

#[test]
fn settling_jobs_without_a_jump_delivers_a_parcel() {
    let mut game = game();
    game.active_jobs.push(ActiveJob {
        job: Job {
            kind: JobKind::Courier,
            tons: 0,
            commodity: None,
            destination: "Sol".to_string(),
            distance: 2.3,
            fuel_cost: 3,
            reward: 40,
            deadline_jumps: 3,
            min_rating: CombatRating::Harmless,
        },
        jumps_left: 3,
    });
    let credits = game.commander.credits();

    game.settle_jobs_with(&mut StdRng::seed_from_u64(1));

    assert_eq!(game.commander.credits(), credits + 40);
    assert_eq!(game.world_clock, 0);
}

#[test]
fn the_automatic_fight_ends_and_an_unarmed_ship_surrenders() {
    let mut rng = StdRng::seed_from_u64(4);
    let mut armed = Combat::new(
        Combatant::from_ship(&game().ship),
        Combatant::pirate_raider(),
    );
    auto_fight(&mut armed, &mut rng);
    assert_ne!(armed.outcome(), Outcome::Ongoing);

    let mut unarmed = Combat::new(Combatant::pirate_raider(), Combatant::pirate_raider());
    unarmed.player.weapons.clear();
    auto_fight(&mut unarmed, &mut rng);
    assert_eq!(unarmed.outcome(), Outcome::Surrendered);
}

fn game_with_contraband() -> GameState {
    let mut game = game();
    assert!(game.ship.cargo_hold.reserve(5));
    game.active_jobs.push(ActiveJob {
        job: Job {
            kind: JobKind::Smuggling,
            tons: 5,
            commodity: None,
            destination: "Proxima Centauri".to_string(),
            distance: 2.3,
            fuel_cost: 3,
            reward: 200,
            deadline_jumps: 3,
            min_rating: CombatRating::Harmless,
        },
        jumps_left: 3,
    });
    game
}

fn inspected_journey(answer: smuggling::InspectionChoice) -> (GameState, u32, Journey) {
    for seed in 0..2_000 {
        let mut game = game_with_contraband();
        let credits = game.commander.credits();
        let plan = game.plan_jump("Proxima Centauri").unwrap();
        let mut rng = StdRng::seed_from_u64(seed);
        let (mut journey, step) = game.start_jump(plan, &mut rng);
        if let Step::Inspection {
            bribe_cost,
            can_bribe,
        } = step
        {
            assert_eq!(bribe_cost, 80);
            assert!(can_bribe);
            let step = game
                .advance_journey(&mut journey, Input::Inspection(answer), &mut rng)
                .unwrap();
            if matches!(step, Step::Arrived { .. }) {
                return (game, credits, journey);
            }
        }
    }
    panic!("no seed produced an inspection");
}

#[test]
fn a_stopped_smuggler_who_bribes_pays_and_delivers() {
    let (game, credits, _) = inspected_journey(smuggling::InspectionChoice::Bribe);

    assert_eq!(game.commander.credits(), credits + 200 - 80);
    assert!(game.active_jobs.is_empty());
    assert_eq!(game.ship.cargo_hold.reserved, 0);
}

#[test]
fn a_stopped_smuggler_who_hands_over_the_cargo_is_fined() {
    let (game, credits, mut journey) = inspected_journey(smuggling::InspectionChoice::HandOver);

    assert_eq!(game.commander.credits(), credits - 50);
    assert_eq!(game.commander.reputation, -5);
    assert!(journey.take_notices().contains(&Notice::Text(
        "The contraband is confiscated. Fine: 50cr.".to_string()
    )));
}

#[test]
fn a_smuggler_who_runs_fights_a_patrol_vessel() {
    for seed in 0..2_000 {
        let mut game = game_with_contraband();
        let plan = game.plan_jump("Proxima Centauri").unwrap();
        let mut rng = StdRng::seed_from_u64(seed);
        let (mut journey, step) = game.start_jump(plan, &mut rng);
        if !matches!(step, Step::Inspection { .. }) {
            continue;
        }
        let Step::Fight(combat) = game
            .advance_journey(
                &mut journey,
                Input::Inspection(smuggling::InspectionChoice::Run),
                &mut rng,
            )
            .unwrap()
        else {
            panic!("running starts a fight");
        };
        assert_eq!(combat.enemy.name, "Patrol Vessel");

        let step = game
            .advance_journey(&mut journey, Input::FightEnded(won_fight(combat)), &mut rng)
            .unwrap();

        assert!(matches!(step, Step::Arrived { .. }));
        assert!(game.active_jobs.is_empty());
        assert_eq!(game.commander.credits(), 1_000 + 200);
        assert_eq!(game.commander.kills, 1);
        return;
    }
    panic!("no seed produced an inspection");
}

#[test]
fn engaging_a_patrol_or_robbing_a_convoy_starts_the_matching_fight() {
    for (event_type, choice, enemy) in [
        (
            EventType::MilitaryPatrol,
            "Engage the patrol",
            "Patrol Vessel",
        ),
        (EventType::TraderConvoy, "Rob the convoy", "Trader Convoy"),
    ] {
        let mut game = game();
        let event = game
            .event_manager
            .get_available_events(&EventPhase::Approach)
            .into_iter()
            .find(|event| event.event_type == event_type)
            .unwrap()
            .clone();
        let index = event
            .choices
            .iter()
            .position(|c| c.description == choice)
            .unwrap();
        let plan = game.plan_jump("Proxima Centauri").unwrap();
        let mut journey = Journey::new(
            plan,
            Stage::AwaitEvent {
                event,
                phase: EventPhase::Approach,
            },
        );

        let step = game
            .advance_journey(
                &mut journey,
                Input::Choice(index),
                &mut StdRng::seed_from_u64(1),
            )
            .unwrap();

        let Step::Fight(combat) = step else {
            panic!("{choice} starts a fight");
        };
        assert_eq!(combat.enemy.name, enemy);
    }
}

fn open_a_patrol(
    standing_score: i32,
    government: GovernmentType,
) -> (GameState, Journey, Option<Step>) {
    let mut game = game();
    game.commander.reputation = standing_score;
    let event = game
        .event_manager
        .get_available_events(&EventPhase::Approach)
        .into_iter()
        .find(|event| event.event_type == EventType::MilitaryPatrol)
        .unwrap()
        .clone();
    let plan = game.plan_jump("Proxima Centauri").unwrap();
    let mut journey = Journey::new(plan, Stage::Start);
    let mut system = game.get_system_info("Proxima Centauri").unwrap().clone();
    system.government = government;
    let step = game.open_event(&mut journey, event, EventPhase::Approach, &system);
    (game, journey, step)
}

#[test]
fn a_wanted_commander_is_attacked_on_sight_in_a_lawful_system() {
    let (_, _, step) = open_a_patrol(-250, GovernmentType::Democracy);

    let Some(Step::Fight(combat)) = step else {
        panic!("the patrol opens fire");
    };
    assert_eq!(combat.enemy.name, "Patrol Vessel");
}

#[test]
fn a_respected_commander_is_waved_through() {
    let (game, mut journey, step) = open_a_patrol(90, GovernmentType::Democracy);

    assert!(step.is_none());
    assert_eq!(game.commander.reputation, 92);
    assert!(journey.take_notices().iter().any(
        |notice| matches!(notice, Notice::Success(text) if text.contains("waves you through"))
    ));
}

#[test]
fn everyone_else_gets_the_patrol_event_and_lawless_systems_do_not_care() {
    for (score, government) in [
        (0, GovernmentType::Democracy),
        (-250, GovernmentType::Anarchy),
        (90, GovernmentType::Feudal),
    ] {
        let (_, _, step) = open_a_patrol(score, government);

        assert!(matches!(step, Some(Step::Event(_))), "{score}");
    }
}
