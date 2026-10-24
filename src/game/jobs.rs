use crate::data::systems::StarSystem;
use crate::game::travel::fuel_for_distance;
use rand::seq::SliceRandom;
use rand::Rng;
use serde::{Deserialize, Serialize};

const MIN_BOARD_SIZE: usize = 3;
const MAX_BOARD_SIZE: usize = 5;
const MAX_JOB_RANGE: f64 = 7.0;
const BASE_REWARD: f64 = 25.0;
const REWARD_PER_FUEL: f64 = 8.0;

/// A courier job: carry a weightless parcel to another system within the deadline
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Job {
    pub destination: String,
    pub distance: f64,
    pub fuel_cost: u32,
    pub reward: u32,
    pub deadline_jumps: u32,
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

/// Roll a fresh board of courier jobs from `origin` to systems within range
pub fn generate_board(
    origin: &StarSystem,
    systems: &[&StarSystem],
    rng: &mut impl Rng,
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
            Job {
                destination: destination.name.clone(),
                distance,
                fuel_cost,
                reward: courier_reward(fuel_cost, rng.gen_range(0.9..=1.1)),
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
    ) -> &[Job] {
        if self.location != origin.name {
            self.jobs = generate_board(origin, systems, rng);
            self.location = origin.name.clone();
        }
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
        assert_eq!(courier_reward(3, 1.0), 49);
        assert_eq!(courier_reward(1, 1.0), 33);
        assert_eq!(courier_reward(7, 0.9), 73);
        assert_eq!(courier_reward(7, 1.1), 89);
    }

    #[test]
    fn boards_have_three_to_five_reachable_jobs_with_valid_numbers() {
        let galaxy = galaxy();
        let systems = galaxy.get_all_systems();
        let origin = galaxy.get_system("LAVE").unwrap();

        for seed in 0..200u64 {
            let board = generate_board(origin, &systems, &mut StdRng::seed_from_u64(seed));
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
                let base = 25.0 + 8.0 * f64::from(job.fuel_cost);
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
            let board = generate_board(origin, &systems, &mut StdRng::seed_from_u64(5));
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

        let a = generate_board(
            origin_a,
            &first.get_all_systems(),
            &mut StdRng::seed_from_u64(7),
        );
        let b = generate_board(
            origin_b,
            &second.get_all_systems(),
            &mut StdRng::seed_from_u64(7),
        );
        assert_eq!(a, b);
    }

    #[test]
    fn the_same_seed_gives_the_same_board() {
        let galaxy = galaxy();
        let systems = galaxy.get_all_systems();
        let origin = galaxy.get_system("LAVE").unwrap();

        let a = generate_board(origin, &systems, &mut StdRng::seed_from_u64(7));
        let b = generate_board(origin, &systems, &mut StdRng::seed_from_u64(7));
        assert_eq!(a, b);
    }

    #[test]
    fn the_board_is_stable_while_docked_and_rerolls_after_leaving() {
        let galaxy = galaxy();
        let systems = galaxy.get_all_systems();
        let lave = galaxy.get_system("LAVE").unwrap();
        let mut board = JobBoard::new();
        let mut rng = StdRng::seed_from_u64(1);

        let first = board.jobs_at(lave, &systems, &mut rng).to_vec();
        let again = board.jobs_at(lave, &systems, &mut rng).to_vec();
        assert_eq!(first, again);

        board.clear();
        let rerolled = board.jobs_at(lave, &systems, &mut rng).to_vec();
        assert_ne!(first, rerolled);
    }
}
