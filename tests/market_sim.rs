mod common;

use common::new_game;
use elitey_rust::data::commodities::CommodityType;
use elitey_rust::game::commander::Standing;

const A: &str = "LEESTI";
const B: &str = "RIEDQUAT";

/// Trade between two systems with full knowledge of the other end's current prices, buying the
/// legal commodity that pays best each trip; returns net credits per jump and income per trip
fn route_trader(capital: u32, jumps: u32) -> (f64, Vec<f64>) {
    let mut game = new_game(capital);
    game.commander.location = A.to_string();
    let start = f64::from(game.commander.credits());
    let mut per_trip = Vec::new();
    let mut here = A;
    let mut last = start;

    for _ in 0..jumps {
        let there = if here == A { B } else { A };
        let mine = game.market_prices().unwrap();
        let theirs = game
            .trading_handler
            .market_view(
                &game.system_manager,
                there,
                Standing::Neutral,
                game.world_clock,
            )
            .unwrap();
        let credits = game.commander.credits();
        let free = game.ship.cargo_available();

        let best = [
            CommodityType::Food,
            CommodityType::Textiles,
            CommodityType::Machinery,
            CommodityType::Computers,
            CommodityType::Minerals,
            CommodityType::Alloys,
            CommodityType::Luxury,
        ]
        .into_iter()
        .map(|c| {
            let buy = mine.buy_prices[&c];
            let sell = theirs.sell_prices[&c];
            let tons = (credits / buy).min(free).min(mine.availability[&c]);
            let profit = i64::from(tons) * (i64::from(sell) - i64::from(buy));
            (profit, c, tons)
        })
        .max_by_key(|(profit, _, _)| *profit);

        if let Some((profit, c, tons)) = best {
            if profit > 0 && tons > 0 {
                elitey_rust::cli::trading::buy(
                    &mut game,
                    &[c.name().to_lowercase(), tons.to_string()],
                );
            }
        }
        game.ship.refuel();
        game.travel_to_system(there).unwrap();
        here = there;
        let cargo: Vec<(CommodityType, u32)> = game
            .ship
            .cargo_hold
            .get_items()
            .iter()
            .map(|i| (i.commodity.clone(), i.quantity))
            .collect();
        for (c, q) in cargo {
            elitey_rust::cli::trading::sell(&mut game, &[c.name().to_lowercase(), q.to_string()]);
        }
        let now = f64::from(game.commander.credits());
        per_trip.push(now - last);
        last = now;
    }
    let net = (last - start) - f64::from(jumps) * 2.0 * 3.0;
    (net / f64::from(jumps), per_trip)
}

fn average_route_income(capital: u32) -> (f64, f64, f64) {
    let games = 150;
    let (mut net, mut early, mut late) = (0.0, 0.0, 0.0);
    for _ in 0..games {
        let (income, trips) = route_trader(capital, 40);
        net += income;
        early += trips[..4].iter().sum::<f64>() / 4.0;
        late += trips[30..].iter().sum::<f64>() / 10.0;
    }
    let games = f64::from(games);
    (net / games, early / games, late / games)
}

#[test]
fn a_two_system_route_pays_no_more_than_a_bounty_once_the_market_is_worked() {
    // Best bounty nets about 69 per jump (Feature 17); the markets are random, so allow for noise
    for capital in [100u32, 500, 2000] {
        let (net, _, _) = average_route_income(capital);
        assert!(net <= 88.0, "capital {capital}: net per jump {net:.1}");
        assert!(net >= 40.0, "capital {capital}: net per jump {net:.1}");
    }
}

#[test]
fn working_one_route_pays_less_the_longer_it_is_worked() {
    let (_, early, late) = average_route_income(2000);
    assert!(
        late <= early / 2.0,
        "first trips {early:.1} per trip, last trips {late:.1}"
    );
}
