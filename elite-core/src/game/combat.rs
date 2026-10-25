use crate::data::enemies::{find_enemy, EnemyClass};
use crate::data::equipment::{CombatStats, EquipmentCategory};
use crate::game::Ship;

/// Chance of escaping when fleeing
const FLEE_CHANCE: f64 = 0.5;

#[derive(Debug, Clone, PartialEq)]
pub struct Weapon {
    pub damage: u32,
    pub accuracy: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Combatant {
    pub name: String,
    pub hull: u32,
    pub max_hull: u32,
    pub shield: u32,
    pub max_shield: u32,
    pub recharge: u32,
    pub weapons: Vec<Weapon>,
}

impl Combatant {
    /// The player's fighter, built from the ship's hull and installed equipment
    pub fn from_ship(ship: &Ship) -> Self {
        let mut shield = CombatStats::NONE;
        let mut weapons = Vec::new();
        for installed in ship.equipment.installed() {
            let item = installed.item();
            match item.category {
                EquipmentCategory::Weapon => weapons.push(Weapon {
                    damage: item.combat.damage,
                    accuracy: item.combat.accuracy,
                }),
                EquipmentCategory::Shield => shield = item.combat,
                _ => {}
            }
        }

        Self {
            name: "Your ship".to_string(),
            hull: ship.hull_percentage,
            max_hull: ship.ship_type.max_hull(),
            shield: shield.protection,
            max_shield: shield.protection,
            recharge: shield.recharge,
            weapons,
        }
    }

    /// An enemy built from its entry in `data/enemies.json`
    pub fn from_enemy(enemy: &EnemyClass) -> Self {
        Self {
            name: enemy.name.clone(),
            hull: enemy.hull,
            max_hull: enemy.hull,
            shield: enemy.shield,
            max_shield: enemy.shield,
            recharge: enemy.recharge,
            weapons: enemy
                .weapons
                .iter()
                .map(|weapon| Weapon {
                    damage: weapon.damage,
                    accuracy: weapon.accuracy,
                })
                .collect(),
        }
    }

    fn named_enemy(id: &str) -> Self {
        Self::from_enemy(find_enemy(id).expect("enemy is in data/enemies.json"))
    }

    pub fn pirate_raider() -> Self {
        Self::named_enemy("pirate-raider")
    }

    pub fn patrol_vessel() -> Self {
        Self::named_enemy("patrol-vessel")
    }

    pub fn is_destroyed(&self) -> bool {
        self.hull == 0
    }

    /// Damage hits the shield first and carries over to the hull
    fn take_damage(&mut self, amount: u32) {
        let absorbed = self.shield.min(amount);
        self.shield -= absorbed;
        self.hull = self.hull.saturating_sub(amount - absorbed);
    }

    fn recharge_shield(&mut self) {
        self.shield = (self.shield + self.recharge).min(self.max_shield);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Attack,
    Flee,
    Surrender,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Ongoing,
    Victory,
    Defeat,
    Escaped,
    Surrendered,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatError {
    NoWeapons,
    Finished,
}

impl std::fmt::Display for CombatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CombatError::NoWeapons => write!(f, "You have no weapons installed"),
            CombatError::Finished => write!(f, "The fight is over"),
        }
    }
}

impl std::error::Error for CombatError {}

#[derive(Debug, Clone)]
pub struct Combat {
    pub player: Combatant,
    pub enemy: Combatant,
    pub log: Vec<String>,
    outcome: Outcome,
}

impl Combat {
    pub fn new(player: Combatant, enemy: Combatant) -> Self {
        Self {
            player,
            enemy,
            log: Vec::new(),
            outcome: Outcome::Ongoing,
        }
    }

    pub fn outcome(&self) -> Outcome {
        self.outcome
    }

    /// Play one round. `roll` returns a number in 0.0..1.0 and is called once per shot
    /// (player weapons first, then the enemy's) and once for a flee attempt.
    /// A refused action takes no turn and consumes no rolls.
    pub fn resolve_round(
        &mut self,
        action: Action,
        roll: &mut impl FnMut() -> f64,
    ) -> Result<Outcome, CombatError> {
        if self.outcome != Outcome::Ongoing {
            return Err(CombatError::Finished);
        }

        match action {
            Action::Attack => {
                if self.player.weapons.is_empty() {
                    return Err(CombatError::NoWeapons);
                }
                fire(&self.player, &mut self.enemy, roll, &mut self.log);
                if self.enemy.is_destroyed() {
                    self.log
                        .push(format!("The {} is destroyed!", self.enemy.name));
                    return Ok(self.finish(Outcome::Victory));
                }
            }
            Action::Flee => {
                if roll() < FLEE_CHANCE {
                    self.log.push("You escape!".to_string());
                    return Ok(self.finish(Outcome::Escaped));
                }
                self.log.push("You fail to get away.".to_string());
            }
            Action::Surrender => {
                self.log.push("You surrender.".to_string());
                return Ok(self.finish(Outcome::Surrendered));
            }
        }

        fire(&self.enemy, &mut self.player, roll, &mut self.log);
        if self.player.is_destroyed() {
            self.log.push("Your ship is crippled!".to_string());
            return Ok(self.finish(Outcome::Defeat));
        }

        self.player.recharge_shield();
        self.enemy.recharge_shield();
        Ok(Outcome::Ongoing)
    }

    fn finish(&mut self, outcome: Outcome) -> Outcome {
        self.outcome = outcome;
        outcome
    }
}

fn fire(
    attacker: &Combatant,
    target: &mut Combatant,
    roll: &mut impl FnMut() -> f64,
    log: &mut Vec<String>,
) {
    for weapon in &attacker.weapons {
        if roll() < weapon.accuracy {
            target.take_damage(weapon.damage);
            log.push(format!(
                "{} hits {} for {} damage.",
                attacker.name, target.name, weapon.damage
            ));
        } else {
            log.push(format!("{} misses {}.", attacker.name, target.name));
        }
    }
}

#[cfg(test)]
mod tests;
