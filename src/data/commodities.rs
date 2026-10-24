use crate::data::systems::EconomyType;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::OnceLock;

/// One tradeable good, as described in `data/commodities.json`
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommodityData {
    pub id: String,
    pub name: String,
    pub base_price: u32,
    #[serde(default)]
    pub illegal: bool,
    /// Price multiplier in economies not listed below
    #[serde(default = "no_preference")]
    pub default_multiplier: f64,
    /// Price multiplier by economy, as named in the galaxy data ("Rich Ind", "Poor Agri", ...)
    #[serde(default)]
    pub economy_multipliers: HashMap<EconomyType, f64>,
}

fn no_preference() -> f64 {
    1.0
}

/// Every commodity in the game, in the order the market lists them
pub fn commodity_data() -> &'static [CommodityData] {
    static COMMODITIES: OnceLock<Vec<CommodityData>> = OnceLock::new();
    COMMODITIES.get_or_init(|| {
        serde_json::from_str(include_str!("../../data/commodities.json"))
            .expect("data/commodities.json is valid")
    })
}

/// Lower case letters and digits only, so "Food" and "food" are the same name
fn name_key(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect::<String>()
        .to_lowercase()
}

/// A commodity, identified by its id in `data/commodities.json`. It serialises as that id and
/// loads the capitalised names older saves used (`Food`, `Narcotics`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CommodityType(&'static str);

/// Names for the commodities the code refers to directly; any other commodity in the data
/// is reached by id with `CommodityType::from_string`
#[allow(non_upper_case_globals, dead_code)]
impl CommodityType {
    pub const Food: CommodityType = CommodityType("food");
    pub const Textiles: CommodityType = CommodityType("textiles");
    pub const Machinery: CommodityType = CommodityType("machinery");
    pub const Computers: CommodityType = CommodityType("computers");
    pub const Minerals: CommodityType = CommodityType("minerals");
    pub const Alloys: CommodityType = CommodityType("alloys");
    pub const Luxury: CommodityType = CommodityType("luxury");
    pub const Narcotics: CommodityType = CommodityType("narcotics");
}

impl Serialize for CommodityType {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.id())
    }
}

impl<'de> Deserialize<'de> for CommodityType {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let name = String::deserialize(deserializer)?;
        CommodityType::from_string(&name)
            .ok_or_else(|| serde::de::Error::custom(format!("unknown commodity '{}'", name)))
    }
}

impl CommodityType {
    /// Match a commodity by id or name, ignoring case
    pub fn from_string(name: &str) -> Option<Self> {
        let key = name_key(name);
        commodity_data()
            .iter()
            .find(|data| name_key(&data.id) == key || name_key(&data.name) == key)
            .map(|data| CommodityType(data.id.as_str()))
    }

    /// Every commodity, in market order
    pub fn all() -> Vec<CommodityType> {
        commodity_data()
            .iter()
            .map(|data| CommodityType(data.id.as_str()))
            .collect()
    }

    fn data(&self) -> &'static CommodityData {
        commodity_data()
            .iter()
            .find(|data| data.id == self.0)
            .unwrap_or_else(|| panic!("commodity '{}' is not in data/commodities.json", self.0))
    }

    pub fn id(&self) -> &'static str {
        self.0
    }

    /// Get the base price for this commodity
    pub fn base_price(&self) -> u32 {
        self.data().base_price
    }

    /// How far a price sits from this commodity's base price, in whole percent
    pub fn percent_vs_base(&self, price: u32) -> i32 {
        let base = f64::from(self.base_price());
        ((f64::from(price) / base - 1.0) * 100.0).round() as i32
    }

    /// Get the name as a string
    pub fn name(&self) -> &'static str {
        &self.data().name
    }

    /// Check if this commodity is illegal
    pub fn is_illegal(&self) -> bool {
        self.data().illegal
    }

    /// Get economy preference multiplier for this commodity
    pub fn economy_preference(&self, economy: &EconomyType) -> f64 {
        let data = self.data();
        data.economy_multipliers
            .get(economy)
            .copied()
            .unwrap_or(data.default_multiplier)
    }
}

/// Represents a commodity with quantity
#[derive(Debug, Clone)]
pub struct CargoItem {
    pub commodity: CommodityType,
    pub quantity: u32,
}

impl CargoItem {
    pub fn new(commodity: CommodityType, quantity: u32) -> Self {
        Self {
            commodity,
            quantity,
        }
    }

    /// Calculate the value of this cargo item at given price
    #[allow(dead_code)]
    pub fn value_at_price(&self, price_per_ton: u32) -> u32 {
        self.quantity * price_per_ton
    }
}

/// Represents the cargo hold of a ship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CargoHold {
    items: HashMap<CommodityType, u32>,
    pub capacity: u32,
    /// Tons held for contracts: they take up room but are not goods and cannot be sold
    #[serde(default)]
    pub reserved: u32,
}

impl CargoHold {
    pub fn new(capacity: u32) -> Self {
        Self {
            items: HashMap::new(),
            capacity,
            reserved: 0,
        }
    }

    /// Weight of the goods that can be sold
    pub fn goods_weight(&self) -> u32 {
        self.items.values().sum()
    }

    /// Get current cargo weight, goods and contract cargo together
    pub fn current_weight(&self) -> u32 {
        self.goods_weight() + self.reserved
    }

    /// Hold room for contract cargo; false, changing nothing, if it does not fit
    pub fn reserve(&mut self, tons: u32) -> bool {
        if !self.can_add(tons) {
            return false;
        }
        self.reserved += tons;
        true
    }

    /// Give up to `tons` of contract cargo back and return how much was released
    pub fn release(&mut self, tons: u32) -> u32 {
        let released = tons.min(self.reserved);
        self.reserved -= released;
        released
    }

    /// Get remaining capacity
    pub fn remaining_capacity(&self) -> u32 {
        self.capacity.saturating_sub(self.current_weight())
    }

    /// Check if we can add the specified quantity
    pub fn can_add(&self, quantity: u32) -> bool {
        self.current_weight() + quantity <= self.capacity
    }

    /// Add cargo to the hold
    pub fn add_cargo(&mut self, commodity: CommodityType, quantity: u32) -> bool {
        if !self.can_add(quantity) {
            return false;
        }

        *self.items.entry(commodity).or_insert(0) += quantity;
        true
    }

    /// Remove cargo from the hold
    pub fn remove_cargo(&mut self, commodity: &CommodityType, quantity: u32) -> bool {
        if let Some(current_quantity) = self.items.get_mut(commodity) {
            if *current_quantity >= quantity {
                *current_quantity -= quantity;
                if *current_quantity == 0 {
                    self.items.remove(commodity);
                }
                return true;
            }
        }
        false
    }

    /// Get quantity of a specific commodity
    pub fn get_quantity(&self, commodity: &CommodityType) -> u32 {
        self.items.get(commodity).copied().unwrap_or(0)
    }

    /// Get all cargo items
    pub fn get_items(&self) -> Vec<CargoItem> {
        self.items
            .iter()
            .map(|(commodity, &quantity)| CargoItem::new(commodity.clone(), quantity))
            .collect()
    }

    /// Check if cargo hold is empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

/// Represents market prices for commodities
#[derive(Debug, Clone)]
pub struct MarketPrices {
    pub buy_prices: HashMap<CommodityType, u32>,
    pub sell_prices: HashMap<CommodityType, u32>,
    pub availability: HashMap<CommodityType, u32>,
}

impl MarketPrices {
    pub fn new() -> Self {
        Self {
            buy_prices: HashMap::new(),
            sell_prices: HashMap::new(),
            availability: HashMap::new(),
        }
    }

    /// Get buy price for a commodity
    pub fn get_buy_price(&self, commodity: &CommodityType) -> Option<u32> {
        self.buy_prices.get(commodity).copied()
    }

    /// Get sell price for a commodity
    pub fn get_sell_price(&self, commodity: &CommodityType) -> Option<u32> {
        self.sell_prices.get(commodity).copied()
    }

    /// Get availability for a commodity
    pub fn get_availability(&self, commodity: &CommodityType) -> Option<u32> {
        self.availability.get(commodity).copied()
    }
}

impl Default for MarketPrices {
    fn default() -> Self {
        Self::new()
    }
}

/// Manages the trading system and market generation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TradingSystem {
    commodities: Vec<CommodityType>,
}

impl TradingSystem {
    pub fn new() -> Self {
        Self {
            commodities: CommodityType::all(),
        }
    }

    /// Generate market prices for a system based on its economy
    pub fn generate_market_prices(&self, economy: &EconomyType, seed: u64) -> MarketPrices {
        let mut rng = StdRng::seed_from_u64(seed);
        let mut market = MarketPrices::new();

        for commodity in &self.commodities {
            let base_price = commodity.base_price();
            let economy_multiplier = commodity.economy_preference(economy);

            // Add ±30% random fluctuation
            let fluctuation = 0.7 + (rng.gen::<f64>() * 0.6); // 0.7 to 1.3

            let adjusted_price = (base_price as f64 * economy_multiplier * fluctuation) as u32;

            // Buy/sell spread (2-6%)
            let spread = 0.94 + (rng.gen::<f64>() * 0.04); // 0.94 to 0.98

            let buy_price = adjusted_price;
            let sell_price = (adjusted_price as f64 * spread) as u32;

            // Availability based on economy preference
            let availability = if economy_multiplier > 1.0 {
                50 + (rng.gen::<u32>() % 50) // 50-100 tons
            } else if economy_multiplier < 1.0 {
                5 + (rng.gen::<u32>() % 15) // 5-20 tons
            } else {
                20 + (rng.gen::<u32>() % 30) // 20-50 tons
            };

            market.buy_prices.insert(commodity.clone(), buy_price);
            market.sell_prices.insert(commodity.clone(), sell_price);
            market.availability.insert(commodity.clone(), availability);
        }

        market
    }

    /// Get all available commodities
    pub fn get_commodities(&self) -> &[CommodityType] {
        &self.commodities
    }
}

impl Default for TradingSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
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
        let systems =
            crate::data::systems::SystemManager::new().expect("Failed to load galaxy data");
        let from = systems.get_system("LAVE").unwrap().economy.clone();
        let to = systems.get_system("DISO").unwrap().economy.clone();
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
        let systems =
            crate::data::systems::SystemManager::new().expect("Failed to load galaxy data");
        let trading = TradingSystem::new();
        let pairs = [
            ("LAVE", "DISO"),
            ("DISO", "LAVE"),
            ("LAVE", "LEESTI"),
            ("LEESTI", "DISO"),
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
}
