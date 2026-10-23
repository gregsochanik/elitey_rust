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
fn prices_hold_still_while_trading_in_one_system() {
    let mut game = new_game(10_000);

    assert!(game.handle_buy_command(&args("minerals 1")));
    let first = game.trade_log.recent(1)[0].unit_price;
    assert!(game.handle_buy_command(&args("minerals 1")));
    let second = game.trade_log.recent(1)[0].unit_price;

    assert_eq!(first, second);
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
        game.ship.cargo_hold_mut().remove_cargo(&CommodityType::Food, held);
        assert!(bought <= 100, "stock never ran out");
    }

    assert!(bought > 0);
}
