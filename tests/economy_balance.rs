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
                .filter(|c| buy_price_of(c) <= STARTING_CREDITS)
                .min_by_key(|c| c.percent_vs_base(buy_price_of(c)))
                .unwrap();
            let buy_price = buy_price_of(pick);
            let tons = (STARTING_CREDITS / buy_price)
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
