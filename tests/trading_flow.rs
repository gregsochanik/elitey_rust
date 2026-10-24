mod common;

use common::{args, new_game};
use elitey_rust::data::commodities::CommodityType;

#[test]
fn buying_then_selling_updates_credits_cargo_and_log() {
    let mut game = new_game(100);

    assert!(game.handle_buy_command(&args("food 2")));
    let spent = 100 - game.commander.credits();
    assert!(spent > 0);
    assert_eq!(game.ship.cargo_hold.get_quantity(&CommodityType::Food), 2);

    assert!(game.handle_sell_command(&args("food 2")));
    assert_eq!(game.ship.cargo_hold.get_quantity(&CommodityType::Food), 0);

    let log = &game.trade_log;
    let transactions = log.recent(10);
    assert_eq!(transactions.len(), 2);
    let sold = transactions[0].total();
    let bought = transactions[1].total();
    assert_eq!(bought, spent);
    assert_eq!(game.commander.credits(), 100 - bought + sold);
    assert_eq!(log.stats().net_profit, i64::from(sold) - i64::from(bought));
}

#[test]
fn trading_moves_only_the_commodity_traded_and_only_a_little_per_ton() {
    use elitey_rust::data::commodities::CommodityType;

    let mut game = new_game(10_000);
    let before = game.market_prices().unwrap();

    assert!(game.handle_buy_command(&args("minerals 1")));
    let after = game.market_prices().unwrap();

    let first = f64::from(before.buy_prices[&CommodityType::Minerals]);
    let second = f64::from(after.buy_prices[&CommodityType::Minerals]);
    assert!(second >= first && second <= (first * 1.015).ceil());
    assert_eq!(
        before.buy_prices[&CommodityType::Textiles],
        after.buy_prices[&CommodityType::Textiles]
    );
}

#[test]
fn a_purchase_with_too_few_credits_leaves_everything_unchanged() {
    let mut game = new_game(1);

    assert!(!game.handle_buy_command(&args("luxury 5")));

    assert_eq!(game.commander.credits(), 1);
    assert_eq!(game.ship.cargo_used(), 0);
    assert!(game.trade_log.recent(10).is_empty());
}

#[test]
fn stock_runs_out_after_repeated_buying() {
    let mut game = new_game(1_000_000);
    let mut bought = 0;

    // Cargo space limits one trip, so empty the hold between purchases
    while game.handle_buy_command(&args("food 10")) {
        bought += 10;
        let held = game.ship.cargo_hold.get_quantity(&CommodityType::Food);
        game.ship
            .cargo_hold_mut()
            .remove_cargo(&CommodityType::Food, held);
        assert!(bought <= 100, "stock never ran out");
    }

    assert!(bought > 0);
}

#[test]
fn trading_narcotics_costs_reputation_each_transaction() {
    let mut game = new_game(2000);
    let before = game.commander.reputation;

    assert!(game.handle_buy_command(&args("narcotics 2")));
    assert_eq!(game.commander.reputation, before - 2);

    assert!(game.handle_sell_command(&args("narcotics 1")));
    assert_eq!(game.commander.reputation, before - 4);
}

#[test]
fn legal_goods_and_failed_trades_leave_reputation_alone() {
    let mut game = new_game(2000);

    assert!(game.handle_buy_command(&args("food 2")));
    assert!(game.handle_sell_command(&args("food 1")));
    assert!(!game.handle_buy_command(&args("narcotics 500")));
    assert!(!game.handle_sell_command(&args("narcotics 1")));

    assert_eq!(game.commander.reputation, 0);
}

fn latest_unit_price(game: &elitey_rust::game::GameState) -> u32 {
    game.trade_log.recent(1)[0].unit_price
}

fn standing_adjusts_the_same_market(
    reputation: i32,
    standing: elitey_rust::game::commander::Standing,
) {
    use elitey_rust::data::commodities::CommodityType;

    let mut game = new_game(5000);
    let neutral = game.market_prices().unwrap();
    let neutral_buy = neutral.buy_prices[&CommodityType::Food];
    let neutral_sell = neutral.sell_prices[&CommodityType::Food];

    game.commander.reputation = reputation;
    let adjusted = game.market_prices().unwrap();
    assert_eq!(
        adjusted.buy_prices[&CommodityType::Food],
        standing.buy_price(neutral_buy)
    );
    assert_eq!(
        adjusted.sell_prices[&CommodityType::Food],
        standing.sell_price(neutral_sell)
    );

    assert!(game.handle_buy_command(&args("food 1")));
    assert_eq!(
        latest_unit_price(&game),
        adjusted.buy_prices[&CommodityType::Food]
    );
    let after_buy = game.market_prices().unwrap();
    assert!(game.handle_sell_command(&args("food 1")));
    assert_eq!(
        latest_unit_price(&game),
        after_buy.sell_prices[&CommodityType::Food]
    );
}

#[test]
fn good_standing_pays_less_and_earns_more_on_the_same_market() {
    standing_adjusts_the_same_market(30, elitey_rust::game::commander::Standing::Respected);
}

#[test]
fn bad_standing_pays_more_and_earns_less_on_the_same_market() {
    standing_adjusts_the_same_market(-30, elitey_rust::game::commander::Standing::Criminal);
}
