use serde::{Deserialize, Serialize};

pub const STARTING_CREDITS: u32 = 100;
pub const MIN_REPUTATION: i32 = -100;
pub const MAX_REPUTATION: i32 = 100;

/// How the galaxy sees the commander, derived from the reputation score
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
}

#[allow(dead_code)]
impl Commander {
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
            location: "LAVE".to_string(), // Starting location as per spec
            reputation: 0,
        }
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
        assert_eq!(commander.location(), "LAVE");
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
        commander.set_location("Leesti".to_string());
        assert_eq!(commander.location(), "Leesti");
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
        let json = r#"{"name":"Old","credits":1,"location":"LAVE","reputation":250}"#;
        let commander: Commander = serde_json::from_str(json).unwrap();
        assert_eq!(commander.reputation, 100);

        let json = r#"{"name":"Old","credits":1,"location":"LAVE","reputation":-250}"#;
        let commander: Commander = serde_json::from_str(json).unwrap();
        assert_eq!(commander.reputation, -100);
    }

    #[test]
    fn reputation_defaults_to_zero_for_old_saves() {
        let commander: Commander =
            serde_json::from_str(r#"{"name":"Old","credits":100,"location":"LAVE"}"#).unwrap();
        assert_eq!(commander.reputation, 0);
    }
}
