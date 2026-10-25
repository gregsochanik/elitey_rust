use serde::Deserialize;
use std::sync::OnceLock;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WeaponSpec {
    pub damage: u32,
    pub accuracy: f64,
}

/// How an enemy chooses its action each round. Hull thresholds are fractions of full hull;
/// a threshold of 0 never triggers. `aggression` (0 to 1) lowers the flee thresholds, so
/// 1 fights on.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Behaviour {
    pub aggression: f64,
    pub flee_below_hull: f64,
    pub flee_odds: f64,
    pub flee_chance: f64,
    pub surrender_below_hull: f64,
    pub surrender_chance: f64,
}

impl Behaviour {
    /// Fights to the end
    pub const NEVER_RETREATS: Behaviour = Behaviour {
        aggression: 1.0,
        flee_below_hull: 0.0,
        flee_odds: 0.0,
        flee_chance: 0.5,
        surrender_below_hull: 0.0,
        surrender_chance: 0.5,
    };
}

/// One kind of enemy ship, as described in `data/enemies.json`
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnemyClass {
    pub id: String,
    pub name: String,
    pub hull: u32,
    pub shield: u32,
    pub recharge: u32,
    pub weapons: Vec<WeaponSpec>,
    pub behaviour: Behaviour,
}

pub fn enemy_classes() -> &'static [EnemyClass] {
    static ENEMIES: OnceLock<Vec<EnemyClass>> = OnceLock::new();
    ENEMIES.get_or_init(|| {
        serde_json::from_str(include_str!("../../data/enemies.json"))
            .expect("data/enemies.json is valid")
    })
}

pub fn find_enemy(id: &str) -> Option<&'static EnemyClass> {
    enemy_classes().iter().find(|enemy| enemy.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_shipped_enemies_load_with_valid_behaviour() {
        assert!(find_enemy("pirate-raider").is_some());
        assert!(find_enemy("patrol-vessel").is_some());
        for enemy in enemy_classes() {
            let b = enemy.behaviour;
            for value in [
                b.aggression,
                b.flee_below_hull,
                b.flee_chance,
                b.surrender_below_hull,
                b.surrender_chance,
            ] {
                assert!((0.0..=1.0).contains(&value), "{}", enemy.id);
            }
            assert!(b.flee_odds >= 0.0, "{}", enemy.id);
            assert!(enemy.hull > 0, "{}", enemy.id);
        }
    }
}
