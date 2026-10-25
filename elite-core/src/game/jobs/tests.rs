use super::*;
use crate::data::systems::SystemManager;
use rand::rngs::StdRng;
use rand::SeedableRng;

const HOLD: u32 = 20;

fn board_for(
    origin: &StarSystem,
    systems: &[&StarSystem],
    rng: &mut impl Rng,
    standing: Standing,
) -> Vec<Job> {
    board_for_hold(origin, systems, rng, standing, HOLD)
}

fn board_for_hold(
    origin: &StarSystem,
    systems: &[&StarSystem],
    rng: &mut impl Rng,
    standing: Standing,
    hold: u32,
) -> Vec<Job> {
    generate_board_for(origin, systems, rng, hold)
        .into_iter()
        .filter(|job| job.kind.track().open_to(standing))
        .collect()
}

fn settle(active: &mut Vec<ActiveJob>, commander: &mut Commander, arrived_at: &str) -> Arrival {
    settle_arrival(active, commander, &mut Ship::new_cobra_mk3(), arrived_at)
}

fn accept_on_a_fresh_ship(
    board: &mut JobBoard,
    active: &mut Vec<ActiveJob>,
    location: &str,
    number: usize,
) -> Result<Job, String> {
    accept_as(
        board,
        active,
        location,
        Standing::Trusted,
        CombatRating::Elite,
        number,
    )
}

fn accept_as(
    board: &mut JobBoard,
    active: &mut Vec<ActiveJob>,
    location: &str,
    standing: Standing,
    rating: CombatRating,
    number: usize,
) -> Result<Job, String> {
    accept(
        board,
        active,
        &mut Ship::new_cobra_mk3(),
        location,
        standing,
        rating,
        number,
    )
}

fn abandon_on_a_fresh_ship(
    active: &mut Vec<ActiveJob>,
    commander: &mut Commander,
    number: usize,
) -> Result<Job, String> {
    abandon(active, commander, &mut Ship::new_cobra_mk3(), number)
}

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
    let origin = galaxy.get_system("Sol").unwrap();

    let (mut bounties, mut total) = (0, 0);
    for seed in 0..500u64 {
        let board = generate_board_with(
            origin,
            &systems,
            &mut StdRng::seed_from_u64(seed),
            0.3,
            HOLD,
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
    let json = r#"{"destination":"Proxima Centauri","distance":2.3,"fuel_cost":3,"reward":41,"deadline_jumps":3}"#;
    let job: Job = serde_json::from_str(json).unwrap();
    assert_eq!(job.kind, JobKind::Courier);
}

#[test]
fn bounties_here_are_handed_back_to_be_fought_not_paid() {
    let mut commander = Commander::new("Test".to_string());
    let mut bounty = active_job("Proxima Centauri", 150, 1);
    bounty.job.kind = JobKind::Bounty;
    let mut active = vec![bounty, active_job("Proxima Centauri", 40, 2)];

    let arrival = settle(&mut active, &mut commander, "Proxima Centauri");

    assert_eq!(arrival.bounties.len(), 1);
    assert_eq!(commander.credits(), 140);
    assert!(active.is_empty());
}

#[test]
fn a_bounty_that_runs_out_of_time_elsewhere_fails_without_a_fight() {
    let mut commander = Commander::new("Test".to_string());
    let mut bounty = active_job("Proxima Centauri", 150, 1);
    bounty.job.kind = JobKind::Bounty;
    let mut active = vec![bounty];

    let arrival = settle(&mut active, &mut commander, "Alpha Centauri");

    assert!(arrival.bounties.is_empty());
    assert!(active.is_empty());
    assert_eq!(commander.reputation, -3);
}

fn bounty(destination: &str, reward: u32) -> ActiveJob {
    let mut entry = active_job(destination, reward, 1);
    entry.job.kind = JobKind::Bounty;
    entry
}

fn fights(results: Vec<(Outcome, u32)>) -> impl FnMut(&Ship) -> std::io::Result<(Outcome, u32)> {
    let mut results = results.into_iter();
    move |_| Ok(results.next().expect("unexpected extra fight"))
}

#[test]
fn winning_a_bounty_pays_the_bounty_instead_of_the_random_payout() {
    let mut commander = Commander::new("Test".to_string());
    let mut ship = Ship::new_cobra_mk3();

    let messages = resolve_bounties(
        vec![bounty("Proxima Centauri", 150)],
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
            vec![bounty("Proxima Centauri", 150)],
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
        vec![
            bounty("Proxima Centauri", 150),
            bounty("Proxima Centauri", 170),
        ],
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
        vec![
            bounty("Proxima Centauri", 150),
            bounty("Proxima Centauri", 170),
        ],
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
    let origin = galaxy.get_system("Sol").unwrap();

    for seed in 0..200u64 {
        let board = generate_board_for(origin, &systems, &mut StdRng::seed_from_u64(seed), HOLD);
        assert!(
            (3..=5).contains(&board.len()),
            "seed {} gave {}",
            seed,
            board.len()
        );

        let mut seen = std::collections::HashSet::new();
        for job in &board {
            assert_ne!(job.destination, "Sol");
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
                JobKind::Freight => {
                    (20.0 + 7.0 * f64::from(job.fuel_cost)) * (1.0 + f64::from(job.tons) / 30.0)
                }
                JobKind::Smuggling => {
                    2.5 * (20.0 + 7.0 * f64::from(job.fuel_cost)) + 5.0 * f64::from(job.tons)
                }
                JobKind::BlackCourier => 1.5 * (20.0 + 7.0 * f64::from(job.fuel_cost)),
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
            .filter(|other| other.name != origin.name && origin.distance_to(other) <= MAX_JOB_RANGE)
            .count();
        let board = generate_board_for(origin, &systems, &mut StdRng::seed_from_u64(5), HOLD);
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
    let origin_a = first.get_system("Sol").unwrap();
    let origin_b = second.get_system("Sol").unwrap();

    let a = board_for(
        origin_a,
        &first.get_all_systems(),
        &mut StdRng::seed_from_u64(7),
        Standing::Neutral,
    );
    let b = board_for(
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
    let origin = galaxy.get_system("Sol").unwrap();

    let a = board_for(
        origin,
        &systems,
        &mut StdRng::seed_from_u64(7),
        Standing::Neutral,
    );
    let b = board_for(
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
    let sol = galaxy.get_system("Sol").unwrap();
    let mut board = JobBoard::new();
    board.jobs_at(sol, &systems, &mut StdRng::seed_from_u64(2), HOLD);
    (board, Vec::new())
}

#[test]
fn accepting_moves_a_job_from_the_board_to_the_active_list() {
    let (mut board, mut active) = board_with_jobs();
    let before = board.jobs.len();
    let second = board.jobs[1].clone();

    let job = accept_on_a_fresh_ship(&mut board, &mut active, "Sol", 2).unwrap();

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
        accept_on_a_fresh_ship(&mut board, &mut active, "Sol", 1).unwrap();
    }
    let remaining = board.jobs.len();

    assert!(accept_on_a_fresh_ship(&mut board, &mut active, "Sol", 1).is_err());
    assert_eq!(active.len(), 3);
    assert_eq!(board.jobs.len(), remaining);
}

fn active_job(destination: &str, reward: u32, jumps_left: u32) -> ActiveJob {
    ActiveJob {
        job: Job {
            kind: JobKind::Courier,
            tons: 0,
            commodity: None,
            destination: destination.to_string(),
            distance: 2.3,
            fuel_cost: 3,
            reward,
            deadline_jumps: 3,
            min_rating: CombatRating::Harmless,
        },
        jumps_left,
    }
}

#[test]
fn arriving_in_time_pays_the_reward_and_clears_the_job() {
    let mut commander = Commander::new("Test".to_string());
    let mut active = vec![active_job("Proxima Centauri", 50, 3)];

    let messages = settle(&mut active, &mut commander, "Proxima Centauri").messages;

    assert!(active.is_empty());
    assert_eq!(commander.credits(), 150);
    assert_eq!(commander.reputation, 2);
    assert!(messages.iter().any(|m| m.contains("Delivered")));
}

#[test]
fn delivering_on_the_last_allowed_jump_still_counts() {
    let mut commander = Commander::new("Test".to_string());
    let mut active = vec![active_job("Proxima Centauri", 50, 1)];

    settle(&mut active, &mut commander, "proxima centauri");

    assert!(active.is_empty());
    assert_eq!(commander.credits(), 150);
}

#[test]
fn a_job_that_runs_out_of_jumps_fails_at_once() {
    let mut commander = Commander::new("Test".to_string());
    let mut active = vec![active_job("Proxima Centauri", 50, 1)];

    let messages = settle(&mut active, &mut commander, "Alpha Centauri").messages;

    assert!(active.is_empty());
    assert_eq!(commander.credits(), 100);
    assert_eq!(commander.reputation, -3);
    assert!(messages.iter().any(|m| m.contains("ran out of time")));
}

#[test]
fn jobs_with_time_left_lose_one_jump_per_arrival() {
    let mut commander = Commander::new("Test".to_string());
    let mut active = vec![active_job("Proxima Centauri", 50, 3)];

    settle(&mut active, &mut commander, "Alpha Centauri");

    assert_eq!(active[0].jumps_left, 2);
    assert_eq!(commander.reputation, 0);
}

#[test]
fn several_jobs_for_one_destination_all_complete() {
    let mut commander = Commander::new("Test".to_string());
    let mut active = vec![
        active_job("Proxima Centauri", 50, 3),
        active_job("Proxima Centauri", 60, 2),
        active_job("Alpha Centauri", 70, 3),
    ];

    settle(&mut active, &mut commander, "Proxima Centauri");

    assert_eq!(commander.credits(), 210);
    assert_eq!(commander.reputation, 4);
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].job.destination, "Alpha Centauri");
}

#[test]
fn bad_numbers_and_unseen_boards_are_refused() {
    let (mut board, mut active) = board_with_jobs();
    assert!(accept_on_a_fresh_ship(&mut board, &mut active, "Sol", 0).is_err());
    assert!(accept_on_a_fresh_ship(&mut board, &mut active, "Sol", 99).is_err());
    assert!(accept_on_a_fresh_ship(&mut board, &mut active, "Proxima Centauri", 1).is_err());
    assert!(active.is_empty());
}

#[test]
fn abandoning_removes_the_job_and_costs_reputation() {
    let (mut board, mut active) = board_with_jobs();
    accept_on_a_fresh_ship(&mut board, &mut active, "Sol", 1).unwrap();
    accept_on_a_fresh_ship(&mut board, &mut active, "Sol", 1).unwrap();
    let kept = active[1].clone();
    let mut commander = Commander::new("Test".to_string());

    abandon_on_a_fresh_ship(&mut active, &mut commander, 1).unwrap();

    assert_eq!(active, vec![kept]);
    assert_eq!(commander.reputation, -3);
    assert!(abandon_on_a_fresh_ship(&mut active, &mut commander, 5).is_err());
    assert_eq!(commander.reputation, -3);
}

#[test]
fn the_board_is_stable_while_docked_and_rerolls_after_leaving() {
    let galaxy = galaxy();
    let systems = galaxy.get_all_systems();
    let sol = galaxy.get_system("Sol").unwrap();
    let mut board = JobBoard::new();
    let mut rng = StdRng::seed_from_u64(1);

    let first = board.jobs_at(sol, &systems, &mut rng, HOLD).to_vec();
    let again = board.jobs_at(sol, &systems, &mut rng, HOLD).to_vec();
    assert_eq!(first, again);

    board.clear();
    let rerolled = board.jobs_at(sol, &systems, &mut rng, HOLD).to_vec();
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
fn lawful_contracts_are_posted_by_lawful_systems_and_open_to_good_standing() {
    for government in governments() {
        assert_eq!(Track::Lawful.posted_by(&government), government.is_lawful());
    }
    for (standing, open) in all_standings(&[Standing::Trusted, Standing::Respected]) {
        assert_eq!(Track::Lawful.open_to(standing), open, "{:?}", standing);
    }
}

#[test]
fn criminal_contracts_are_posted_by_lawless_systems_and_open_to_bad_standing() {
    for government in governments() {
        assert_eq!(
            Track::Criminal.posted_by(&government),
            !government.is_lawful()
        );
    }
    for (standing, open) in all_standings(&[Standing::Disreputable, Standing::Criminal]) {
        assert_eq!(Track::Criminal.open_to(standing), open, "{:?}", standing);
    }
}

#[test]
fn the_gateway_is_posted_by_lawless_systems_and_open_to_neutral_standing() {
    for government in governments() {
        assert_eq!(
            Track::Gateway.posted_by(&government),
            !government.is_lawful()
        );
    }
    for (standing, open) in all_standings(&[Standing::Neutral]) {
        assert_eq!(Track::Gateway.open_to(standing), open, "{:?}", standing);
    }
}

#[test]
fn ordinary_jobs_are_open_to_everyone() {
    for government in governments() {
        assert!(Track::Ordinary.posted_by(&government));
    }
    for (standing, _) in all_standings(&[]) {
        assert!(Track::Ordinary.open_to(standing));
        assert!(Track::Ordinary.open_to(standing));
    }
}

fn all_standings(open: &[Standing]) -> Vec<(Standing, bool)> {
    [
        Standing::Criminal,
        Standing::Disreputable,
        Standing::Neutral,
        Standing::Trusted,
        Standing::Respected,
    ]
    .into_iter()
    .map(|standing| (standing, open.contains(&standing)))
    .collect()
}

fn job_of(kind: JobKind, fuel_cost: u32) -> Job {
    Job {
        kind,
        tons: 0,
        commodity: None,
        destination: "Proxima Centauri".to_string(),
        distance: f64::from(fuel_cost),
        fuel_cost,
        reward: 100,
        deadline_jumps: 3,
        min_rating: if kind == JobKind::Bounty {
            bounty_min_rating(fuel_cost)
        } else {
            CombatRating::Harmless
        },
    }
}

#[test]
fn a_job_is_locked_for_standing_before_rating() {
    let freight = job_of(JobKind::Freight, 3);
    assert_eq!(
        freight.lock(Standing::Neutral, CombatRating::Harmless),
        Some(JobLock::Standing(Track::Lawful))
    );
    assert_eq!(
        freight.lock(Standing::Trusted, CombatRating::Harmless),
        None
    );

    let black = job_of(JobKind::BlackCourier, 3);
    assert_eq!(
        black.lock(Standing::Trusted, CombatRating::Elite),
        Some(JobLock::Standing(Track::Gateway))
    );
    assert_eq!(black.lock(Standing::Neutral, CombatRating::Harmless), None);

    let smuggling = job_of(JobKind::Smuggling, 3);
    assert!(smuggling
        .lock(Standing::Neutral, CombatRating::Elite)
        .is_some());
    assert_eq!(
        smuggling.lock(Standing::Criminal, CombatRating::Harmless),
        None
    );
}

#[test]
fn long_bounties_need_a_combat_rating() {
    let short = job_of(JobKind::Bounty, 3);
    let middling = job_of(JobKind::Bounty, 5);
    let long = job_of(JobKind::Bounty, 7);

    assert_eq!(short.lock(Standing::Neutral, CombatRating::Harmless), None);
    assert_eq!(
        middling.lock(Standing::Neutral, CombatRating::Harmless),
        Some(JobLock::Rating(CombatRating::MostlyHarmless))
    );
    assert_eq!(
        middling.lock(Standing::Neutral, CombatRating::MostlyHarmless),
        None
    );
    assert_eq!(
        long.lock(Standing::Neutral, CombatRating::MostlyHarmless),
        Some(JobLock::Rating(CombatRating::Poor))
    );
    assert_eq!(long.lock(Standing::Neutral, CombatRating::Poor), None);
}

#[test]
fn couriers_are_never_locked() {
    let courier = job_of(JobKind::Courier, 7);
    for (standing, _) in all_standings(&[]) {
        assert_eq!(courier.lock(standing, CombatRating::Harmless), None);
    }
}

#[test]
fn locks_say_what_is_needed() {
    assert_eq!(
        JobLock::Standing(Track::Lawful).describe(),
        "needs standing Trusted or better"
    );
    assert_eq!(
        JobLock::Rating(CombatRating::Poor).describe(),
        "needs combat rating Poor"
    );
}

#[test]
fn accepting_a_locked_job_is_refused_and_changes_nothing() {
    let (mut board, mut active) = board_with_jobs();
    board.jobs[0] = job_of(JobKind::Freight, 3);
    let location = board.location.clone();

    let refused = accept_as(
        &mut board,
        &mut active,
        &location,
        Standing::Neutral,
        CombatRating::Harmless,
        1,
    );

    assert_eq!(
        refused,
        Err("Locked: needs standing Trusted or better.".to_string())
    );
    assert!(active.is_empty());
    let accepted = accept_as(
        &mut board,
        &mut active,
        &location,
        Standing::Trusted,
        CombatRating::Harmless,
        1,
    );
    assert!(accepted.is_ok());
}

#[test]
fn every_system_posts_a_board() {
    let galaxy = galaxy();
    let systems = galaxy.get_all_systems();
    for origin in &systems {
        let board = generate_board_for(origin, &systems, &mut StdRng::seed_from_u64(3), HOLD);
        assert!(!board.is_empty(), "{}", origin.name);
    }
}

fn lawful_origin(galaxy: &SystemManager) -> &StarSystem {
    galaxy
        .get_all_systems()
        .into_iter()
        .find(|system| system.government.is_lawful())
        .expect("a lawful system")
}

fn freight(destination: &str, tons: u32, reward: u32, jumps_left: u32) -> ActiveJob {
    ActiveJob {
        job: Job {
            kind: JobKind::Freight,
            tons,
            commodity: Some(CommodityType::Machinery),
            destination: destination.to_string(),
            distance: 2.3,
            fuel_cost: 3,
            reward,
            deadline_jumps: 3,
            min_rating: CombatRating::Harmless,
        },
        jumps_left,
    }
}

#[test]
fn freight_pay_grows_with_the_load() {
    assert_eq!(freight_reward(3, 0, 1.0), 41);
    assert_eq!(freight_reward(3, 20, 1.0), 68);
    assert_eq!(freight_reward(3, 60, 1.0), 123);
    assert_eq!(freight_reward(3, 10, 1.1), 60);
}

#[test]
fn freight_is_only_offered_to_good_standing_at_lawful_systems() {
    let galaxy = galaxy();
    let systems = galaxy.get_all_systems();
    let origin = lawful_origin(&galaxy);

    let freight_seen = |standing| {
        (0..100u64).any(|seed| {
            board_for_hold(
                origin,
                &systems,
                &mut StdRng::seed_from_u64(seed),
                standing,
                20,
            )
            .iter()
            .any(|job| job.kind == JobKind::Freight)
        })
    };
    assert!(freight_seen(Standing::Trusted));
    assert!(freight_seen(Standing::Respected));
    assert!(!freight_seen(Standing::Neutral));
    assert!(!freight_seen(Standing::Disreputable));
    assert!(!freight_seen(Standing::Criminal));
}

#[test]
fn freight_loads_are_legal_and_fit_the_hold() {
    let galaxy = galaxy();
    let systems = galaxy.get_all_systems();
    let origin = lawful_origin(&galaxy);

    for (hold, seeds) in [(20u32, 200u64), (60, 200)] {
        for seed in 0..seeds {
            for job in board_for_hold(
                origin,
                &systems,
                &mut StdRng::seed_from_u64(seed),
                Standing::Trusted,
                hold,
            )
            .iter()
            .filter(|job| job.kind == JobKind::Freight)
            {
                assert!(
                    (MIN_FREIGHT_TONS..=hold).contains(&job.tons),
                    "{}",
                    job.tons
                );
                assert!(!job.commodity.as_ref().unwrap().is_illegal());
            }
        }
    }
}

fn board_with_freight(tons: u32) -> (JobBoard, Vec<ActiveJob>, String) {
    let (mut board, active) = board_with_jobs();
    board.jobs[0] = freight("Proxima Centauri", tons, 100, 3).job;
    let location = board.location.clone();
    (board, active, location)
}

#[test]
fn accepting_freight_reserves_hold_room_and_a_full_hold_refuses_it() {
    let (mut board, mut active, location) = board_with_freight(15);
    let mut ship = Ship::new_cobra_mk3();

    accept(
        &mut board,
        &mut active,
        &mut ship,
        &location,
        Standing::Respected,
        CombatRating::Elite,
        1,
    )
    .unwrap();
    assert_eq!(ship.cargo_hold.reserved, 15);
    assert_eq!(ship.cargo_available(), 5);

    let (mut board, mut active, location) = board_with_freight(15);
    let mut ship = Ship::new_cobra_mk3();
    ship.add_cargo(CommodityType::Food, 10);
    let before = board.jobs.len();
    assert!(accept(
        &mut board,
        &mut active,
        &mut ship,
        &location,
        Standing::Respected,
        CombatRating::Elite,
        1
    )
    .is_err());
    assert!(active.is_empty());
    assert_eq!(board.jobs.len(), before);
    assert_eq!(ship.cargo_hold.reserved, 0);
}

#[test]
fn delivering_freight_pays_and_frees_the_hold() {
    let mut commander = Commander::new("Test".to_string());
    let mut ship = Ship::new_cobra_mk3();
    assert!(ship.cargo_hold.reserve(12));
    let mut active = vec![freight("Proxima Centauri", 12, 90, 2)];

    let arrival = settle_arrival(&mut active, &mut commander, &mut ship, "Proxima Centauri");

    assert!(active.is_empty());
    assert_eq!(commander.credits(), 190);
    assert_eq!(commander.reputation, 2);
    assert_eq!(ship.cargo_hold.reserved, 0);
    assert!(arrival
        .messages
        .iter()
        .any(|m| m.contains("12 tons of freight")));
}

#[test]
fn freight_that_runs_out_of_time_or_is_abandoned_frees_the_hold_and_costs_reputation() {
    let mut commander = Commander::new("Test".to_string());
    let mut ship = Ship::new_cobra_mk3();
    assert!(ship.cargo_hold.reserve(12));
    let mut active = vec![freight("Proxima Centauri", 12, 90, 1)];
    settle_arrival(&mut active, &mut commander, &mut ship, "Alpha Centauri");
    assert!(active.is_empty());
    assert_eq!(ship.cargo_hold.reserved, 0);
    assert_eq!(commander.reputation, -3);

    let mut commander = Commander::new("Test".to_string());
    let mut ship = Ship::new_cobra_mk3();
    assert!(ship.cargo_hold.reserve(12));
    let mut active = vec![freight("Proxima Centauri", 12, 90, 3)];
    abandon(&mut active, &mut commander, &mut ship, 1).unwrap();
    assert_eq!(ship.cargo_hold.reserved, 0);
    assert_eq!(commander.reputation, -3);
}

#[test]
fn freight_lost_on_the_way_voids_the_contract() {
    let mut commander = Commander::new("Test".to_string());
    let mut ship = Ship::new_cobra_mk3();
    assert!(ship.cargo_hold.reserve(12));
    let mut active = vec![freight("Proxima Centauri", 12, 90, 2)];

    crate::game::events::apply_cargo_loss(&mut ship, &crate::game::events::CargoLoss::Tons(20));
    assert_eq!(ship.cargo_hold.reserved, 0);

    let arrival = settle_arrival(&mut active, &mut commander, &mut ship, "Proxima Centauri");
    assert!(active.is_empty());
    assert_eq!(commander.credits(), 100);
    assert_eq!(commander.reputation, -3);
    assert!(arrival
        .messages
        .iter()
        .any(|m| m.contains("lost on the way")));
}

#[test]
fn cargo_loss_takes_goods_first_then_contract_cargo() {
    let mut ship = Ship::new_cobra_mk3();
    ship.add_cargo(CommodityType::Food, 4);
    assert!(ship.cargo_hold.reserve(10));

    crate::game::events::apply_cargo_loss(&mut ship, &crate::game::events::CargoLoss::Tons(7));

    assert_eq!(ship.cargo_hold.goods_weight(), 0);
    assert_eq!(ship.cargo_hold.reserved, 7);
}

#[test]
fn jobs_saved_without_freight_fields_load_with_none() {
    let json = r#"{"kind":"Courier","destination":"Proxima Centauri","distance":2.3,"fuel_cost":3,"reward":41,"deadline_jumps":3}"#;
    let job: Job = serde_json::from_str(json).unwrap();
    assert_eq!(job.tons, 0);
    assert_eq!(job.commodity, None);
}

#[test]
fn jobs_describe_themselves_by_kind() {
    let entry = freight("Proxima Centauri", 12, 90, 2);
    assert_eq!(
        entry.job.describe(),
        "Freight: 12 tons of Machinery to Proxima Centauri"
    );
    assert_eq!(
        active_job("Proxima Centauri", 41, 2).job.describe(),
        "Courier to Proxima Centauri"
    );
}

fn lawless_origin(galaxy: &SystemManager) -> &StarSystem {
    galaxy
        .get_all_systems()
        .into_iter()
        .find(|system| !system.government.is_lawful())
        .expect("a lawless system")
}

#[test]
fn smuggling_pays_two_and_a_half_times_a_courier_job_plus_a_premium_per_ton() {
    assert_eq!(smuggling_reward(3, 0, 1.0), 103);
    assert_eq!(smuggling_reward(3, 10, 1.0), 153);
    assert_eq!(smuggling_reward(3, 5, 1.1), 140);
}

#[test]
fn smuggling_is_only_offered_to_bad_standing_at_lawless_systems_with_lawful_destinations() {
    let galaxy = galaxy();
    let systems = galaxy.get_all_systems();
    let lawless = lawless_origin(&galaxy);
    let lawful = lawful_origin(&galaxy);

    let boards = |origin: &StarSystem, standing| {
        (0..200u64)
            .flat_map(|seed| {
                board_for_hold(
                    origin,
                    &systems,
                    &mut StdRng::seed_from_u64(seed),
                    standing,
                    20,
                )
            })
            .collect::<Vec<_>>()
    };
    let galaxy_ref = &galaxy;
    let smuggling: Vec<Job> = boards(lawless, Standing::Criminal)
        .into_iter()
        .filter(|job| job.kind == JobKind::Smuggling)
        .collect();
    assert!(!smuggling.is_empty());
    for job in &smuggling {
        assert!((MIN_SMUGGLING_TONS..=MAX_SMUGGLING_TONS).contains(&job.tons));
        assert_eq!(job.commodity, Some(CommodityType::Narcotics));
        let destination = galaxy_ref.get_system(&job.destination).unwrap();
        assert!(destination.government.is_lawful(), "{}", job.destination);
    }
    assert!(boards(lawless, Standing::Disreputable)
        .iter()
        .any(|job| job.kind == JobKind::Smuggling));

    for standing in [Standing::Neutral, Standing::Trusted, Standing::Respected] {
        assert!(boards(lawless, standing)
            .iter()
            .all(|job| job.kind != JobKind::Smuggling));
    }
    for standing in [Standing::Criminal, Standing::Disreputable] {
        assert!(boards(lawful, standing)
            .iter()
            .all(|job| job.kind != JobKind::Smuggling));
    }
}

fn smuggling_job(tons: u32) -> ActiveJob {
    let mut entry = freight("Proxima Centauri", tons, 150, 3);
    entry.job.kind = JobKind::Smuggling;
    entry.job.commodity = Some(CommodityType::Narcotics);
    entry
}

#[test]
fn smuggling_needs_hold_room_to_accept_and_frees_it_when_it_ends() {
    let (mut board, mut active) = board_with_jobs();
    board.jobs[0] = smuggling_job(8).job;
    let location = board.location.clone();
    let mut ship = Ship::new_cobra_mk3();
    ship.add_cargo(CommodityType::Food, 15);
    assert!(accept(
        &mut board,
        &mut active,
        &mut ship,
        &location,
        Standing::Criminal,
        CombatRating::Elite,
        1
    )
    .is_err());
    assert_eq!(ship.cargo_hold.reserved, 0);

    let mut ship = Ship::new_cobra_mk3();
    accept(
        &mut board,
        &mut active,
        &mut ship,
        &location,
        Standing::Criminal,
        CombatRating::Elite,
        1,
    )
    .unwrap();
    assert_eq!(ship.cargo_hold.reserved, 8);

    let mut commander = Commander::new("Test".to_string());
    abandon(&mut active, &mut commander, &mut ship, 1).unwrap();
    assert_eq!(ship.cargo_hold.reserved, 0);

    assert!(ship.cargo_hold.reserve(8));
    let mut active = vec![smuggling_job(8)];
    active[0].jumps_left = 1;
    settle_arrival(&mut active, &mut commander, &mut ship, "Alpha Centauri");
    assert!(active.is_empty());
    assert_eq!(ship.cargo_hold.reserved, 0);
}

#[test]
fn smuggling_at_its_destination_is_handed_back_for_inspection_not_paid() {
    let mut commander = Commander::new("Test".to_string());
    let mut ship = Ship::new_cobra_mk3();
    assert!(ship.cargo_hold.reserve(8));
    let mut active = vec![smuggling_job(8)];

    let arrival = settle_arrival(&mut active, &mut commander, &mut ship, "Proxima Centauri");

    assert_eq!(arrival.smuggling.len(), 1);
    assert!(active.is_empty());
    assert_eq!(commander.credits(), 100);
    assert_eq!(ship.cargo_hold.reserved, 8);
}

#[test]
fn smuggling_describes_itself() {
    assert_eq!(
        smuggling_job(8).job.describe(),
        "Smuggling: 8 tons of contraband to Proxima Centauri"
    );
}

#[test]
fn black_courier_pays_one_and_a_half_times_a_courier_job() {
    assert_eq!(black_courier_reward(3, 1.0), 62);
    assert_eq!(black_courier_reward(7, 1.0), 104);
}

#[test]
fn the_gateway_job_is_offered_only_to_neutral_commanders_at_lawless_systems() {
    let galaxy = galaxy();
    let systems = galaxy.get_all_systems();
    let lawless = lawless_origin(&galaxy);
    let lawful = lawful_origin(&galaxy);

    let offered = |origin: &StarSystem, standing| {
        (0..200u64).any(|seed| {
            board_for_hold(
                origin,
                &systems,
                &mut StdRng::seed_from_u64(seed),
                standing,
                20,
            )
            .iter()
            .any(|job| job.kind == JobKind::BlackCourier)
        })
    };
    assert!(offered(lawless, Standing::Neutral));
    assert!(!offered(lawful, Standing::Neutral));
    for standing in [
        Standing::Criminal,
        Standing::Disreputable,
        Standing::Trusted,
        Standing::Criminal,
    ] {
        assert!(!offered(lawless, standing), "{:?}", standing);
    }
}

#[test]
fn the_gateway_job_is_occasional_not_the_whole_board() {
    let galaxy = galaxy();
    let systems = galaxy.get_all_systems();
    let origin = lawless_origin(&galaxy);

    let (mut gateway, mut total) = (0, 0);
    for seed in 0..300u64 {
        for job in generate_board_for(origin, &systems, &mut StdRng::seed_from_u64(seed), 20) {
            total += 1;
            gateway += i32::from(job.kind == JobKind::BlackCourier);
        }
    }
    let share = f64::from(gateway) / f64::from(total);
    assert!((0.15..=0.45).contains(&share), "share {}", share);
}

fn black_courier(destination: &str, reward: u32) -> ActiveJob {
    let mut entry = active_job(destination, reward, 2);
    entry.job.kind = JobKind::BlackCourier;
    entry
}

#[test]
fn delivering_a_black_courier_job_pays_and_costs_reputation() {
    let mut commander = Commander::new("Test".to_string());
    let mut active = vec![black_courier("Proxima Centauri", 62)];

    let messages = settle(&mut active, &mut commander, "Proxima Centauri").messages;

    assert!(active.is_empty());
    assert_eq!(commander.credits(), 162);
    assert_eq!(commander.reputation, -3);
    assert!(messages.iter().any(|m| m.contains("contraband parcel")));
}

#[test]
fn a_few_black_courier_deliveries_lead_to_the_criminal_track() {
    let mut commander = Commander::new("Test".to_string());
    for _ in 0..4 {
        let mut active = vec![black_courier("Proxima Centauri", 62)];
        settle(&mut active, &mut commander, "Proxima Centauri");
    }
    assert_eq!(commander.standing(), Standing::Disreputable);
    assert!(Track::Criminal.open_to(commander.standing()));
    assert!(!Track::Gateway.open_to(commander.standing()));
}
