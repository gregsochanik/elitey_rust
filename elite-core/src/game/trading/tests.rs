use super::*;

#[test]
fn test_parse_commodity() {
    let handler = TradingHandler::new();

    assert_eq!(handler.parse_commodity("food"), Some(CommodityType::Food));
    assert_eq!(handler.parse_commodity("FOOD"), Some(CommodityType::Food));
    assert_eq!(
        handler.parse_commodity("machinery"),
        Some(CommodityType::Machinery)
    );
    assert_eq!(handler.parse_commodity("unknown"), None);
}

#[test]
fn prices_are_stable_across_arrivals_within_a_period_and_reroll_after_it() {
    let mut handler = TradingHandler::new();
    let economy = EconomyType::RichIndustrial;

    let first = handler.market_for("Sol", &economy, 0);
    let seed = handler.markets["Sol"].seed;
    handler.market_for("Proxima Centauri", &economy, 1);
    let back = handler.market_for("Sol", &economy, 2);
    assert_eq!(first.buy_prices, back.buy_prices);
    assert_eq!(first.availability, back.availability);
    assert_eq!(handler.markets["Sol"].seed, seed);

    handler.market_for("Sol", &economy, MARKET_REFRESH_JUMPS - 1);
    assert_eq!(handler.markets["Sol"].seed, seed);

    handler.market_for("Sol", &economy, MARKET_REFRESH_JUMPS);
    assert_ne!(handler.markets["Sol"].seed, seed);
    assert_eq!(handler.markets["Sol"].rolled_at, MARKET_REFRESH_JUMPS);
}

#[test]
fn test_trading_handler_creation() {
    let handler = TradingHandler::new();
    assert_eq!(handler.trading_system.get_commodities().len(), 8);
}

fn market(buy: u32, sell: u32, available: u32) -> MarketPrices {
    let mut m = MarketPrices::new();
    m.buy_prices.insert(CommodityType::Food, buy);
    m.sell_prices.insert(CommodityType::Food, sell);
    m.availability.insert(CommodityType::Food, available);
    m
}

#[test]
fn successful_buy_and_sell_are_logged() {
    let mut handler = TradingHandler::new();
    let mut commander = Commander::new("Test".to_string());
    let mut ship = Ship::new_cobra_mk3();
    let mut log = TradeLog::new();

    assert!(handler
        .execute_buy(
            CommodityType::Food,
            5,
            &mut commander,
            &mut ship,
            &market(10, 8, 20),
            &mut log
        )
        .is_ok());
    assert!(handler
        .execute_sell(
            CommodityType::Food,
            5,
            &mut commander,
            &mut ship,
            &market(12, 15, 20),
            &mut log
        )
        .is_ok());

    assert_eq!(log.transactions().len(), 2);
    assert_eq!(log.transactions()[1].profit, Some(25));
    assert_eq!(log.transactions()[0].location, commander.location());
}

#[test]
fn failed_buy_and_sell_are_not_logged() {
    let mut handler = TradingHandler::new();
    let mut commander = Commander::new("Test".to_string());
    let mut ship = Ship::new_cobra_mk3();
    let mut log = TradeLog::new();

    // too expensive, not enough stock, nothing to sell
    assert!(handler
        .execute_buy(
            CommodityType::Food,
            5,
            &mut commander,
            &mut ship,
            &market(1_000_000, 8, 20),
            &mut log
        )
        .is_err());
    assert!(handler
        .execute_buy(
            CommodityType::Food,
            5,
            &mut commander,
            &mut ship,
            &market(10, 8, 2),
            &mut log
        )
        .is_err());
    assert!(handler
        .execute_sell(
            CommodityType::Food,
            5,
            &mut commander,
            &mut ship,
            &market(10, 8, 20),
            &mut log
        )
        .is_err());

    assert!(log.transactions().is_empty());
}

#[test]
fn buying_reduces_stock_until_the_market_refreshes() {
    let mut handler = TradingHandler::new();
    let economy = EconomyType::RichIndustrial;
    let mut commander = Commander::new("Test".to_string());
    let mut ship = Ship::new_cobra_mk3();
    let mut log = TradeLog::new();

    let before = handler.market_for("Sol", &economy, 0);
    let stock = before.get_availability(&CommodityType::Food).unwrap();
    assert!(handler
        .execute_buy(
            CommodityType::Food,
            1,
            &mut commander,
            &mut ship,
            &before,
            &mut log
        )
        .is_ok());

    let after = handler.market_for("Sol", &economy, 0);
    assert_eq!(
        after.get_availability(&CommodityType::Food),
        Some(stock - 1)
    );
    assert_eq!(
        after.get_availability(&CommodityType::Textiles),
        before.get_availability(&CommodityType::Textiles)
    );

    assert_eq!(handler.markets["Sol"].purchased.len(), 1);
    handler.market_for("Proxima Centauri", &economy, 1);
    let back = handler.market_for("Sol", &economy, 2);
    assert_eq!(back.get_availability(&CommodityType::Food), Some(stock - 1));

    handler.market_for("Sol", &economy, MARKET_REFRESH_JUMPS);
    assert!(handler.markets["Sol"].purchased.is_empty());
}

#[test]
fn cannot_buy_more_than_remaining_stock() {
    let mut handler = TradingHandler::new();
    let mut commander = Commander::new("Test".to_string());
    let mut ship = Ship::new_cobra_mk3();
    let mut log = TradeLog::new();
    commander.add_credits(1_000);

    let mut stock = market(10, 8, 6);
    assert!(handler
        .execute_buy(
            CommodityType::Food,
            5,
            &mut commander,
            &mut ship,
            &stock,
            &mut log
        )
        .is_ok());
    stock.availability.insert(CommodityType::Food, 1);
    assert!(handler
        .execute_buy(
            CommodityType::Food,
            2,
            &mut commander,
            &mut ship,
            &stock,
            &mut log
        )
        .is_err());
}

#[test]
fn system_markets_remember_their_state_across_visits() {
    let mut handler = TradingHandler::new();
    let economy = EconomyType::RichIndustrial;
    handler.market_for("Sol", &economy, 3);
    handler.market_for("Proxima Centauri", &economy, 4);

    assert_eq!(handler.markets.len(), 2);
    assert_eq!(handler.markets["Sol"].rolled_at, 3);
    assert_eq!(handler.markets["Proxima Centauri"].rolled_at, 4);
}

#[test]
fn markets_survive_serialization() {
    let mut handler = TradingHandler::new();
    handler.market_for("Sol", &EconomyType::RichIndustrial, 7);
    handler
        .markets
        .get_mut("Sol")
        .unwrap()
        .purchased
        .insert(CommodityType::Food, 4.0);

    let json = serde_json::to_string(&handler).unwrap();
    let restored: TradingHandler = serde_json::from_str(&json).unwrap();

    assert_eq!(restored.markets["Sol"].seed, handler.markets["Sol"].seed);
    assert_eq!(restored.markets["Sol"].rolled_at, 7);
    assert_eq!(
        restored.markets["Sol"].purchased.get(&CommodityType::Food),
        Some(&4.0)
    );
}

#[test]
fn old_saves_with_the_single_market_fields_load_with_no_markets() {
    let json = r#"{"trading_system":{"commodities":["Food"]},"market_seed":5,"market_location":"Sol","purchased":{"Food":2}}"#;
    let restored: TradingHandler = serde_json::from_str(json).unwrap();
    assert!(restored.markets.is_empty());
}

fn apply(state: &SystemMarket) -> MarketPrices {
    let mut market = market(100, 90, 50);
    state.apply_to(&mut market);
    market
}

#[test]
fn selling_lowers_the_sell_price_by_one_and_a_half_percent_per_ton() {
    let mut state = SystemMarket::rolled(1, 0);
    state.record_sell(&CommodityType::Food, 10);

    let after = apply(&state);
    assert_eq!(after.get_sell_price(&CommodityType::Food), Some(77));
    assert_eq!(after.get_buy_price(&CommodityType::Food), Some(100));
}

#[test]
fn buying_raises_the_buy_price_and_lowers_stock() {
    let mut state = SystemMarket::rolled(1, 0);
    state.record_buy(&CommodityType::Food, 10);

    let after = apply(&state);
    assert_eq!(after.get_buy_price(&CommodityType::Food), Some(115));
    assert_eq!(after.get_sell_price(&CommodityType::Food), Some(90));
    assert_eq!(after.get_availability(&CommodityType::Food), Some(40));
}

#[test]
fn impact_is_capped_at_forty_percent() {
    let mut state = SystemMarket::rolled(1, 0);
    state.record_sell(&CommodityType::Food, 200);
    state.record_buy(&CommodityType::Food, 200);

    let after = apply(&state);
    assert_eq!(after.get_sell_price(&CommodityType::Food), Some(54));
    assert_eq!(after.get_buy_price(&CommodityType::Food), Some(140));
}

#[test]
fn impact_and_stock_recover_ten_percent_of_the_gap_per_jump() {
    let mut state = SystemMarket::rolled(1, 0);
    state.record_sell(&CommodityType::Food, 20);
    state.record_buy(&CommodityType::Food, 20);

    state.recover(1);
    assert!((state.sold_impact[&CommodityType::Food] - 0.27).abs() < 1e-9);
    assert!((state.purchased[&CommodityType::Food] - 18.0).abs() < 1e-9);

    state.recover(16);
    let left = state.sold_impact[&CommodityType::Food] / 0.30;
    assert!((left - 0.9_f64.powi(16)).abs() < 1e-9);
    assert!(left < 0.2);
}

#[test]
fn recovery_happens_at_systems_the_player_is_not_in() {
    let mut handler = TradingHandler::new();
    let economy = EconomyType::RichIndustrial;
    handler.market_for("Sol", &economy, 0);
    handler
        .markets
        .get_mut("Sol")
        .unwrap()
        .record_sell(&CommodityType::Food, 20);

    handler.market_for("Proxima Centauri", &economy, 3);
    handler.market_for("Sol", &economy, 3);

    let impact = handler.markets["Sol"].sold_impact[&CommodityType::Food];
    assert!((impact - 0.30 * 0.9_f64.powi(3)).abs() < 1e-9);
}

#[test]
fn a_refresh_clears_the_impact() {
    let mut handler = TradingHandler::new();
    let economy = EconomyType::RichIndustrial;
    handler.market_for("Sol", &economy, 0);
    handler
        .markets
        .get_mut("Sol")
        .unwrap()
        .record_sell(&CommodityType::Food, 20);

    handler.market_for("Sol", &economy, MARKET_REFRESH_JUMPS);

    assert!(handler.markets["Sol"].sold_impact.is_empty());
}

#[test]
fn old_markets_without_impact_fields_load_untouched() {
    let json = r#"{"seed":3,"rolled_at":2,"purchased":{"Food":4.0}}"#;
    let state: SystemMarket = serde_json::from_str(json).unwrap();
    assert!(state.sold_impact.is_empty());
    assert_eq!(state.updated_at, 0);
}

#[test]
fn markers_show_for_sixteen_jumps_after_trading() {
    let mut state = SystemMarket::rolled(1, 0);
    state.recover(2);
    state.record_sell(&CommodityType::Food, 5);
    state.record_buy(&CommodityType::Textiles, 5);

    let food = state.pushed(&CommodityType::Food, 2);
    assert_eq!(
        food,
        Pushed {
            sold: true,
            bought: false
        }
    );
    assert_eq!(food.markers(), vec!["▼ sold here"]);
    let textiles = state.pushed(&CommodityType::Textiles, 2);
    assert_eq!(textiles.markers(), vec!["▲ bought here"]);
    assert_eq!(state.pushed(&CommodityType::Minerals, 2), Pushed::default());

    assert!(
        state
            .pushed(&CommodityType::Food, 2 + MARKER_JUMPS - 1)
            .sold
    );
    assert!(!state.pushed(&CommodityType::Food, 2 + MARKER_JUMPS).sold);
}

#[test]
fn a_commodity_both_bought_and_sold_shows_both_markers() {
    let mut state = SystemMarket::rolled(1, 0);
    state.record_buy(&CommodityType::Food, 1);
    state.record_sell(&CommodityType::Food, 1);

    assert_eq!(
        state.pushed(&CommodityType::Food, 0).markers(),
        vec!["▼ sold here", "▲ bought here"]
    );
}

#[test]
fn a_refresh_clears_the_markers() {
    let mut handler = TradingHandler::new();
    let economy = EconomyType::RichIndustrial;
    handler.market_for("Sol", &economy, 0);
    handler
        .markets
        .get_mut("Sol")
        .unwrap()
        .record_sell(&CommodityType::Food, 3);
    assert!(!handler.pushed_commodities("Sol", 1).is_empty());

    handler.market_for("Sol", &economy, MARKET_REFRESH_JUMPS);

    assert!(handler
        .pushed_commodities("Sol", MARKET_REFRESH_JUMPS)
        .is_empty());
}

#[test]
fn a_buy_returns_a_receipt_with_the_price_total_and_credits_left() {
    let mut handler = TradingHandler::new();
    let mut commander = Commander::new("Test".to_string());
    commander.credits = 100;
    let mut ship = Ship::new_cobra_mk3();
    let mut log = TradeLog::new();

    let receipt = handler
        .execute_buy(
            CommodityType::Food,
            5,
            &mut commander,
            &mut ship,
            &market(10, 8, 20),
            &mut log,
        )
        .unwrap();

    assert_eq!(receipt.kind, TradeKind::Buy);
    assert_eq!(receipt.quantity, 5);
    assert_eq!(receipt.unit_price, 10);
    assert_eq!(receipt.total, 50);
    assert_eq!(receipt.credits_after, 50);
    assert_eq!(receipt.illegal_penalty, None);
}

#[test]
fn refusals_say_what_was_missing() {
    let mut handler = TradingHandler::new();
    let mut commander = Commander::new("Test".to_string());
    commander.credits = 20;
    let mut ship = Ship::new_cobra_mk3();
    let mut log = TradeLog::new();
    let mut buy = |price, stock| {
        handler.execute_buy(
            CommodityType::Food,
            5,
            &mut commander,
            &mut ship,
            &market(price, 8, stock),
            &mut log,
        )
    };

    assert_eq!(
        buy(10, 2),
        Err(TradeError::NotEnoughStock {
            available: 2,
            requested: 5
        })
    );
    assert_eq!(
        buy(10, 20),
        Err(TradeError::NotEnoughCredits {
            needed: 50,
            have: 20
        })
    );
    assert_eq!(
        TradeError::NotEnoughCredits {
            needed: 50,
            have: 20
        }
        .to_string(),
        "Insufficient credits. Need 50cr, have 20cr"
    );
}

#[test]
fn selling_what_you_do_not_hold_is_refused() {
    let mut handler = TradingHandler::new();
    let mut commander = Commander::new("Test".to_string());
    let mut ship = Ship::new_cobra_mk3();
    let mut log = TradeLog::new();

    assert_eq!(
        handler.execute_sell(
            CommodityType::Food,
            3,
            &mut commander,
            &mut ship,
            &market(10, 8, 20),
            &mut log
        ),
        Err(TradeError::NotEnoughCargo {
            held: 0,
            requested: 3
        })
    );
}

#[test]
fn the_market_overview_lists_every_commodity_with_prices() {
    use crate::data::systems::SystemManager;
    let systems = SystemManager::new().unwrap();
    let mut handler = TradingHandler::new();

    let view = handler
        .market_overview(
            &systems,
            "Sol",
            Standing::Neutral,
            0,
            &Ship::new_cobra_mk3(),
            &TradeLog::new(),
        )
        .unwrap();

    assert_eq!(view.system_name, "Sol");
    assert_eq!(view.rows.len(), 8);
    assert!(view.rows.iter().all(|row| row.sell_price <= row.buy_price));
    assert!(handler
        .market_overview(
            &systems,
            "NOWHERE",
            Standing::Neutral,
            0,
            &Ship::new_cobra_mk3(),
            &TradeLog::new(),
        )
        .is_none());
}

#[test]
fn the_cargo_view_totals_the_hold() {
    let handler = TradingHandler::new();
    let mut ship = Ship::new_cobra_mk3();
    assert!(ship.cargo_hold.add_cargo(CommodityType::Food, 3));

    let view = handler.cargo_view(&ship, &TradeLog::new());

    assert_eq!(view.weight, 3);
    assert_eq!(view.capacity, 20);
    assert_eq!(view.items.len(), 1);
    assert_eq!(view.total_value, 3 * CommodityType::Food.base_price());
}

#[test]
fn a_seeded_handler_rolls_the_same_markets_every_time() {
    let economy = EconomyType::RichIndustrial;
    let mut first = TradingHandler::with_seed(7);
    let mut second = TradingHandler::with_seed(7);
    let mut other = TradingHandler::with_seed(8);

    let a = first.market_for("Sol", &economy, 0);
    assert_eq!(
        a.buy_prices,
        second.market_for("Sol", &economy, 0).buy_prices
    );
    assert_ne!(
        a.buy_prices,
        other.market_for("Sol", &economy, 0).buy_prices
    );
    assert_ne!(
        first
            .market_for("Sol", &economy, MARKET_REFRESH_JUMPS)
            .buy_prices,
        a.buy_prices
    );
}

#[test]
fn the_views_show_what_held_cargo_cost_on_average() {
    use crate::data::systems::SystemManager;
    let systems = SystemManager::new().unwrap();
    let mut handler = TradingHandler::new();
    let mut ship = Ship::new_cobra_mk3();
    let mut log = TradeLog::new();
    ship.add_cargo(CommodityType::Food, 2);
    log.record_buy(CommodityType::Food, 2, 20, "Sol", 0);
    ship.add_cargo(CommodityType::Food, 2);
    log.record_buy(CommodityType::Food, 2, 30, "Sol", 2);

    let cargo = handler.cargo_view(&ship, &log);
    assert_eq!(cargo.items[0].paid, Some(25));

    let market = handler
        .market_overview(&systems, "Sol", Standing::Neutral, 0, &ship, &log)
        .unwrap();
    let food = market
        .rows
        .iter()
        .find(|row| row.commodity == CommodityType::Food)
        .unwrap();
    assert_eq!((food.held, food.paid), (4, Some(25)));
    let textiles = market
        .rows
        .iter()
        .find(|row| row.commodity == CommodityType::Textiles)
        .unwrap();
    assert_eq!((textiles.held, textiles.paid), (0, None));
}
