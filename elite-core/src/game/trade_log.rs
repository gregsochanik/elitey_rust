use crate::data::commodities::CommodityType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TradeKind {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub kind: TradeKind,
    pub commodity: CommodityType,
    pub quantity: u32,
    pub unit_price: u32,
    pub location: String,
    pub timestamp: u64,
    /// Set on sells only
    pub profit: Option<i64>,
}

impl Transaction {
    pub fn total(&self) -> u32 {
        self.unit_price * self.quantity
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct Holding {
    quantity: u32,
    total_cost: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TradeStats {
    pub total_trades: usize,
    pub net_profit: i64,
    pub success_rate: Option<f64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TradeLog {
    transactions: Vec<Transaction>,
    holdings: HashMap<CommodityType, Holding>,
}

impl TradeLog {
    pub fn new() -> Self {
        Self::default()
    }

    #[cfg(test)]
    pub fn transactions(&self) -> &[Transaction] {
        &self.transactions
    }

    /// Record a buy. `cargo_before` is the quantity of this commodity in the hold before the purchase.
    pub fn record_buy(
        &mut self,
        commodity: CommodityType,
        quantity: u32,
        unit_price: u32,
        location: &str,
        cargo_before: u32,
    ) {
        self.sync_holding(&commodity, cargo_before);
        let holding = self.holdings.entry(commodity.clone()).or_default();
        holding.quantity += quantity;
        holding.total_cost += f64::from(unit_price) * f64::from(quantity);

        self.transactions.push(Transaction {
            kind: TradeKind::Buy,
            commodity,
            quantity,
            unit_price,
            location: location.to_string(),
            timestamp: now(),
            profit: None,
        });
    }

    /// Record a sell. `cargo_before` is the quantity of this commodity in the hold before the sale.
    /// Returns the profit against average cost.
    pub fn record_sell(
        &mut self,
        commodity: CommodityType,
        quantity: u32,
        unit_price: u32,
        location: &str,
        cargo_before: u32,
    ) -> i64 {
        self.sync_holding(&commodity, cargo_before);
        let holding = self.holdings.entry(commodity.clone()).or_default();
        let average_cost = if holding.quantity == 0 {
            0.0
        } else {
            holding.total_cost / f64::from(holding.quantity)
        };
        let cost_basis = (average_cost * f64::from(quantity)).round() as i64;
        let profit = i64::from(unit_price) * i64::from(quantity) - cost_basis;

        holding.quantity = holding.quantity.saturating_sub(quantity);
        holding.total_cost = average_cost * f64::from(holding.quantity);

        self.transactions.push(Transaction {
            kind: TradeKind::Sell,
            commodity,
            quantity,
            unit_price,
            location: location.to_string(),
            timestamp: now(),
            profit: Some(profit),
        });
        profit
    }

    /// Newest first
    pub fn recent(&self, count: usize) -> Vec<&Transaction> {
        self.transactions.iter().rev().take(count).collect()
    }

    pub fn stats(&self) -> TradeStats {
        let profits: Vec<i64> = self.transactions.iter().filter_map(|t| t.profit).collect();
        let success_rate = if profits.is_empty() {
            None
        } else {
            let wins = profits.iter().filter(|p| **p > 0).count();
            Some(wins as f64 / profits.len() as f64)
        };
        TradeStats {
            total_trades: self.transactions.len(),
            net_profit: profits.iter().sum(),
            success_rate,
        }
    }

    pub fn best_trade(&self) -> Option<&Transaction> {
        self.transactions
            .iter()
            .filter(|t| t.profit.is_some_and(|p| p > 0))
            .max_by_key(|t| t.profit)
    }

    /// What each ton of `commodity` in the hold cost on average, or `None` when none was
    /// bought (cargo from loot or contracts costs nothing and dilutes the average)
    pub fn average_cost(&self, commodity: &CommodityType, held: u32) -> Option<u32> {
        let holding = self.holdings.get(commodity)?;
        let tons = holding.quantity.max(held);
        if held == 0 || tons == 0 || holding.total_cost <= 0.0 {
            return None;
        }
        Some((holding.total_cost / f64::from(tons)).round() as u32)
    }

    /// Align tracked holdings with the real hold: cargo gained outside trading dilutes
    /// average cost (cost 0); cargo lost leaves average cost unchanged.
    fn sync_holding(&mut self, commodity: &CommodityType, actual_quantity: u32) {
        let holding = self.holdings.entry(commodity.clone()).or_default();
        let denominator = holding.quantity.max(actual_quantity);
        let average_cost = if denominator == 0 {
            0.0
        } else {
            holding.total_cost / f64::from(denominator)
        };
        holding.quantity = actual_quantity;
        holding.total_cost = average_cost * f64::from(actual_quantity);
    }
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    const FOOD: CommodityType = CommodityType::Food;

    #[test]
    fn buy_records_a_transaction() {
        let mut log = TradeLog::new();
        log.record_buy(FOOD, 5, 15, "Sol", 0);

        let t = &log.transactions()[0];
        assert_eq!(t.kind, TradeKind::Buy);
        assert_eq!((t.quantity, t.unit_price, t.total()), (5, 15, 75));
        assert_eq!(t.location, "Sol");
        assert_eq!(t.profit, None);
    }

    #[test]
    fn sell_records_profit_against_cost() {
        let mut log = TradeLog::new();
        log.record_buy(FOOD, 5, 15, "Sol", 0);
        let profit = log.record_sell(FOOD, 5, 18, "Alpha Centauri", 5);

        assert_eq!(profit, 15);
        assert_eq!(log.transactions()[1].profit, Some(15));
    }

    #[test]
    fn profit_uses_average_cost_across_buys() {
        let mut log = TradeLog::new();
        log.record_buy(FOOD, 5, 10, "Sol", 0);
        log.record_buy(FOOD, 5, 20, "Sol", 5);
        // average cost 15; sell 10 at 20 -> +50
        assert_eq!(log.record_sell(FOOD, 10, 20, "Alpha Centauri", 10), 50);
    }

    #[test]
    fn partial_sell_keeps_average_cost() {
        let mut log = TradeLog::new();
        log.record_buy(FOOD, 4, 10, "Sol", 0);
        assert_eq!(log.record_sell(FOOD, 2, 15, "Alpha Centauri", 4), 10);
        assert_eq!(log.record_sell(FOOD, 2, 15, "Alpha Centauri", 2), 10);
    }

    #[test]
    fn loss_gives_negative_profit_and_lowers_success_rate() {
        let mut log = TradeLog::new();
        log.record_buy(FOOD, 2, 20, "Sol", 0);
        assert_eq!(log.record_sell(FOOD, 1, 25, "Alpha Centauri", 2), 5);
        assert_eq!(log.record_sell(FOOD, 1, 15, "Alpha Centauri", 1), -5);

        let stats = log.stats();
        assert_eq!(stats.net_profit, 0);
        assert_eq!(stats.success_rate, Some(0.5));
    }

    #[test]
    fn cargo_gained_outside_trading_has_zero_cost() {
        let mut log = TradeLog::new();
        log.record_buy(FOOD, 5, 10, "Sol", 0);
        // an event added 5 tons: 10 held, average cost 5
        assert_eq!(log.record_sell(FOOD, 10, 10, "Alpha Centauri", 10), 50);
    }

    #[test]
    fn cargo_lost_outside_trading_keeps_average_cost() {
        let mut log = TradeLog::new();
        log.record_buy(FOOD, 10, 10, "Sol", 0);
        // an event removed 5 tons: 5 held, average cost still 10
        assert_eq!(log.record_sell(FOOD, 5, 12, "Alpha Centauri", 5), 10);
    }

    #[test]
    fn selling_untracked_cargo_is_all_profit() {
        let mut log = TradeLog::new();
        assert_eq!(log.record_sell(FOOD, 3, 10, "Sol", 3), 30);
    }

    #[test]
    fn best_trade_ignores_losses() {
        let mut log = TradeLog::new();
        log.record_buy(FOOD, 2, 20, "Sol", 0);
        log.record_sell(FOOD, 2, 15, "Alpha Centauri", 2);
        assert!(log.best_trade().is_none());
    }

    #[test]
    fn empty_log_stats() {
        let log = TradeLog::new();
        assert_eq!(
            log.stats(),
            TradeStats {
                total_trades: 0,
                net_profit: 0,
                success_rate: None
            }
        );
        assert!(log.best_trade().is_none());
        assert!(log.recent(10).is_empty());
    }

    #[test]
    fn stats_count_buys_and_sells_and_find_best_trade() {
        let mut log = TradeLog::new();
        log.record_buy(FOOD, 5, 10, "Sol", 0);
        log.record_sell(FOOD, 2, 20, "Alpha Centauri", 5);
        log.record_sell(FOOD, 3, 12, "Alpha Centauri", 3);

        let stats = log.stats();
        assert_eq!(stats.total_trades, 3);
        assert_eq!(stats.net_profit, 20 + 6);
        assert_eq!(stats.success_rate, Some(1.0));
        assert_eq!(log.best_trade().unwrap().profit, Some(20));
    }

    #[test]
    fn recent_is_newest_first_and_limited() {
        let mut log = TradeLog::new();
        log.record_buy(FOOD, 1, 10, "A", 0);
        log.record_buy(FOOD, 2, 10, "B", 1);
        log.record_buy(FOOD, 3, 10, "C", 3);

        let recent = log.recent(2);
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].location, "C");
        assert_eq!(recent[1].location, "B");
    }

    #[test]
    fn survives_serialization_round_trip() {
        let mut log = TradeLog::new();
        log.record_buy(FOOD, 5, 10, "Sol", 0);
        let json = serde_json::to_string(&log).unwrap();
        let mut restored: TradeLog = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.transactions().len(), 1);
        assert_eq!(restored.record_sell(FOOD, 5, 12, "Alpha Centauri", 5), 10);
    }

    #[test]
    fn average_cost_blends_purchases_and_ignores_free_cargo() {
        let mut log = TradeLog::new();
        log.record_buy(CommodityType::Food, 2, 20, "Sol", 0);
        log.record_buy(CommodityType::Food, 2, 30, "Sol", 2);

        assert_eq!(log.average_cost(&CommodityType::Food, 4), Some(25));
        assert_eq!(log.average_cost(&CommodityType::Food, 0), None);
        assert_eq!(log.average_cost(&CommodityType::Textiles, 3), None);
        // two extra free tons dilute the average
        assert_eq!(log.average_cost(&CommodityType::Food, 6), Some(17));
    }
}
