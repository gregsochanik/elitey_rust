use crate::game::frontier::Discovery;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const STARTING_CREDITS: u32 = 100;
pub const MIN_REPUTATION: i32 = -100;
pub const MAX_REPUTATION: i32 = 100;

/// How the galaxy sees the commander, derived from the reputation score
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Standing {
    Criminal,
    Disreputable,
    Neutral,
    Trusted,
    Respected,
}

impl Standing {
    pub fn from_score(score: i32) -> Self {
        match score {
            i32::MIN..=-30 => Standing::Criminal,
            -29..=-10 => Standing::Disreputable,
            -9..=9 => Standing::Neutral,
            10..=29 => Standing::Trusted,
            _ => Standing::Respected,
        }
    }

    /// Percentage points a trade moves in the player's favour (negative when against)
    pub fn price_favour_percent(&self) -> i64 {
        match self {
            Standing::Criminal => -4,
            Standing::Disreputable => -2,
            Standing::Neutral => 0,
            Standing::Trusted => 2,
            Standing::Respected => 4,
        }
    }

    /// Price the player pays: cheaper with good standing
    pub fn buy_price(&self, price: u32) -> u32 {
        Self::scale(price, 100 - self.price_favour_percent())
    }

    /// Price the player receives: higher with good standing
    pub fn sell_price(&self, price: u32) -> u32 {
        Self::scale(price, 100 + self.price_favour_percent())
    }

    fn scale(price: u32, percent: i64) -> u32 {
        ((i64::from(price) * percent + 50) / 100).clamp(1, i64::from(u32::MAX)) as u32
    }

    pub fn name(&self) -> &'static str {
        match self {
            Standing::Criminal => "Criminal",
            Standing::Disreputable => "Disreputable",
            Standing::Neutral => "Neutral",
            Standing::Trusted => "Trusted",
            Standing::Respected => "Respected",
        }
    }
}

/// How dangerous the commander is, from the number of ships destroyed
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CombatRating {
    #[default]
    Harmless,
    MostlyHarmless,
    Poor,
    Average,
    AboveAverage,
    Competent,
    Dangerous,
    Deadly,
    Elite,
}

impl CombatRating {
    const LADDER: [(u32, CombatRating); 9] = [
        (0, CombatRating::Harmless),
        (3, CombatRating::MostlyHarmless),
        (6, CombatRating::Poor),
        (12, CombatRating::Average),
        (25, CombatRating::AboveAverage),
        (50, CombatRating::Competent),
        (100, CombatRating::Dangerous),
        (200, CombatRating::Deadly),
        (400, CombatRating::Elite),
    ];

    pub fn from_kills(kills: u32) -> Self {
        Self::LADDER
            .iter()
            .rev()
            .find(|(needed, _)| kills >= *needed)
            .map(|(_, rating)| *rating)
            .unwrap_or(CombatRating::Harmless)
    }

    /// The next rating and the kills it needs, or `None` at ELITE
    pub fn next(&self) -> Option<(CombatRating, u32)> {
        let index = Self::LADDER.iter().position(|(_, rating)| rating == self)?;
        Self::LADDER
            .get(index + 1)
            .map(|(needed, rating)| (*rating, *needed))
    }

    pub fn name(&self) -> &'static str {
        match self {
            CombatRating::Harmless => "Harmless",
            CombatRating::MostlyHarmless => "Mostly Harmless",
            CombatRating::Poor => "Poor",
            CombatRating::Average => "Average",
            CombatRating::AboveAverage => "Above Average",
            CombatRating::Competent => "Competent",
            CombatRating::Dangerous => "Dangerous",
            CombatRating::Deadly => "Deadly",
            CombatRating::Elite => "ELITE",
        }
    }
}

/// What a reputation change actually did once limits were applied
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReputationChange {
    pub applied: i32,
    pub new_standing: Option<Standing>,
}

fn clamped_reputation<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<i32, D::Error> {
    Ok(i32::deserialize(deserializer)?.clamp(MIN_REPUTATION, MAX_REPUTATION))
}

/// Represents the player commander with all their data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commander {
    /// Commander's name
    pub name: String,
    /// Current credits
    pub credits: u32,
    /// Current location (system name)
    pub location: String,
    /// Reputation score, kept between MIN_REPUTATION and MAX_REPUTATION
    #[serde(default, deserialize_with = "clamped_reputation")]
    pub reputation: i32,
    /// Enemy ships destroyed
    #[serde(default)]
    pub kills: u32,
    /// Systems the commander has been to
    #[serde(default)]
    pub visited: BTreeSet<String>,
    /// What the commander has found at the frontier
    #[serde(default)]
    pub discoveries: Vec<Discovery>,
}

impl Commander {
    pub fn combat_rating(&self) -> CombatRating {
        CombatRating::from_kills(self.kills)
    }

    /// Count a destroyed enemy; returns the new rating when it earns a promotion
    pub fn record_kill(&mut self) -> Option<CombatRating> {
        let before = self.combat_rating();
        self.kills = self.kills.saturating_add(1);
        let after = self.combat_rating();
        (after != before).then_some(after)
    }

    pub fn standing(&self) -> Standing {
        Standing::from_score(self.reputation)
    }

    /// Change reputation within its limits
    pub fn change_reputation(&mut self, delta: i32) -> ReputationChange {
        let before = self.reputation;
        let standing_before = self.standing();
        self.reputation = before
            .saturating_add(delta)
            .clamp(MIN_REPUTATION, MAX_REPUTATION);
        let standing_after = self.standing();
        ReputationChange {
            applied: self.reputation - before,
            new_standing: (standing_after != standing_before).then_some(standing_after),
        }
    }

    /// Create a new commander with starting conditions
    pub fn new(name: String) -> Self {
        Self {
            name,
            credits: STARTING_CREDITS,
            location: "Sol".to_string(), // Starting location as per spec
            reputation: 0,
            kills: 0,
            visited: BTreeSet::from(["Sol".to_string()]),
            discoveries: Vec::new(),
        }
    }

    /// Note that the commander has been to a system
    pub fn visit(&mut self, system: &str) {
        if !self.visited.contains(system) {
            self.visited.insert(system.to_string());
        }
    }

    /// Note the system the commander is in, for saves made before visits were kept
    pub fn visit_here(&mut self) {
        let here = self.location.clone();
        self.visit(&here);
    }

    pub fn has_visited(&self, system: &str) -> bool {
        self.visited.contains(system)
    }

    /// Get the commander's name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get current credits
    pub fn credits(&self) -> u32 {
        self.credits
    }

    /// Get current location
    pub fn location(&self) -> &str {
        &self.location
    }

    /// Add credits to the commander's account
    pub fn add_credits(&mut self, amount: u32) {
        self.credits += amount;
    }

    /// Remove credits from the commander's account
    /// Returns true if successful, false if insufficient funds
    pub fn remove_credits(&mut self, amount: u32) -> bool {
        if self.credits >= amount {
            self.credits -= amount;
            true
        } else {
            false
        }
    }

    /// Check if commander has sufficient credits
    pub fn has_credits(&self, amount: u32) -> bool {
        self.credits >= amount
    }

    /// Update the commander's location
    pub fn set_location(&mut self, location: String) {
        self.location = location;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commander_creation() {
        let commander = Commander::new("Test Commander".to_string());
        assert_eq!(commander.name(), "Test Commander");
        assert_eq!(commander.credits(), 100);
        assert_eq!(commander.location(), "Sol");
    }

    #[test]
    fn test_add_credits() {
        let mut commander = Commander::new("Test".to_string());
        commander.add_credits(50);
        assert_eq!(commander.credits(), 150);
    }

    #[test]
    fn test_remove_credits_success() {
        let mut commander = Commander::new("Test".to_string());
        let result = commander.remove_credits(30);
        assert!(result);
        assert_eq!(commander.credits(), 70);
    }

    #[test]
    fn test_remove_credits_insufficient() {
        let mut commander = Commander::new("Test".to_string());
        let result = commander.remove_credits(150);
        assert!(!result);
        assert_eq!(commander.credits(), 100); // Credits unchanged
    }

    #[test]
    fn test_has_credits() {
        let commander = Commander::new("Test".to_string());
        assert!(commander.has_credits(50));
        assert!(commander.has_credits(100));
        assert!(!commander.has_credits(150));
    }

    #[test]
    fn test_set_location() {
        let mut commander = Commander::new("Test".to_string());
        commander.set_location("Alpha Centauri".to_string());
        assert_eq!(commander.location(), "Alpha Centauri");
    }

    #[test]
    fn standing_boundaries() {
        let cases = [
            (-100, Standing::Criminal),
            (-30, Standing::Criminal),
            (-29, Standing::Disreputable),
            (-10, Standing::Disreputable),
            (-9, Standing::Neutral),
            (9, Standing::Neutral),
            (10, Standing::Trusted),
            (29, Standing::Trusted),
            (30, Standing::Respected),
            (100, Standing::Respected),
        ];
        for (score, expected) in cases {
            assert_eq!(Standing::from_score(score), expected, "score {}", score);
        }
    }

    #[test]
    fn good_standing_buys_cheaper_and_sells_higher() {
        assert_eq!(Standing::Neutral.buy_price(100), 100);
        assert_eq!(Standing::Neutral.sell_price(100), 100);
        assert_eq!(Standing::Respected.buy_price(100), 96);
        assert_eq!(Standing::Respected.sell_price(100), 104);
        assert_eq!(Standing::Trusted.buy_price(100), 98);
        assert_eq!(Standing::Trusted.sell_price(100), 102);
        assert_eq!(Standing::Disreputable.buy_price(100), 102);
        assert_eq!(Standing::Disreputable.sell_price(100), 98);
        assert_eq!(Standing::Criminal.buy_price(100), 104);
        assert_eq!(Standing::Criminal.sell_price(100), 96);
    }

    #[test]
    fn price_adjustment_saturates_instead_of_wrapping() {
        assert_eq!(Standing::Respected.sell_price(u32::MAX), u32::MAX);
        assert_eq!(Standing::Respected.sell_price(4_129_776_246), u32::MAX);
    }

    #[test]
    fn price_adjustment_rounds_half_up_and_never_reaches_zero() {
        assert_eq!(Standing::Respected.sell_price(13), 14); // 13.52
        assert_eq!(Standing::Trusted.sell_price(25), 26); // 25.5
        assert_eq!(Standing::Trusted.buy_price(25), 25); // 24.5
        assert_eq!(Standing::Respected.buy_price(1), 1);
    }

    #[test]
    fn reputation_clamps_at_both_ends() {
        let mut commander = Commander::new("Test".to_string());
        assert_eq!(commander.change_reputation(500).applied, 100);
        assert_eq!(commander.reputation, 100);
        assert_eq!(commander.change_reputation(5).applied, 0);
        assert_eq!(commander.change_reputation(i32::MAX).applied, 0);
        assert_eq!(commander.change_reputation(-500).applied, -200);
        assert_eq!(commander.reputation, -100);
        assert_eq!(commander.change_reputation(i32::MIN).applied, 0);
        assert_eq!(commander.reputation, -100);
    }

    #[test]
    fn change_reputation_reports_only_standing_changes() {
        let mut commander = Commander::new("Test".to_string());
        assert_eq!(commander.change_reputation(5).new_standing, None);
        assert_eq!(
            commander.change_reputation(5).new_standing,
            Some(Standing::Trusted)
        );
        assert_eq!(commander.change_reputation(5).new_standing, None);
        assert_eq!(
            commander.change_reputation(-15).new_standing,
            Some(Standing::Neutral)
        );
    }

    #[test]
    fn out_of_range_saved_reputation_is_clamped_on_load() {
        let json = r#"{"name":"Old","credits":1,"location":"Sol","reputation":250}"#;
        let commander: Commander = serde_json::from_str(json).unwrap();
        assert_eq!(commander.reputation, 100);

        let json = r#"{"name":"Old","credits":1,"location":"Sol","reputation":-250}"#;
        let commander: Commander = serde_json::from_str(json).unwrap();
        assert_eq!(commander.reputation, -100);
    }

    #[test]
    fn reputation_defaults_to_zero_for_old_saves() {
        let commander: Commander =
            serde_json::from_str(r#"{"name":"Old","credits":100,"location":"Sol"}"#).unwrap();
        assert_eq!(commander.reputation, 0);
    }

    #[test]
    fn kills_map_to_the_rating_ladder() {
        let expected = [
            (0, CombatRating::Harmless),
            (2, CombatRating::Harmless),
            (3, CombatRating::MostlyHarmless),
            (6, CombatRating::Poor),
            (12, CombatRating::Average),
            (25, CombatRating::AboveAverage),
            (50, CombatRating::Competent),
            (100, CombatRating::Dangerous),
            (200, CombatRating::Deadly),
            (399, CombatRating::Deadly),
            (400, CombatRating::Elite),
            (5_000, CombatRating::Elite),
        ];
        for (kills, rating) in expected {
            assert_eq!(CombatRating::from_kills(kills), rating, "{kills} kills");
        }
    }

    #[test]
    fn the_next_rating_and_its_kills_are_known_until_elite() {
        assert_eq!(
            CombatRating::Harmless.next(),
            Some((CombatRating::MostlyHarmless, 3))
        );
        assert_eq!(
            CombatRating::Deadly.next(),
            Some((CombatRating::Elite, 400))
        );
        assert_eq!(CombatRating::Elite.next(), None);
        assert_eq!(CombatRating::Elite.name(), "ELITE");
    }

    #[test]
    fn a_kill_reports_only_the_promotions() {
        let mut commander = Commander::new("Test".to_string());

        assert_eq!(commander.record_kill(), None);
        assert_eq!(commander.record_kill(), None);
        assert_eq!(commander.record_kill(), Some(CombatRating::MostlyHarmless));
        assert_eq!(commander.kills, 3);
        assert_eq!(commander.combat_rating(), CombatRating::MostlyHarmless);
    }

    #[test]
    fn old_commanders_start_with_no_kills() {
        let commander: Commander =
            serde_json::from_str(r#"{"name":"Old","credits":5,"location":"Sol"}"#).unwrap();

        assert_eq!(commander.kills, 0);
    }
}
