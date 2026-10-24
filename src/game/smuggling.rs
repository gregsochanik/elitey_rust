use crate::data::systems::GovernmentType;
use crate::game::aftermath::settle_with_reward;
use crate::game::combat::Outcome;
use crate::game::jobs::ActiveJob;
use crate::game::{Commander, Ship};
use rand::Rng;

pub const SMUGGLING_REPUTATION: i32 = -3;
pub const ATTACKING_THE_LAW_REPUTATION: i32 = -5;
pub const INSPECTION_FAILURE_REPUTATION: i32 = -5;
const BRIBE_SHARE: f64 = 0.4;
const FINE_SHARE: f64 = 0.25;

/// How the player answers a patrol that has stopped them with contraband aboard
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InspectionChoice {
    Bribe,
    Run,
    HandOver,
}

/// Chance a smuggler is stopped on arriving at a system with this government
pub fn inspection_chance(government: &GovernmentType) -> f64 {
    match government {
        GovernmentType::Communist | GovernmentType::Dictatorship => 0.6,
        GovernmentType::CorporateState | GovernmentType::Democracy => 0.4,
        _ => 0.25,
    }
}

/// What a bribe costs for these jobs together
pub fn bribe_cost(jobs: &[ActiveJob]) -> u32 {
    let total: u32 = jobs.iter().map(|entry| entry.job.reward).sum();
    (f64::from(total) * BRIBE_SHARE).ceil() as u32
}

fn reputation_messages(
    messages: &mut Vec<String>,
    change: crate::game::commander::ReputationChange,
) {
    if change.applied != 0 {
        messages.push(format!("Reputation {:+}.", change.applied));
    }
    if let Some(standing) = change.new_standing {
        messages.push(format!("Your standing is now {}.", standing.name()));
    }
}

fn release_cargo(jobs: &[ActiveJob], ship: &mut Ship) {
    for entry in jobs {
        ship.cargo_hold.release(entry.job.tons);
    }
}

fn pay_all(
    jobs: &[ActiveJob],
    commander: &mut Commander,
    ship: &mut Ship,
    extra_reputation: i32,
    messages: &mut Vec<String>,
) {
    release_cargo(jobs, ship);
    for entry in jobs {
        commander.add_credits(entry.job.reward);
        messages.push(format!(
            "Contraband delivered to {}: +{}cr.",
            entry.job.destination, entry.job.reward
        ));
        reputation_messages(messages, commander.change_reputation(SMUGGLING_REPUTATION));
    }
    if extra_reputation != 0 {
        reputation_messages(messages, commander.change_reputation(extra_reputation));
    }
}

/// The contraband is lost: a fine of a quarter of each reward (never more than the player has)
/// and one reputation penalty for the whole inspection
fn fail_all(
    jobs: &[ActiveJob],
    commander: &mut Commander,
    ship: &mut Ship,
    messages: &mut Vec<String>,
) {
    release_cargo(jobs, ship);
    let fine: u32 = jobs
        .iter()
        .map(|entry| (f64::from(entry.job.reward) * FINE_SHARE).round() as u32)
        .sum();
    let fine = fine.min(commander.credits());
    commander.remove_credits(fine);
    messages.push(format!("The contraband is confiscated. Fine: {}cr.", fine));
    reputation_messages(
        messages,
        commander.change_reputation(INSPECTION_FAILURE_REPUTATION),
    );
}

/// Settle smuggling jobs that have reached a lawful system. At most one inspection covers all
/// of them. `choose` is asked what to do when inspected, given the bribe cost and whether the
/// player can afford it; `fight` plays a fight against a patrol vessel.
pub fn resolve_smuggling(
    jobs: Vec<ActiveJob>,
    commander: &mut Commander,
    ship: &mut Ship,
    government: &GovernmentType,
    rng: &mut impl Rng,
    choose: &mut impl FnMut(u32, bool) -> InspectionChoice,
    fight: &mut impl FnMut(&Ship) -> std::io::Result<(Outcome, u32)>,
) -> Vec<String> {
    let mut messages = Vec::new();
    if jobs.is_empty() {
        return messages;
    }

    if rng.gen::<f64>() >= inspection_chance(government) {
        pay_all(&jobs, commander, ship, 0, &mut messages);
        return messages;
    }

    messages.push("A patrol stops you and scans the hold.".to_string());
    let cost = bribe_cost(&jobs);
    let can_bribe = commander.credits() >= cost;
    let mut choice = choose(cost, can_bribe);
    if choice == InspectionChoice::Bribe && !can_bribe {
        choice = InspectionChoice::HandOver;
    }

    match choice {
        InspectionChoice::Bribe => {
            commander.remove_credits(cost);
            messages.push(format!(
                "You pay {}cr and the patrol waves you through.",
                cost
            ));
            pay_all(&jobs, commander, ship, 0, &mut messages);
        }
        InspectionChoice::HandOver => fail_all(&jobs, commander, ship, &mut messages),
        InspectionChoice::Run => match fight(ship) {
            Ok((Outcome::Victory, hull)) => {
                ship.hull_percentage = hull.min(ship.ship_type.max_hull());
                messages.push("You destroy the patrol vessel and make the drop.".to_string());
                pay_all(
                    &jobs,
                    commander,
                    ship,
                    ATTACKING_THE_LAW_REPUTATION,
                    &mut messages,
                );
            }
            Ok((Outcome::Defeat, hull)) => {
                messages.extend(settle_with_reward(
                    Outcome::Defeat,
                    hull,
                    commander,
                    ship,
                    rng,
                    None,
                ));
                release_cargo(&jobs, ship);
            }
            Ok((_, hull)) => {
                ship.hull_percentage = hull.min(ship.ship_type.max_hull());
                fail_all(&jobs, commander, ship, &mut messages);
            }
            Err(error) => {
                messages.push(format!("The fight was interrupted: {}", error));
                fail_all(&jobs, commander, ship, &mut messages);
            }
        },
    }
    messages
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::commodities::CommodityType;
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
            vec![run("DISO", 8, 120)],
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
            vec![run("DISO", 8, 100)],
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
            vec![run("DISO", 8, 100)],
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
            vec![run("DISO", 8, 100), run("DISO", 8, 60)],
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
            vec![run("DISO", 8, 100)],
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
            vec![run("DISO", 8, 120)],
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
                vec![run("DISO", 8, 100)],
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
            vec![run("DISO", 8, 100)],
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
}
