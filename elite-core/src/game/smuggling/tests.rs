use super::*;
use crate::data::commodities::CommodityType;
use crate::game::commander::CombatRating;
use crate::game::jobs::{Job, JobKind};
use rand::rngs::StdRng;
use rand::SeedableRng;

fn run(destination: &str, tons: u32, reward: u32) -> ActiveJob {
    ActiveJob {
        job: Job {
            kind: JobKind::Smuggling,
            tons,
            commodity: Some(CommodityType::Narcotics),
            destination: destination.to_string(),
            distance: 2.3,
            fuel_cost: 3,
            reward,
            deadline_jumps: 3,
            min_rating: CombatRating::Harmless,
        },
        jumps_left: 1,
    }
}

fn loaded(tons: u32) -> (Commander, Ship) {
    let mut ship = Ship::new_cobra_mk3();
    assert!(ship.cargo_hold.reserve(tons));
    (Commander::new("Test".to_string()), ship)
}

/// A government where an inspection always (chance 1) or never (chance 0) happens, by seed
fn rng_that_inspects() -> StdRng {
    (0..1000u64)
        .map(StdRng::seed_from_u64)
        .find(|rng| rng.clone().gen::<f64>() < 0.25)
        .unwrap()
}

fn rng_that_waves_through() -> StdRng {
    (0..1000u64)
        .map(StdRng::seed_from_u64)
        .find(|rng| rng.clone().gen::<f64>() >= 0.6)
        .unwrap()
}

fn never_asked() -> impl FnMut(u32, bool) -> InspectionChoice {
    |_, _| panic!("the player should not have been asked")
}

fn never_fights() -> impl FnMut(&Ship) -> std::io::Result<(Outcome, u32)> {
    |_| panic!("there should be no fight")
}

fn fights(outcome: Outcome, hull: u32) -> impl FnMut(&Ship) -> std::io::Result<(Outcome, u32)> {
    move |_| Ok((outcome, hull))
}

fn government() -> GovernmentType {
    GovernmentType::Democracy
}

#[test]
fn inspection_odds_follow_the_government() {
    assert_eq!(inspection_chance(&GovernmentType::Communist), 0.6);
    assert_eq!(inspection_chance(&GovernmentType::Dictatorship), 0.6);
    assert_eq!(inspection_chance(&GovernmentType::CorporateState), 0.4);
    assert_eq!(inspection_chance(&GovernmentType::Democracy), 0.4);
    assert_eq!(inspection_chance(&GovernmentType::Confederacy), 0.25);
    assert_eq!(inspection_chance(&GovernmentType::MultiGovernment), 0.25);
}

#[test]
fn an_uninspected_run_pays_in_full_and_costs_infamy() {
    let (mut commander, mut ship) = loaded(8);

    let messages = resolve_smuggling(
        vec![run("Proxima Centauri", 8, 120)],
        &mut commander,
        &mut ship,
        &GovernmentType::Communist,
        &mut rng_that_waves_through(),
        &mut never_asked(),
        &mut never_fights(),
    );

    assert_eq!(commander.credits(), 220);
    assert_eq!(commander.reputation, -3);
    assert_eq!(ship.cargo_hold.reserved, 0);
    assert!(messages.iter().any(|m| m.contains("Contraband delivered")));
}

#[test]
fn a_bribe_costs_forty_percent_and_the_run_succeeds() {
    let (mut commander, mut ship) = loaded(8);
    let mut asked = None;

    resolve_smuggling(
        vec![run("Proxima Centauri", 8, 100)],
        &mut commander,
        &mut ship,
        &government(),
        &mut rng_that_inspects(),
        &mut |cost, can| {
            asked = Some((cost, can));
            InspectionChoice::Bribe
        },
        &mut never_fights(),
    );

    assert_eq!(asked, Some((40, true)));
    assert_eq!(commander.credits(), 160);
    assert_eq!(commander.reputation, -3);
    assert_eq!(ship.cargo_hold.reserved, 0);
}

#[test]
fn a_bribe_the_player_cannot_afford_hands_the_cargo_over() {
    let (mut commander, mut ship) = loaded(8);
    commander.remove_credits(80);

    let mut asked = None;
    resolve_smuggling(
        vec![run("Proxima Centauri", 8, 100)],
        &mut commander,
        &mut ship,
        &government(),
        &mut rng_that_inspects(),
        &mut |cost, can| {
            asked = Some((cost, can));
            InspectionChoice::Bribe
        },
        &mut never_fights(),
    );

    assert_eq!(asked, Some((40, false)));
    assert_eq!(commander.credits(), 0);
    assert_eq!(ship.cargo_hold.reserved, 0);
}

#[test]
fn handing_over_the_cargo_fails_the_jobs_with_a_fine_capped_by_credits() {
    let (mut commander, mut ship) = loaded(16);

    resolve_smuggling(
        vec![
            run("Proxima Centauri", 8, 100),
            run("Proxima Centauri", 8, 60),
        ],
        &mut commander,
        &mut ship,
        &government(),
        &mut rng_that_inspects(),
        &mut |_, _| InspectionChoice::HandOver,
        &mut never_fights(),
    );

    assert_eq!(commander.credits(), 60); // fine 25 + 15
    assert_eq!(commander.reputation, -5); // once for the whole inspection
    assert_eq!(ship.cargo_hold.reserved, 0);

    let (mut broke, mut ship) = loaded(8);
    broke.remove_credits(90);
    resolve_smuggling(
        vec![run("Proxima Centauri", 8, 100)],
        &mut broke,
        &mut ship,
        &government(),
        &mut rng_that_inspects(),
        &mut |_, _| InspectionChoice::HandOver,
        &mut never_fights(),
    );
    assert_eq!(broke.credits(), 0);
}

#[test]
fn running_and_winning_pays_in_full_with_both_reputation_costs() {
    let (mut commander, mut ship) = loaded(8);

    resolve_smuggling(
        vec![run("Proxima Centauri", 8, 120)],
        &mut commander,
        &mut ship,
        &government(),
        &mut rng_that_inspects(),
        &mut |_, _| InspectionChoice::Run,
        &mut fights(Outcome::Victory, 55),
    );

    assert_eq!(commander.credits(), 220);
    assert_eq!(commander.reputation, -8); // −3 for the delivery, −5 for attacking the law
    assert_eq!(ship.hull_percentage, 55);
    assert_eq!(ship.cargo_hold.reserved, 0);
}

#[test]
fn escaping_or_surrendering_fails_the_jobs_with_the_fine_and_keeps_the_hull_damage() {
    for outcome in [Outcome::Escaped, Outcome::Surrendered] {
        let (mut commander, mut ship) = loaded(8);

        resolve_smuggling(
            vec![run("Proxima Centauri", 8, 100)],
            &mut commander,
            &mut ship,
            &government(),
            &mut rng_that_inspects(),
            &mut |_, _| InspectionChoice::Run,
            &mut fights(outcome, 62),
        );

        assert_eq!(commander.credits(), 75, "{:?}", outcome);
        assert_eq!(commander.reputation, -5, "{:?}", outcome);
        assert_eq!(ship.hull_percentage, 62, "{:?}", outcome);
        assert_eq!(ship.cargo_hold.reserved, 0, "{:?}", outcome);
    }
}

#[test]
fn losing_the_fight_applies_only_the_combat_defeat_rules() {
    let (mut commander, mut ship) = loaded(8);
    ship.add_cargo(CommodityType::Food, 2);

    resolve_smuggling(
        vec![run("Proxima Centauri", 8, 100)],
        &mut commander,
        &mut ship,
        &government(),
        &mut rng_that_inspects(),
        &mut |_, _| InspectionChoice::Run,
        &mut fights(Outcome::Defeat, 0),
    );

    assert_eq!(commander.credits(), 50); // half, with no extra fine
    assert_eq!(commander.reputation, -5); // the defeat penalty once, nothing stacked
    assert_eq!(ship.cargo_used(), 0);
    assert_eq!(ship.hull_percentage, 10);
}
