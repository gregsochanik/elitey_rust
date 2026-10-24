use crate::data::commodities::CommodityType;
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
const FREIGHT_SHARE: f64 = 0.3;
const MIN_FREIGHT_TONS: u32 = 5;
const FREIGHT_TONS_PER_BONUS: f64 = 30.0;
const SMUGGLING_SHARE: f64 = 0.4;
const GATEWAY_SHARE: f64 = 0.3;
const BLACK_COURIER_REWARD_MULTIPLIER: f64 = 1.5;
pub const BLACK_COURIER_REPUTATION: i32 = -3;
const SMUGGLING_REWARD_MULTIPLIER: f64 = 2.5;
const SMUGGLING_REWARD_PER_TON: f64 = 5.0;
const MIN_SMUGGLING_TONS: u32 = 5;
const MAX_SMUGGLING_TONS: u32 = 10;

/// What a job asks of the player
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobKind {
    /// Carry a weightless parcel to another system
    #[default]
    Courier,
    /// Hunt a pirate raider at another system
    Bounty,
    /// Haul client cargo to another system for a premium that grows with the load
    Freight,
    /// Run contraband narcotics into a lawful system, past a possible inspection
    Smuggling,
    /// A courier job whose parcel is contraband, offered to Neutral commanders in lawless systems
    BlackCourier,
}

/// Who a kind of job is posted for; the lawful, criminal and gateway kinds arrive with their jobs
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
            JobKind::Freight => Track::Lawful,
            JobKind::Smuggling => Track::Criminal,
            JobKind::BlackCourier => Track::Gateway,
        }
    }

    /// Whether the client's cargo takes up room in the player's hold
    pub fn carries_cargo(&self) -> bool {
        matches!(self, JobKind::Freight | JobKind::Smuggling)
    }
}

impl Job {
    pub fn describe(&self) -> String {
        match (self.kind, &self.commodity) {
            (JobKind::Courier, _) => format!("Courier to {}", self.destination),
            (JobKind::BlackCourier, _) => {
                format!("Black courier: contraband parcel to {}", self.destination)
            }
            (JobKind::Bounty, _) => format!("Bounty: pirate raider near {}", self.destination),
            (JobKind::Freight, Some(commodity)) => format!(
                "Freight: {} tons of {} to {}",
                self.tons,
                commodity.name(),
                self.destination
            ),
            (JobKind::Freight, None) => format!("Freight to {}", self.destination),
            (JobKind::Smuggling, _) => format!(
                "Smuggling: {} tons of contraband to {}",
                self.tons, self.destination
            ),
        }
    }
}

/// A job on the board or in the player's active list, due within a number of jumps
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Job {
    #[serde(default)]
    pub kind: JobKind,
    /// Freight only: tons of client cargo to carry
    #[serde(default)]
    pub tons: u32,
    /// Freight only: what the cargo is
    #[serde(default)]
    pub commodity: Option<CommodityType>,
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
    ship: &mut Ship,
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
    let wanted = &board.jobs[number - 1];
    if wanted.kind.carries_cargo() && !ship.cargo_hold.reserve(wanted.tons) {
        return Err(format!(
            "No room: the cargo is {} tons and you have {} free.",
            wanted.tons,
            ship.cargo_available()
        ));
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
    ship: &mut Ship,
    number: usize,
) -> Result<Job, String> {
    if number == 0 || number > active.len() {
        return Err(format!("You have no job {}.", number));
    }
    let abandoned = active.remove(number - 1);
    release_freight(&abandoned.job, ship);
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

/// Give the hold room back when a freight job ends, however it ends
fn release_freight(job: &Job, ship: &mut Ship) {
    if job.kind.carries_cargo() {
        ship.cargo_hold.release(job.tons);
    }
}

/// What arriving at a system did to the player's jobs
#[derive(Debug, Default)]
pub struct Arrival {
    pub messages: Vec<String>,
    /// Bounties whose target is here and still in time; each now needs a fight
    pub bounties: Vec<ActiveJob>,
    /// Smuggling jobs whose cargo has reached a lawful system and may be inspected
    pub smuggling: Vec<ActiveJob>,
}

/// Use up one deadline jump on every active job after the player arrives at `arrived_at`:
/// courier jobs for this system are delivered and paid, bounties for it are handed back to be
/// fought, and jobs out of jumps fail
pub fn settle_arrival(
    active: &mut Vec<ActiveJob>,
    commander: &mut Commander,
    ship: &mut Ship,
    arrived_at: &str,
) -> Arrival {
    let mut arrival = Arrival::default();
    let mut remaining = Vec::new();

    for mut entry in std::mem::take(active) {
        entry.jumps_left = entry.jumps_left.saturating_sub(1);
        let here = entry.job.destination.eq_ignore_ascii_case(arrived_at);
        if here && entry.job.kind == JobKind::Bounty {
            arrival.bounties.push(entry);
        } else if here
            && entry.job.kind == JobKind::Freight
            && ship.cargo_hold.reserved < entry.job.tons
        {
            ship.cargo_hold.release(entry.job.tons);
            arrival.messages.push(format!(
                "The cargo for {} was lost on the way and the contract is void.",
                entry.job.destination
            ));
            reputation_messages(
                &mut arrival.messages,
                commander.change_reputation(FAILURE_REPUTATION),
            );
        } else if here && entry.job.kind == JobKind::Smuggling {
            arrival.smuggling.push(entry);
        } else if here {
            release_freight(&entry.job, ship);
            commander.add_credits(entry.job.reward);
            let what = match entry.job.kind {
                JobKind::Freight => format!("{} tons of freight", entry.job.tons),
                JobKind::BlackCourier => "the contraband parcel".to_string(),
                _ => "the parcel".to_string(),
            };
            let reputation = match entry.job.kind {
                JobKind::BlackCourier => BLACK_COURIER_REPUTATION,
                _ => DELIVERY_REPUTATION,
            };
            arrival.messages.push(format!(
                "Delivered {} to {}: +{}cr.",
                what, entry.job.destination, entry.job.reward
            ));
            reputation_messages(
                &mut arrival.messages,
                commander.change_reputation(reputation),
            );
        } else if entry.jumps_left == 0 {
            release_freight(&entry.job, ship);
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

/// Black courier pay: one and a half times the courier rate for the distance
pub fn black_courier_reward(fuel_cost: u32, variance: f64) -> u32 {
    (f64::from(courier_reward(fuel_cost, variance)) * BLACK_COURIER_REWARD_MULTIPLIER).round()
        as u32
}

/// Courier pay for the same distance, scaled up by the size of the load
pub fn freight_reward(fuel_cost: u32, tons: u32, variance: f64) -> u32 {
    let load_bonus = 1.0 + f64::from(tons) / FREIGHT_TONS_PER_BONUS;
    ((BASE_REWARD + REWARD_PER_FUEL * f64::from(fuel_cost)) * load_bonus * variance).round() as u32
}

/// About three times the courier rate for the distance, plus a premium per ton of contraband
pub fn smuggling_reward(fuel_cost: u32, tons: u32, variance: f64) -> u32 {
    let base = (BASE_REWARD + REWARD_PER_FUEL * f64::from(fuel_cost)) * SMUGGLING_REWARD_MULTIPLIER;
    ((base + SMUGGLING_REWARD_PER_TON * f64::from(tons)) * variance).round() as u32
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
    hold: u32,
) -> Vec<Job> {
    let access = JobAccess::for_standing_at(standing, &origin.government);
    generate_board_with(origin, systems, rng, BOUNTY_SHARE, access, hold)
}

fn generate_board_with(
    origin: &StarSystem,
    systems: &[&StarSystem],
    rng: &mut impl Rng,
    bounty_share: f64,
    access: JobAccess,
    hold: u32,
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
            let roll = rng.gen::<f64>();
            let kind = if roll < bounty_share && access.allows(JobKind::Bounty.track()) {
                JobKind::Bounty
            } else if roll >= 1.0 - FREIGHT_SHARE && access.allows(JobKind::Freight.track()) {
                JobKind::Freight
            } else if roll >= bounty_share
                && roll < bounty_share + SMUGGLING_SHARE
                && access.allows(JobKind::Smuggling.track())
                && destination.government.is_lawful()
            {
                JobKind::Smuggling
            } else if roll >= bounty_share
                && roll < bounty_share + GATEWAY_SHARE
                && access.allows(JobKind::BlackCourier.track())
            {
                JobKind::BlackCourier
            } else {
                JobKind::Courier
            };
            let variance = rng.gen_range(0.9..=1.1);
            let (tons, commodity) = if kind == JobKind::Smuggling {
                (
                    rng.gen_range(MIN_SMUGGLING_TONS..=MAX_SMUGGLING_TONS),
                    Some(CommodityType::Narcotics),
                )
            } else if kind == JobKind::Freight {
                let legal: Vec<CommodityType> = CommodityType::all()
                    .into_iter()
                    .filter(|commodity| !commodity.is_illegal())
                    .collect();
                (
                    rng.gen_range(MIN_FREIGHT_TONS..=hold.max(MIN_FREIGHT_TONS)),
                    legal.choose(rng).cloned(),
                )
            } else {
                (0, None)
            };
            Job {
                kind,
                tons,
                commodity,
                destination: destination.name.clone(),
                distance,
                fuel_cost,
                reward: match kind {
                    JobKind::Courier => courier_reward(fuel_cost, variance),
                    JobKind::BlackCourier => black_courier_reward(fuel_cost, variance),
                    JobKind::Bounty => bounty_reward(fuel_cost, variance),
                    JobKind::Freight => freight_reward(fuel_cost, tons, variance),
                    JobKind::Smuggling => smuggling_reward(fuel_cost, tons, variance),
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
        hold: u32,
    ) -> &[Job] {
        if self.location != origin.name {
            self.jobs = generate_board_for(origin, systems, rng, standing, hold);
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
mod tests;
