use super::*;
use crate::data::systems::EconomyType;

#[test]
fn test_commodity_base_prices() {
    assert_eq!(CommodityType::Food.base_price(), 15);
    assert_eq!(CommodityType::Machinery.base_price(), 45);
    assert_eq!(CommodityType::Narcotics.base_price(), 120);
}

#[test]
fn test_commodity_names() {
    assert_eq!(CommodityType::Food.name(), "Food");
    assert_eq!(CommodityType::Narcotics.name(), "Narcotics");
}

#[test]
fn test_illegal_commodities() {
    assert!(!CommodityType::Food.is_illegal());
    assert!(CommodityType::Narcotics.is_illegal());
}

#[test]
fn test_economy_preferences() {
    // Food should be preferred in agricultural economies
    assert!(CommodityType::Food.economy_preference(&EconomyType::RichAgricultural) > 1.0);
    assert!(CommodityType::Food.economy_preference(&EconomyType::RichIndustrial) == 1.0);

    // Machinery should be preferred in industrial economies
    assert!(CommodityType::Machinery.economy_preference(&EconomyType::RichIndustrial) > 1.0);
    assert!(CommodityType::Machinery.economy_preference(&EconomyType::RichAgricultural) == 1.0);
}

#[test]
fn test_cargo_hold_operations() {
    let mut cargo = CargoHold::new(20);

    // Test adding cargo
    assert!(cargo.add_cargo(CommodityType::Food, 5));
    assert_eq!(cargo.current_weight(), 5);
    assert_eq!(cargo.remaining_capacity(), 15);

    // Test adding more of same commodity
    assert!(cargo.add_cargo(CommodityType::Food, 3));
    assert_eq!(cargo.get_quantity(&CommodityType::Food), 8);

    // Test capacity limit
    assert!(!cargo.add_cargo(CommodityType::Machinery, 15)); // Would exceed capacity

    // Test removing cargo
    assert!(cargo.remove_cargo(&CommodityType::Food, 3));
    assert_eq!(cargo.get_quantity(&CommodityType::Food), 5);

    // Test removing more than available
    assert!(!cargo.remove_cargo(&CommodityType::Food, 10));
}

#[test]
fn test_market_generation() {
    let trading_system = TradingSystem::new();
    let market = trading_system.generate_market_prices(&EconomyType::RichAgricultural, 1);

    // Check that all commodities have prices
    assert_eq!(market.buy_prices.len(), 8);
    assert_eq!(market.sell_prices.len(), 8);
    assert_eq!(market.availability.len(), 8);

    // Check that buy prices are higher than sell prices
    for commodity in trading_system.get_commodities() {
        let buy_price = market.get_buy_price(commodity).unwrap();
        let sell_price = market.get_sell_price(commodity).unwrap();
        assert!(buy_price >= sell_price);
    }
}

#[test]
fn same_seed_gives_same_prices_and_different_seed_differs() {
    let system = TradingSystem::new();
    let a = system.generate_market_prices(&EconomyType::RichAgricultural, 7);
    let b = system.generate_market_prices(&EconomyType::RichAgricultural, 7);
    let c = system.generate_market_prices(&EconomyType::RichAgricultural, 8);

    assert_eq!(a.buy_prices, b.buy_prices);
    assert_eq!(a.availability, b.availability);
    assert_ne!(a.buy_prices, c.buy_prices);
}

#[test]
fn best_commodity_on_a_short_trip_usually_profits() {
    let systems = crate::data::systems::SystemManager::new().expect("Failed to load galaxy data");
    let from = systems.get_system("Sol").unwrap().economy.clone();
    let to = systems
        .get_system("Proxima Centauri")
        .unwrap()
        .economy
        .clone();
    let trading = TradingSystem::new();

    let runs = 500u64;
    let profitable = (0..runs)
        .filter(|seed| {
            let buy_market = trading.generate_market_prices(&from, *seed);
            let sell_market = trading.generate_market_prices(&to, seed + 10_000);
            trading.get_commodities().iter().any(|c| {
                sell_market.get_sell_price(c).unwrap() > buy_market.get_buy_price(c).unwrap()
            })
        })
        .count();

    assert!(
        profitable as f64 / runs as f64 >= 0.9,
        "profitable in {}/{} runs",
        profitable,
        runs
    );
}

#[test]
fn a_poor_choice_can_still_lose_money() {
    let trading = TradingSystem::new();
    let market = trading.generate_market_prices(&EconomyType::RichAgricultural, 3);
    for c in trading.get_commodities() {
        assert!(market.get_sell_price(c).unwrap() <= market.get_buy_price(c).unwrap());
    }
}

#[test]
fn percent_vs_base_shows_distance_from_base_price() {
    assert_eq!(CommodityType::Food.percent_vs_base(15), 0);
    assert_eq!(CommodityType::Food.percent_vs_base(12), -20);
    assert_eq!(CommodityType::Food.percent_vs_base(18), 20);
}

#[test]
fn buying_the_cheapest_against_base_beats_buying_blind() {
    let systems = crate::data::systems::SystemManager::new().expect("Failed to load galaxy data");
    let trading = TradingSystem::new();
    let pairs = [
        ("Sol", "Proxima Centauri"),
        ("Proxima Centauri", "Sol"),
        ("Sol", "Alpha Centauri"),
        ("Alpha Centauri", "Proxima Centauri"),
    ];

    let (mut informed_wins, mut blind_wins, mut runs) = (0, 0, 0);
    for (from, to) in pairs {
        let from_economy = systems.get_system(from).unwrap().economy.clone();
        let to_economy = systems.get_system(to).unwrap().economy.clone();
        for seed in 0..300u64 {
            let buy_market = trading.generate_market_prices(&from_economy, seed);
            let sell_market = trading.generate_market_prices(&to_economy, seed + 10_000);
            let profit = |c: &CommodityType| {
                i64::from(sell_market.get_sell_price(c).unwrap())
                    - i64::from(buy_market.get_buy_price(c).unwrap())
            };

            let cheapest = trading
                .get_commodities()
                .iter()
                .min_by_key(|c| c.percent_vs_base(buy_market.get_buy_price(c).unwrap()))
                .unwrap();
            let blind = &trading.get_commodities()[(seed % 8) as usize];

            runs += 1;
            informed_wins += i32::from(profit(cheapest) > 0);
            blind_wins += i32::from(profit(blind) > 0);
        }
    }

    assert!(informed_wins as f64 / runs as f64 >= 0.6);
    assert!(informed_wins > blind_wins);
}

#[test]
fn commodity_data_is_consistent() {
    let data = commodity_data();
    assert!(data.len() >= 2);

    let ids: std::collections::HashSet<_> = data.iter().map(|c| c.id.as_str()).collect();
    assert_eq!(ids.len(), data.len(), "commodity ids must be unique");
    let names: std::collections::HashSet<_> = data.iter().map(|c| name_key(&c.name)).collect();
    assert_eq!(names.len(), data.len(), "commodity names must be unique");

    for commodity in data {
        assert!(!commodity.id.is_empty() && !commodity.name.is_empty());
        assert!(commodity.base_price > 0, "{}", commodity.id);
        assert!(
            commodity.id.chars().all(|c| c.is_ascii_lowercase()),
            "{}",
            commodity.id
        );
        for multiplier in std::iter::once(&commodity.default_multiplier)
            .chain(commodity.economy_multipliers.values())
        {
            assert!(
                (0.5..=2.0).contains(multiplier),
                "{} has multiplier {}",
                commodity.id,
                multiplier
            );
        }
    }
}

#[test]
fn the_named_commodities_exist_in_the_data() {
    for (named, id) in [
        (CommodityType::Food, "food"),
        (CommodityType::Textiles, "textiles"),
        (CommodityType::Machinery, "machinery"),
        (CommodityType::Computers, "computers"),
        (CommodityType::Minerals, "minerals"),
        (CommodityType::Alloys, "alloys"),
        (CommodityType::Luxury, "luxury"),
        (CommodityType::Narcotics, "narcotics"),
    ] {
        assert_eq!(named.id(), id);
        assert!(!named.name().is_empty(), "{}", id);
    }
    assert_eq!(CommodityType::all().len(), commodity_data().len());
}

#[test]
fn only_narcotics_are_illegal_and_cheaper_everywhere() {
    let illegal: Vec<_> = CommodityType::all()
        .into_iter()
        .filter(CommodityType::is_illegal)
        .collect();
    assert_eq!(illegal, vec![CommodityType::Narcotics]);
    assert_eq!(
        CommodityType::Narcotics.economy_preference(&EconomyType::RichIndustrial),
        0.8
    );
    assert_eq!(
        CommodityType::Food.economy_preference(&EconomyType::RichIndustrial),
        1.0
    );
    assert_eq!(
        CommodityType::Food.economy_preference(&EconomyType::RichAgricultural),
        1.5
    );
}

#[test]
fn commodities_serialise_as_ids_and_load_the_names_old_saves_used() {
    assert_eq!(
        serde_json::to_string(&CommodityType::Luxury).unwrap(),
        "\"luxury\""
    );
    for (old, expected) in [
        ("Food", CommodityType::Food),
        ("Narcotics", CommodityType::Narcotics),
        ("luxury", CommodityType::Luxury),
    ] {
        let loaded: CommodityType = serde_json::from_str(&format!("\"{}\"", old)).unwrap();
        assert_eq!(loaded, expected, "{}", old);
    }
    assert!(serde_json::from_str::<CommodityType>("\"Spice\"").is_err());

    let mut hold = CargoHold::new(10);
    hold.add_cargo(CommodityType::Food, 3);
    let json = serde_json::to_string(&hold).unwrap();
    assert!(json.contains("\"food\""));
    let old = r#"{"items":{"Food":3},"capacity":10}"#;
    let loaded: CargoHold = serde_json::from_str(old).unwrap();
    assert_eq!(loaded.get_quantity(&CommodityType::Food), 3);
}

#[test]
fn unknown_fields_and_unknown_economies_in_the_data_are_rejected() {
    let extra = r#"[{"id":"x","name":"X","base_price":1,"colour":"red"}]"#;
    assert!(serde_json::from_str::<Vec<CommodityData>>(extra).is_err());
    let economy =
        r#"[{"id":"x","name":"X","base_price":1,"economy_multipliers":{"Moon Base":1.2}}]"#;
    assert!(serde_json::from_str::<Vec<CommodityData>>(economy).is_err());
    let minimal = r#"[{"id":"x","name":"X","base_price":1}]"#;
    let parsed: Vec<CommodityData> = serde_json::from_str(minimal).unwrap();
    assert_eq!(parsed[0].default_multiplier, 1.0);
    assert!(!parsed[0].illegal);
}
