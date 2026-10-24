use elitey_rust::data::commodities::TradingSystem;
use elitey_rust::data::systems::SystemManager;
use elitey_rust::game::commander::{Standing, STARTING_CREDITS};
use elitey_rust::game::refuel::CREDITS_PER_UNIT;

const TRIPS: [(&str, &str); 5] = [
    ("LAVE", "DISO"),
    ("DISO", "LAVE"),
    ("LAVE", "LEESTI"),
    ("LEESTI", "DISO"),
    ("DISO", "RIEDQUAT"),
];

struct Totals {
    trips: i64,
    gross: i64,
    fuel_cost: i64,
    losing: i64,
}

/// A trader who buys the affordable commodity furthest below its base price
fn informed_trader_totals(seeds: u64, standing: Standing) -> Totals {
    trader_totals(seeds, standing, STARTING_CREDITS)
}

fn trader_totals(seeds: u64, standing: Standing, capital: u32) -> Totals {
    let systems = SystemManager::new().unwrap();
    let trading = TradingSystem::new();
    let mut totals = Totals {
        trips: 0,
        gross: 0,
        fuel_cost: 0,
        losing: 0,
    };

    for seed in 0..seeds {
        for (from, to) in TRIPS {
            let origin = systems.get_system(from).unwrap();
            let destination = systems.get_system(to).unwrap();
            let buy_market = trading.generate_market_prices(&origin.economy, seed);
            let sell_market = trading.generate_market_prices(&destination.economy, seed + 10_000);
            let buy_price_of = |c: &_| standing.buy_price(buy_market.get_buy_price(c).unwrap());
            let sell_price_of = |c: &_| standing.sell_price(sell_market.get_sell_price(c).unwrap());

            let pick = trading
                .get_commodities()
                .iter()
                .filter(|c| buy_price_of(c) <= capital)
                .min_by_key(|c| c.percent_vs_base(buy_price_of(c)))
                .unwrap();
            let buy_price = buy_price_of(pick);
            let tons = (capital / buy_price)
                .min(20)
                .min(buy_market.availability[pick]);
            let gross = i64::from(tons) * (i64::from(sell_price_of(pick)) - i64::from(buy_price));
            let fuel_cost =
                (origin.distance_to(destination).ceil() as i64) * i64::from(CREDITS_PER_UNIT);

            totals.trips += 1;
            totals.gross += gross;
            totals.fuel_cost += fuel_cost;
            totals.losing += i64::from(gross - fuel_cost < 0);
        }
    }
    totals
}

#[test]
fn informed_trading_from_the_starting_credits_outearns_its_fuel() {
    let totals = informed_trader_totals(300, Standing::Neutral);
    let gross = totals.gross as f64 / totals.trips as f64;
    let fuel = totals.fuel_cost as f64 / totals.trips as f64;

    assert!(
        gross >= 2.0 * fuel,
        "gross {gross:.1} per trip, fuel {fuel:.1}"
    );
    assert!(
        gross >= 0.1 * f64::from(STARTING_CREDITS),
        "gross {gross:.1} per trip"
    );
}

#[test]
fn trips_can_still_lose_money_after_fuel() {
    let totals = informed_trader_totals(300, Standing::Neutral);
    let losing = totals.losing as f64 / totals.trips as f64;

    assert!(losing > 0.05, "losing {:.0}%", losing * 100.0);
    assert!(losing < 0.4, "losing {:.0}%", losing * 100.0);
}

fn gross_per_trip(standing: Standing) -> f64 {
    let totals = informed_trader_totals(300, standing);
    totals.gross as f64 / totals.trips as f64
}

#[test]
fn better_standing_earns_more_but_a_criminal_can_still_trade() {
    let criminal = gross_per_trip(Standing::Criminal);
    let disreputable = gross_per_trip(Standing::Disreputable);
    let neutral = gross_per_trip(Standing::Neutral);
    let trusted = gross_per_trip(Standing::Trusted);
    let respected = gross_per_trip(Standing::Respected);

    assert!(criminal < disreputable && disreputable < neutral);
    assert!(neutral < trusted && trusted < respected);

    assert!(respected >= 1.2 * neutral && respected <= 1.6 * neutral);
    assert!(criminal >= 0.5 * neutral && criminal <= 0.8 * neutral);

    let fuel = informed_trader_totals(300, Standing::Criminal).fuel_cost as f64
        / informed_trader_totals(300, Standing::Criminal).trips as f64;
    assert!(
        criminal > fuel,
        "criminal gross {criminal:.1}, fuel {fuel:.1}"
    );
}

fn best_courier_net_per_jump(seeds: u64) -> f64 {
    use elitey_rust::game::jobs::generate_board_for;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    let systems = SystemManager::new().unwrap();
    let all = systems.get_all_systems();
    let mut total = 0i64;
    let mut count = 0i64;
    for seed in 0..seeds {
        for (from, _) in TRIPS {
            let origin = systems.get_system(from).unwrap();
            let board = generate_board_for(
                origin,
                &all,
                &mut StdRng::seed_from_u64(seed),
                Standing::Neutral,
                20,
            );
            let best = board
                .iter()
                .filter(|job| job.kind == elitey_rust::game::jobs::JobKind::Courier)
                .map(|job| {
                    i64::from(job.reward) - i64::from(job.fuel_cost) * i64::from(CREDITS_PER_UNIT)
                })
                .max();
            if let Some(best) = best {
                total += best;
                count += 1;
            }
        }
    }
    total as f64 / count as f64
}

fn trading_net_per_trip(capital: u32) -> f64 {
    let totals = trader_totals(300, Standing::Neutral, capital);
    (totals.gross - totals.fuel_cost) as f64 / totals.trips as f64
}

#[test]
fn courier_jobs_beat_trading_when_poor_and_fall_behind_as_capital_grows() {
    let courier = best_courier_net_per_jump(300);

    assert!(
        courier >= 2.0 * trading_net_per_trip(100),
        "courier {courier:.1} vs trading {:.1} at 100cr",
        trading_net_per_trip(100)
    );
    assert!(
        courier <= trading_net_per_trip(500),
        "courier {courier:.1} vs trading {:.1} at 500cr",
        trading_net_per_trip(500)
    );
}

#[test]
fn random_pirate_fights_add_less_than_trading_and_courier_jobs() {
    use elitey_rust::game::aftermath::VICTORY_REWARD;

    // Measured share of jumps that meet a pirate attack, over every system
    const PIRATE_ATTACKS_PER_JUMP: f64 = 0.031;
    // A starting-loadout win costs about 28 hull to repair at 2cr per point
    const REPAIR_PER_FIGHT: f64 = 56.0;

    let average_payout = f64::from(*VICTORY_REWARD.start() + *VICTORY_REWARD.end()) / 2.0;
    let per_jump = PIRATE_ATTACKS_PER_JUMP * (average_payout - REPAIR_PER_FIGHT);

    assert!(
        per_jump <= trading_net_per_trip(100),
        "pirates {per_jump:.1} per jump vs trading {:.1}",
        trading_net_per_trip(100)
    );
    assert!(
        per_jump <= best_courier_net_per_jump(300),
        "pirates {per_jump:.1} per jump vs courier jobs"
    );
}

struct FightStats {
    win_rate: f64,
    average_hull_lost: f64,
}

fn starting_loadout_fights() -> FightStats {
    use elitey_rust::game::combat::{Action, Combat, Combatant, Outcome};
    use elitey_rust::game::Ship;
    use rand::Rng;
    use rand::SeedableRng;

    let ship = Ship::new_cobra_mk3();
    let fights = 1_000u64;
    let (mut lost, mut wins) = (0u64, 0u64);
    for seed in 0..fights {
        let mut combat = Combat::new(Combatant::from_ship(&ship), Combatant::pirate_raider());
        let start = combat.player.hull;
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        let mut roll = || rng.gen::<f64>();
        for _ in 0..500 {
            match combat.resolve_round(Action::Attack, &mut roll) {
                Ok(Outcome::Ongoing) => continue,
                Ok(Outcome::Victory) => wins += 1,
                _ => {}
            }
            break;
        }
        lost += u64::from(start - combat.player.hull);
    }
    FightStats {
        win_rate: wins as f64 / fights as f64,
        average_hull_lost: lost as f64 / fights as f64,
    }
}

fn best_bounty_net_per_jump(seeds: u64) -> f64 {
    use elitey_rust::game::jobs::{generate_board_for, JobKind};
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    let fights = starting_loadout_fights();
    let repair = fights.average_hull_lost * 2.0;
    let systems = SystemManager::new().unwrap();
    let all = systems.get_all_systems();
    let (mut total, mut count) = (0.0, 0);
    for seed in 0..seeds {
        for (from, _) in TRIPS {
            let origin = systems.get_system(from).unwrap();
            let board = generate_board_for(
                origin,
                &all,
                &mut StdRng::seed_from_u64(seed),
                Standing::Neutral,
                20,
            );
            let best = board
                .iter()
                .filter(|job| job.kind == JobKind::Bounty)
                .map(|job| {
                    fights.win_rate * f64::from(job.reward)
                        - f64::from(job.fuel_cost) * f64::from(CREDITS_PER_UNIT)
                        - repair
                })
                .fold(f64::MIN, f64::max);
            if best > f64::MIN {
                total += best;
                count += 1;
            }
        }
    }
    total / f64::from(count)
}

#[test]
fn the_starting_loadout_wins_every_bounty_fight() {
    let fights = starting_loadout_fights();
    assert!(fights.win_rate >= 0.99, "win rate {}", fights.win_rate);
}

#[test]
fn bounties_pay_more_than_courier_jobs_without_running_away() {
    let bounty = best_bounty_net_per_jump(300);
    let courier = best_courier_net_per_jump(300);

    assert!(
        bounty >= 1.2 * courier,
        "bounty {bounty:.1} vs courier {courier:.1}"
    );
    assert!(
        bounty <= 2.0 * courier,
        "bounty {bounty:.1} vs courier {courier:.1}"
    );
}
