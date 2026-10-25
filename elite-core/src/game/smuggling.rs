use crate::data::systems::GovernmentType;
use crate::game::aftermath::{push_promotion_message, settle_with_reward};
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

/// The result of arriving at a lawful system with contraband aboard
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Inspection {
    /// No patrol stopped the ship; the drop was made
    Passed(Vec<String>),
    /// A patrol stopped the ship and the player must choose how to answer
    Stopped {
        bribe_cost: u32,
        can_bribe: bool,
        messages: Vec<String>,
    },
}

/// What answering an inspection led to
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Answer {
    Settled(Vec<String>),
    /// The player ran: fight a patrol vessel, then call `finish_patrol_fight`
    Fight,
}

/// Roll whether the patrol stops a smuggler. At most one inspection covers all of the jobs.
pub fn inspect(
    jobs: &[ActiveJob],
    commander: &mut Commander,
    ship: &mut Ship,
    government: &GovernmentType,
    rng: &mut impl Rng,
) -> Inspection {
    let mut messages = Vec::new();
    if rng.gen::<f64>() >= inspection_chance(government) {
        pay_all(jobs, commander, ship, 0, &mut messages);
        return Inspection::Passed(messages);
    }

    messages.push("A patrol stops you and scans the hold.".to_string());
    let bribe_cost = bribe_cost(jobs);
    Inspection::Stopped {
        bribe_cost,
        can_bribe: commander.credits() >= bribe_cost,
        messages,
    }
}

/// Carry out the player's answer to an inspection. A bribe the player cannot afford is
/// treated as handing the cargo over.
pub fn answer_inspection(
    jobs: &[ActiveJob],
    choice: InspectionChoice,
    commander: &mut Commander,
    ship: &mut Ship,
) -> Answer {
    let cost = bribe_cost(jobs);
    let can_bribe = commander.credits() >= cost;
    let choice = if choice == InspectionChoice::Bribe && !can_bribe {
        InspectionChoice::HandOver
    } else {
        choice
    };

    let mut messages = Vec::new();
    match choice {
        InspectionChoice::Bribe => {
            commander.remove_credits(cost);
            messages.push(format!(
                "You pay {}cr and the patrol waves you through.",
                cost
            ));
            pay_all(jobs, commander, ship, 0, &mut messages);
        }
        InspectionChoice::HandOver => fail_all(jobs, commander, ship, &mut messages),
        InspectionChoice::Run => return Answer::Fight,
    }
    Answer::Settled(messages)
}

/// Settle the jobs after the fight against the patrol vessel
pub fn finish_patrol_fight(
    jobs: &[ActiveJob],
    result: Result<(Outcome, u32), String>,
    commander: &mut Commander,
    ship: &mut Ship,
    rng: &mut impl Rng,
) -> Vec<String> {
    let mut messages = Vec::new();
    match result {
        Ok((Outcome::Victory, hull)) => {
            ship.hull_percentage = hull.min(ship.ship_type.max_hull());
            messages.push("You destroy the patrol vessel and make the drop.".to_string());
            push_promotion_message(&mut messages, commander.record_kill());
            pay_all(
                jobs,
                commander,
                ship,
                ATTACKING_THE_LAW_REPUTATION,
                &mut messages,
            );
        }
        Ok((Outcome::EnemyFled | Outcome::EnemySurrendered, hull)) => {
            ship.hull_percentage = hull.min(ship.ship_type.max_hull());
            messages.push("The patrol vessel backs off and you make the drop.".to_string());
            pay_all(
                jobs,
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
            release_cargo(jobs, ship);
        }
        Ok((_, hull)) => {
            ship.hull_percentage = hull.min(ship.ship_type.max_hull());
            fail_all(jobs, commander, ship, &mut messages);
        }
        Err(error) => {
            messages.push(format!("The fight was interrupted: {}", error));
            fail_all(jobs, commander, ship, &mut messages);
        }
    }
    messages
}

/// Settle smuggling jobs that have reached a lawful system in one go. `choose` is asked what
/// to do when inspected, given the bribe cost and whether the player can afford it; `fight`
/// plays a fight against a patrol vessel.
pub fn resolve_smuggling(
    jobs: Vec<ActiveJob>,
    commander: &mut Commander,
    ship: &mut Ship,
    government: &GovernmentType,
    rng: &mut impl Rng,
    choose: &mut impl FnMut(u32, bool) -> InspectionChoice,
    fight: &mut impl FnMut(&Ship) -> std::io::Result<(Outcome, u32)>,
) -> Vec<String> {
    if jobs.is_empty() {
        return Vec::new();
    }
    let (bribe_cost, can_bribe, mut messages) =
        match inspect(&jobs, commander, ship, government, rng) {
            Inspection::Passed(messages) => return messages,
            Inspection::Stopped {
                bribe_cost,
                can_bribe,
                messages,
            } => (bribe_cost, can_bribe, messages),
        };

    let choice = choose(bribe_cost, can_bribe);
    match answer_inspection(&jobs, choice, commander, ship) {
        Answer::Settled(more) => messages.extend(more),
        Answer::Fight => {
            let result = fight(ship).map_err(|error| error.to_string());
            messages.extend(finish_patrol_fight(&jobs, result, commander, ship, rng));
        }
    }
    messages
}

#[cfg(test)]
mod tests;
