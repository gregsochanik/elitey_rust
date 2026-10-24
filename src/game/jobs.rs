use crate::data::systems::{GovernmentType, StarSystem};
use crate::game::aftermath::settle_with_reward;
use crate::game::combat::Outcome;
use crate::game::commander::Standing;
use crate::game::travel::fuel_for_distance;
use crate::game::{Commander, Ship};
use rand::seq::SliceRandom;
use rand::Rng;
use serde::{Deserialize, Serialize};

pub const MAX_ACTIVE_JOBS: usize = 3;
pub const ABANDON_REPUTATION: i32 = -3;
pub const DELIVERY_REPUTATION: i32 = 2;
pub const FAILURE_REPUTATION: i32 = -3;
const MIN_BOARD_SIZE: usize = 3;
const MAX_BOARD_SIZE: usize = 5;
const MAX_JOB_RANGE: f64 = 7.0;
const BASE_REWARD: f64 = 20.0;
const REWARD_PER_FUEL: f64 = 7.0;
const BOUNTY_BASE_REWARD: f64 = 80.0;
const BOUNTY_REWARD_PER_FUEL: f64 = 10.0;
const BOUNTY_SHARE: f64 = 0.3;

/// What a job asks of the player
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobKind {
    /// Carry a weightless parcel to another system
    #[default]
    Courier,
    /// Hunt a pirate raider at another system
    Bounty,
}

/// Who a kind of job is posted for; the lawful, criminal and gateway kinds arrive with their jobs
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Track {
    /// On every board
    Ordinary,
    /// Posted by lawful systems for Trusted and Respected commanders
    Lawful,
    /// Posted by lawless systems for Disreputable and Criminal commanders
    Criminal,
    /// Posted by lawless systems for Neutral commanders starting down the criminal path
    Gateway,
}

/// Which tracks of job a commander is offered at a system
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JobAccess {
    pub lawful: bool,
    pub criminal: bool,
    pub gateway: bool,
}

impl JobAccess {
    pub fn for_standing_at(standing: Standing, government: &GovernmentType) -> Self {
        let lawful_post = government.is_lawful();
        let none = JobAccess {
            lawful: false,
            criminal: false,
            gateway: false,
        };
        match standing {
            Standing::Respected | Standing::Trusted => JobAccess {
                lawful: lawful_post,
                ..none
            },
            Standing::Neutral => JobAccess {
                gateway: !lawful_post,
                ..none
            },
            Standing::Disreputable | Standing::Criminal => JobAccess {
                criminal: !lawful_post,
                ..none
            },
        }
    }

    pub fn allows(&self, track: Track) -> bool {
        match track {
            Track::Ordinary => true,
            Track::Lawful => self.lawful,
            Track::Criminal => self.criminal,
            Track::Gateway => self.gateway,
        }
    }
}

impl JobKind {
    pub fn track(&self) -> Track {
        match self {
            JobKind::Courier | JobKind::Bounty => Track::Ordinary,
        }
    }

    pub fn describe(&self, destination: &str) -> String {
        match self {
            JobKind::Courier => format!("Courier to {}", destination),
            JobKind::Bounty => format!("Bounty: pirate raider near {}", destination),
        }
    }
}

/// A job on the board or in the player's active list, due within a number of jumps
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Job {
    #[serde(default)]
    pub kind: JobKind,
    pub destination: String,
    pub distance: f64,
    pub fuel_cost: u32,
    pub reward: u32,
    pub deadline_jumps: u32,
}

/// A job the player has accepted and is working towards
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActiveJob {
    pub job: Job,
    pub jumps_left: u32,
}

/// Take job `number` (1-based) from the board at `location` into the active list
pub fn accept(
    board: &mut JobBoard,
    active: &mut Vec<ActiveJob>,
    location: &str,
    number: usize,
) -> Result<Job, String> {
    if board.location != location {
        return Err("Check the board first with 'jobs'.".to_string());
    }
    if active.len() >= MAX_ACTIVE_JOBS {
        return Err(format!(
            "You already have {} jobs. Finish or abandon one first.",
            MAX_ACTIVE_JOBS
        ));
    }
    if number == 0 || number > board.jobs.len() {
        return Err(format!("There is no job {} on the board.", number));
    }
    let job = board.jobs.remove(number - 1);
    active.push(ActiveJob {
        jumps_left: job.deadline_jumps,
        job: job.clone(),
    });
    Ok(job)
}

/// Drop active job `number` (1-based), costing reputation
pub fn abandon(
    active: &mut Vec<ActiveJob>,
    commander: &mut Commander,
    number: usize,
) -> Result<Job, String> {
    if number == 0 || number > active.len() {
        return Err(format!("You have no job {}.", number));
    }
    let abandoned = active.remove(number - 1);
    let change = commander.change_reputation(ABANDON_REPUTATION);
    if let Some(standing) = change.new_standing {
        crate::utils::display::show_info(&format!("Your standing is now {}.", standing.name()));
    }
    Ok(abandoned.job)
}

fn reputation_messages(
    messages: &mut Vec<String>,
    change: crate::game::commander::ReputationChange,
) {
    messages.push(format!("Reputation {:+}.", change.applied));
    if let Some(standing) = change.new_standing {
        messages.push(format!("Your standing is now {}.", standing.name()));
    }
}

/// What arriving at a system did to the player's jobs
#[derive(Debug, Default)]
pub struct Arrival {
    pub messages: Vec<String>,
    /// Bounties whose target is here and still in time; each now needs a fight
    pub bounties: Vec<ActiveJob>,
}

/// Use up one deadline jump on every active job after the player arrives at `arrived_at`:
/// courier jobs for this system are delivered and paid, bounties for it are handed back to be
/// fought, and jobs out of jumps fail
pub fn settle_arrival(
    active: &mut Vec<ActiveJob>,
    commander: &mut Commander,
    arrived_at: &str,
) -> Arrival {
    let mut arrival = Arrival::default();
    let mut remaining = Vec::new();

    for mut entry in std::mem::take(active) {
        entry.jumps_left = entry.jumps_left.saturating_sub(1);
        let here = entry.job.destination.eq_ignore_ascii_case(arrived_at);
        if here && entry.job.kind == JobKind::Bounty {
            arrival.bounties.push(entry);
        } else if here {
            commander.add_credits(entry.job.reward);
            arrival.messages.push(format!(
                "Delivered the parcel to {}: +{}cr.",
                entry.job.destination, entry.job.reward
            ));
            reputation_messages(
                &mut arrival.messages,
                commander.change_reputation(DELIVERY_REPUTATION),
            );
        } else if entry.jumps_left == 0 {
            arrival.messages.push(format!(
                "The job to {} ran out of time and was cancelled.",
                entry.job.destination
            ));
            reputation_messages(
                &mut arrival.messages,
                commander.change_reputation(FAILURE_REPUTATION),
            );
        } else {
            remaining.push(entry);
        }
    }

    *active = remaining;
    arrival
}

/// Fight each bounty in turn. `fight` plays one fight and returns how it ended and the
/// player's remaining hull. Anything but a victory ends the hunt and fails the rest.
pub fn resolve_bounties(
    bounties: Vec<ActiveJob>,
    commander: &mut Commander,
    ship: &mut Ship,
    fight: &mut impl FnMut(&Ship) -> std::io::Result<(Outcome, u32)>,
    rng: &mut impl Rng,
) -> Vec<String> {
    let mut messages = Vec::new();
    let mut hunt_over = false;

    for entry in bounties {
        if !hunt_over {
            match fight(ship) {
                Ok((outcome, hull)) => {
                    messages.extend(settle_with_reward(
                        outcome,
                        hull,
                        commander,
                        ship,
                        rng,
                        Some(entry.job.reward),
                    ));
                    if outcome == Outcome::Victory {
                        messages.push(format!(
                            "Bounty claimed near {}: {}cr.",
                            entry.job.destination, entry.job.reward
                        ));
                        continue;
                    }
                }
                Err(error) => messages.push(format!("The fight was interrupted: {}", error)),
            }
            hunt_over = true;
        }
        messages.push(format!("The bounty near {} failed.", entry.job.destination));
        reputation_messages(
            &mut messages,
            commander.change_reputation(FAILURE_REPUTATION),
        );
    }
    messages
}

/// The jobs on offer at the system the player is docked at
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct JobBoard {
    location: String,
    jobs: Vec<Job>,
}

pub fn courier_reward(fuel_cost: u32, variance: f64) -> u32 {
    ((BASE_REWARD + REWARD_PER_FUEL * f64::from(fuel_cost)) * variance).round() as u32
}

pub fn bounty_reward(fuel_cost: u32, variance: f64) -> u32 {
    ((BOUNTY_BASE_REWARD + BOUNTY_REWARD_PER_FUEL * f64::from(fuel_cost)) * variance).round() as u32
}

/// Roll a fresh board of jobs, offering the kinds this standing may see at this system
pub fn generate_board_for(
    origin: &StarSystem,
    systems: &[&StarSystem],
    rng: &mut impl Rng,
    standing: Standing,
) -> Vec<Job> {
    let access = JobAccess::for_standing_at(standing, &origin.government);
    generate_board_with(origin, systems, rng, BOUNTY_SHARE, access)
}

fn generate_board_with(
    origin: &StarSystem,
    systems: &[&StarSystem],
    rng: &mut impl Rng,
    bounty_share: f64,
    access: JobAccess,
) -> Vec<Job> {
    let mut destinations: Vec<&&StarSystem> = systems
        .iter()
        .filter(|system| system.name != origin.name && origin.distance_to(system) <= MAX_JOB_RANGE)
        .collect();
    destinations.sort_by(|a, b| a.name.cmp(&b.name));
    destinations.shuffle(rng);

    let size = rng
        .gen_range(MIN_BOARD_SIZE..=MAX_BOARD_SIZE)
        .min(destinations.len());
    destinations
        .into_iter()
        .take(size)
        .map(|destination| {
            let distance = origin.distance_to(destination);
            let fuel_cost = fuel_for_distance(distance);
            let kind = if rng.gen_bool(bounty_share) && access.allows(JobKind::Bounty.track()) {
                JobKind::Bounty
            } else {
                JobKind::Courier
            };
            let variance = rng.gen_range(0.9..=1.1);
            Job {
                kind,
                destination: destination.name.clone(),
                distance,
                fuel_cost,
                reward: match kind {
                    JobKind::Courier => courier_reward(fuel_cost, variance),
                    JobKind::Bounty => bounty_reward(fuel_cost, variance),
                },
                deadline_jumps: rng.gen_range(2..=4),
            }
        })
        .collect()
}

impl JobBoard {
    pub fn new() -> Self {
        Self::default()
    }

    /// The board for this system, rolled on first sight and kept until the player leaves
    pub fn jobs_at(
        &mut self,
        origin: &StarSystem,
        systems: &[&StarSystem],
        rng: &mut impl Rng,
        standing: Standing,
    ) -> &[Job] {
        if self.location != origin.name {
            self.jobs = generate_board_for(origin, systems, rng, standing);
            self.location = origin.name.clone();
        }
        &self.jobs
    }

    /// The jobs currently on the board
    pub fn jobs(&self) -> &[Job] {
        &self.jobs
    }

    /// Forget the board so the next visit rolls a new one
    pub fn clear(&mut self) {
        self.location.clear();
        self.jobs.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::systems::SystemManager;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    fn galaxy() -> SystemManager {
        SystemManager::new().expect("galaxy data")
    }

    #[test]
    fn reward_follows_the_formula() {
        assert_eq!(courier_reward(3, 1.0), 41);
        assert_eq!(courier_reward(1, 1.0), 27);
        assert_eq!(courier_reward(7, 0.9), 62);
        assert_eq!(courier_reward(7, 1.1), 76);
    }

    #[test]
    fn bounty_reward_follows_the_formula() {
        assert_eq!(bounty_reward(3, 1.0), 110);
        assert_eq!(bounty_reward(7, 0.9), 135);
        assert_eq!(bounty_reward(7, 1.1), 165);
    }

    #[test]
    fn about_thirty_percent_of_jobs_are_bounties_when_enabled() {
        let galaxy = galaxy();
        let systems = galaxy.get_all_systems();
        let origin = galaxy.get_system("LAVE").unwrap();

        let (mut bounties, mut total) = (0, 0);
        for seed in 0..500u64 {
            let board = generate_board_with(
                origin,
                &systems,
                &mut StdRng::seed_from_u64(seed),
                0.3,
                JobAccess::for_standing_at(Standing::Neutral, &origin.government),
            );
            for job in &board {
                total += 1;
                if job.kind == JobKind::Bounty {
                    bounties += 1;
                    let base = 80.0 + 10.0 * f64::from(job.fuel_cost);
                    assert!(f64::from(job.reward) >= (base * 0.9).round());
                    assert!(f64::from(job.reward) <= (base * 1.1).round());
                }
            }
        }
        let share = f64::from(bounties) / f64::from(total);
        assert!((0.25..=0.35).contains(&share), "share {}", share);
    }

    #[test]
    fn jobs_saved_without_a_kind_load_as_courier_jobs() {
        let json =
            r#"{"destination":"DISO","distance":2.3,"fuel_cost":3,"reward":41,"deadline_jumps":3}"#;
        let job: Job = serde_json::from_str(json).unwrap();
        assert_eq!(job.kind, JobKind::Courier);
    }

    #[test]
    fn bounties_here_are_handed_back_to_be_fought_not_paid() {
        let mut commander = Commander::new("Test".to_string());
        let mut bounty = active_job("DISO", 150, 1);
        bounty.job.kind = JobKind::Bounty;
        let mut active = vec![bounty, active_job("DISO", 40, 2)];

        let arrival = settle_arrival(&mut active, &mut commander, "DISO");

        assert_eq!(arrival.bounties.len(), 1);
        assert_eq!(commander.credits(), 140);
        assert!(active.is_empty());
    }

    #[test]
    fn a_bounty_that_runs_out_of_time_elsewhere_fails_without_a_fight() {
        let mut commander = Commander::new("Test".to_string());
        let mut bounty = active_job("DISO", 150, 1);
        bounty.job.kind = JobKind::Bounty;
        let mut active = vec![bounty];

        let arrival = settle_arrival(&mut active, &mut commander, "LEESTI");

        assert!(arrival.bounties.is_empty());
        assert!(active.is_empty());
        assert_eq!(commander.reputation, -3);
    }

    fn bounty(destination: &str, reward: u32) -> ActiveJob {
        let mut entry = active_job(destination, reward, 1);
        entry.job.kind = JobKind::Bounty;
        entry
    }

    fn fights(
        results: Vec<(Outcome, u32)>,
    ) -> impl FnMut(&Ship) -> std::io::Result<(Outcome, u32)> {
        let mut results = results.into_iter();
        move |_| Ok(results.next().expect("unexpected extra fight"))
    }

    #[test]
    fn winning_a_bounty_pays_the_bounty_instead_of_the_random_payout() {
        let mut commander = Commander::new("Test".to_string());
        let mut ship = Ship::new_cobra_mk3();

        let messages = resolve_bounties(
            vec![bounty("DISO", 150)],
            &mut commander,
            &mut ship,
            &mut fights(vec![(Outcome::Victory, 70)]),
            &mut StdRng::seed_from_u64(1),
        );

        assert_eq!(commander.credits(), 250);
        assert_eq!(commander.reputation, 5);
        assert_eq!(ship.hull_percentage, 70);
        assert!(messages.iter().any(|m| m.contains("Bounty claimed")));
    }

    #[test]
    fn escaping_surrendering_or_losing_fails_the_bounty() {
        for outcome in [Outcome::Escaped, Outcome::Surrendered, Outcome::Defeat] {
            let mut commander = Commander::new("Test".to_string());
            let mut ship = Ship::new_cobra_mk3();

            resolve_bounties(
                vec![bounty("DISO", 150)],
                &mut commander,
                &mut ship,
                &mut fights(vec![(outcome, 40)]),
                &mut StdRng::seed_from_u64(1),
            );

            assert!(commander.credits() <= 100, "{:?}", outcome);
            let expected = match outcome {
                Outcome::Defeat => -8,
                _ => -3,
            };
            assert_eq!(commander.reputation, expected, "{:?}", outcome);
        }
    }

    #[test]
    fn escaping_the_first_of_two_bounties_fails_both() {
        let mut commander = Commander::new("Test".to_string());
        let mut ship = Ship::new_cobra_mk3();

        resolve_bounties(
            vec![bounty("DISO", 150), bounty("DISO", 170)],
            &mut commander,
            &mut ship,
            &mut fights(vec![(Outcome::Escaped, 60)]),
            &mut StdRng::seed_from_u64(1),
        );

        assert_eq!(commander.credits(), 100);
        assert_eq!(commander.reputation, -6);
    }

    #[test]
    fn winning_the_first_bounty_leads_to_a_second_fight() {
        let mut commander = Commander::new("Test".to_string());
        let mut ship = Ship::new_cobra_mk3();

        resolve_bounties(
            vec![bounty("DISO", 150), bounty("DISO", 170)],
            &mut commander,
            &mut ship,
            &mut fights(vec![(Outcome::Victory, 80), (Outcome::Victory, 60)]),
            &mut StdRng::seed_from_u64(1),
        );

        assert_eq!(commander.credits(), 420);
        assert_eq!(ship.hull_percentage, 60);
    }

    #[test]
    fn boards_have_three_to_five_reachable_jobs_with_valid_numbers() {
        let galaxy = galaxy();
        let systems = galaxy.get_all_systems();
        let origin = galaxy.get_system("LAVE").unwrap();

        for seed in 0..200u64 {
            let board = generate_board_for(
                origin,
                &systems,
                &mut StdRng::seed_from_u64(seed),
                Standing::Neutral,
            );
            assert!(
                (3..=5).contains(&board.len()),
                "seed {} gave {}",
                seed,
                board.len()
            );

            let mut seen = std::collections::HashSet::new();
            for job in &board {
                assert_ne!(job.destination, "LAVE");
                assert!(
                    seen.insert(job.destination.clone()),
                    "duplicate destination"
                );
                assert!(job.distance <= MAX_JOB_RANGE);
                assert_eq!(job.fuel_cost, fuel_for_distance(job.distance));
                assert!((2..=4).contains(&job.deadline_jumps));
                let base = match job.kind {
                    JobKind::Courier => 20.0 + 7.0 * f64::from(job.fuel_cost),
                    JobKind::Bounty => 80.0 + 10.0 * f64::from(job.fuel_cost),
                };
                assert!(f64::from(job.reward) >= (base * 0.9).round());
                assert!(f64::from(job.reward) <= (base * 1.1).round());
            }
        }
    }

    #[test]
    fn sparse_systems_offer_what_is_in_range() {
        let galaxy = galaxy();
        let systems = galaxy.get_all_systems();

        for origin in &systems {
            let in_range = systems
                .iter()
                .filter(|other| {
                    other.name != origin.name && origin.distance_to(other) <= MAX_JOB_RANGE
                })
                .count();
            let board = generate_board_for(
                origin,
                &systems,
                &mut StdRng::seed_from_u64(5),
                Standing::Neutral,
            );
            assert!(
                board.len() <= in_range.min(MAX_BOARD_SIZE),
                "{}",
                origin.name
            );
            assert!(
                board.len() >= in_range.min(MIN_BOARD_SIZE),
                "{}",
                origin.name
            );
        }
    }

    #[test]
    fn the_same_seed_gives_the_same_board_across_galaxy_instances() {
        let first = galaxy();
        let second = galaxy();
        let origin_a = first.get_system("LAVE").unwrap();
        let origin_b = second.get_system("LAVE").unwrap();

        let a = generate_board_for(
            origin_a,
            &first.get_all_systems(),
            &mut StdRng::seed_from_u64(7),
            Standing::Neutral,
        );
        let b = generate_board_for(
            origin_b,
            &second.get_all_systems(),
            &mut StdRng::seed_from_u64(7),
            Standing::Neutral,
        );
        assert_eq!(a, b);
    }

    #[test]
    fn the_same_seed_gives_the_same_board() {
        let galaxy = galaxy();
        let systems = galaxy.get_all_systems();
        let origin = galaxy.get_system("LAVE").unwrap();

        let a = generate_board_for(
            origin,
            &systems,
            &mut StdRng::seed_from_u64(7),
            Standing::Neutral,
        );
        let b = generate_board_for(
            origin,
            &systems,
            &mut StdRng::seed_from_u64(7),
            Standing::Neutral,
        );
        assert_eq!(a, b);
    }

    fn board_with_jobs() -> (JobBoard, Vec<ActiveJob>) {
        let galaxy = galaxy();
        let systems = galaxy.get_all_systems();
        let lave = galaxy.get_system("LAVE").unwrap();
        let mut board = JobBoard::new();
        board.jobs_at(
            lave,
            &systems,
            &mut StdRng::seed_from_u64(2),
            Standing::Neutral,
        );
        (board, Vec::new())
    }

    #[test]
    fn accepting_moves_a_job_from_the_board_to_the_active_list() {
        let (mut board, mut active) = board_with_jobs();
        let before = board.jobs.len();
        let second = board.jobs[1].clone();

        let job = accept(&mut board, &mut active, "LAVE", 2).unwrap();

        assert_eq!(job, second);
        assert_eq!(board.jobs.len(), before - 1);
        assert!(!board.jobs.contains(&second));
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].jumps_left, second.deadline_jumps);
    }

    #[test]
    fn a_fourth_active_job_is_refused() {
        let (mut board, mut active) = board_with_jobs();
        for _ in 0..3 {
            accept(&mut board, &mut active, "LAVE", 1).unwrap();
        }
        let remaining = board.jobs.len();

        assert!(accept(&mut board, &mut active, "LAVE", 1).is_err());
        assert_eq!(active.len(), 3);
        assert_eq!(board.jobs.len(), remaining);
    }

    fn active_job(destination: &str, reward: u32, jumps_left: u32) -> ActiveJob {
        ActiveJob {
            job: Job {
                kind: JobKind::Courier,
                destination: destination.to_string(),
                distance: 2.3,
                fuel_cost: 3,
                reward,
                deadline_jumps: 3,
            },
            jumps_left,
        }
    }

    #[test]
    fn arriving_in_time_pays_the_reward_and_clears_the_job() {
        let mut commander = Commander::new("Test".to_string());
        let mut active = vec![active_job("DISO", 50, 3)];

        let messages = settle_arrival(&mut active, &mut commander, "DISO").messages;

        assert!(active.is_empty());
        assert_eq!(commander.credits(), 150);
        assert_eq!(commander.reputation, 2);
        assert!(messages.iter().any(|m| m.contains("Delivered")));
    }

    #[test]
    fn delivering_on_the_last_allowed_jump_still_counts() {
        let mut commander = Commander::new("Test".to_string());
        let mut active = vec![active_job("DISO", 50, 1)];

        settle_arrival(&mut active, &mut commander, "diso");

        assert!(active.is_empty());
        assert_eq!(commander.credits(), 150);
    }

    #[test]
    fn a_job_that_runs_out_of_jumps_fails_at_once() {
        let mut commander = Commander::new("Test".to_string());
        let mut active = vec![active_job("DISO", 50, 1)];

        let messages = settle_arrival(&mut active, &mut commander, "LEESTI").messages;

        assert!(active.is_empty());
        assert_eq!(commander.credits(), 100);
        assert_eq!(commander.reputation, -3);
        assert!(messages.iter().any(|m| m.contains("ran out of time")));
    }

    #[test]
    fn jobs_with_time_left_lose_one_jump_per_arrival() {
        let mut commander = Commander::new("Test".to_string());
        let mut active = vec![active_job("DISO", 50, 3)];

        settle_arrival(&mut active, &mut commander, "LEESTI");

        assert_eq!(active[0].jumps_left, 2);
        assert_eq!(commander.reputation, 0);
    }

    #[test]
    fn several_jobs_for_one_destination_all_complete() {
        let mut commander = Commander::new("Test".to_string());
        let mut active = vec![
            active_job("DISO", 50, 3),
            active_job("DISO", 60, 2),
            active_job("LEESTI", 70, 3),
        ];

        settle_arrival(&mut active, &mut commander, "DISO");

        assert_eq!(commander.credits(), 210);
        assert_eq!(commander.reputation, 4);
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].job.destination, "LEESTI");
    }

    #[test]
    fn bad_numbers_and_unseen_boards_are_refused() {
        let (mut board, mut active) = board_with_jobs();
        assert!(accept(&mut board, &mut active, "LAVE", 0).is_err());
        assert!(accept(&mut board, &mut active, "LAVE", 99).is_err());
        assert!(accept(&mut board, &mut active, "DISO", 1).is_err());
        assert!(active.is_empty());
    }

    #[test]
    fn abandoning_removes_the_job_and_costs_reputation() {
        let (mut board, mut active) = board_with_jobs();
        accept(&mut board, &mut active, "LAVE", 1).unwrap();
        accept(&mut board, &mut active, "LAVE", 1).unwrap();
        let kept = active[1].clone();
        let mut commander = Commander::new("Test".to_string());

        abandon(&mut active, &mut commander, 1).unwrap();

        assert_eq!(active, vec![kept]);
        assert_eq!(commander.reputation, -3);
        assert!(abandon(&mut active, &mut commander, 5).is_err());
        assert_eq!(commander.reputation, -3);
    }

    #[test]
    fn the_board_is_stable_while_docked_and_rerolls_after_leaving() {
        let galaxy = galaxy();
        let systems = galaxy.get_all_systems();
        let lave = galaxy.get_system("LAVE").unwrap();
        let mut board = JobBoard::new();
        let mut rng = StdRng::seed_from_u64(1);

        let first = board
            .jobs_at(lave, &systems, &mut rng, Standing::Neutral)
            .to_vec();
        let again = board
            .jobs_at(lave, &systems, &mut rng, Standing::Neutral)
            .to_vec();
        assert_eq!(first, again);

        board.clear();
        let rerolled = board
            .jobs_at(lave, &systems, &mut rng, Standing::Neutral)
            .to_vec();
        assert_ne!(first, rerolled);
    }

    fn governments() -> [GovernmentType; 8] {
        [
            GovernmentType::Anarchy,
            GovernmentType::Feudal,
            GovernmentType::MultiGovernment,
            GovernmentType::Dictatorship,
            GovernmentType::Communist,
            GovernmentType::Confederacy,
            GovernmentType::Democracy,
            GovernmentType::CorporateState,
        ]
    }

    #[test]
    fn only_anarchy_and_feudal_systems_are_lawless() {
        for government in governments() {
            let lawless = matches!(government, GovernmentType::Anarchy | GovernmentType::Feudal);
            assert_eq!(government.is_lawful(), !lawless, "{:?}", government);
        }
    }

    #[test]
    fn lawful_contracts_need_a_good_standing_and_a_lawful_system() {
        for government in governments() {
            for standing in [Standing::Trusted, Standing::Respected] {
                let access = JobAccess::for_standing_at(standing, &government);
                assert_eq!(access.lawful, government.is_lawful(), "{:?}", government);
                assert!(!access.criminal && !access.gateway);
            }
        }
    }

    #[test]
    fn criminal_contracts_need_a_bad_standing_and_a_lawless_system() {
        for government in governments() {
            for standing in [Standing::Disreputable, Standing::Criminal] {
                let access = JobAccess::for_standing_at(standing, &government);
                assert_eq!(access.criminal, !government.is_lawful(), "{:?}", government);
                assert!(!access.lawful && !access.gateway);
            }
        }
    }

    #[test]
    fn neutral_commanders_see_only_the_gateway_and_only_in_lawless_systems() {
        for government in governments() {
            let access = JobAccess::for_standing_at(Standing::Neutral, &government);
            assert_eq!(access.gateway, !government.is_lawful(), "{:?}", government);
            assert!(!access.lawful && !access.criminal);
        }
    }

    #[test]
    fn ordinary_jobs_are_open_to_everyone() {
        for government in governments() {
            for standing in [
                Standing::Criminal,
                Standing::Disreputable,
                Standing::Neutral,
                Standing::Trusted,
                Standing::Respected,
            ] {
                let access = JobAccess::for_standing_at(standing, &government);
                assert!(access.allows(JobKind::Courier.track()));
                assert!(access.allows(JobKind::Bounty.track()));
            }
        }
    }

    #[test]
    fn every_standing_still_gets_a_board_at_every_system() {
        let galaxy = galaxy();
        let systems = galaxy.get_all_systems();
        for origin in &systems {
            for standing in [Standing::Criminal, Standing::Neutral, Standing::Respected] {
                let board =
                    generate_board_for(origin, &systems, &mut StdRng::seed_from_u64(3), standing);
                assert!(!board.is_empty(), "{} for {:?}", origin.name, standing);
            }
        }
    }
}
