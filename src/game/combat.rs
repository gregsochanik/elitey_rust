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

    pub fn pirate_raider() -> Self {
        Self {
            name: "Pirate Raider".to_string(),
            hull: 70,
            max_hull: 70,
            shield: 25,
            max_shield: 25,
            recharge: 4,
            weapons: vec![Weapon {
                damage: 14,
                accuracy: 0.65,
            }],
        }
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
mod tests {
    use super::*;

    fn fighter(
        name: &str,
        hull: u32,
        shield: u32,
        recharge: u32,
        weapons: &[(u32, f64)],
    ) -> Combatant {
        Combatant {
            name: name.to_string(),
            hull,
            max_hull: hull,
            shield,
            max_shield: shield,
            recharge,
            weapons: weapons
                .iter()
                .map(|(damage, accuracy)| Weapon {
                    damage: *damage,
                    accuracy: *accuracy,
                })
                .collect(),
        }
    }

    /// Dice that return the given values in order and fail the test if asked for more
    fn rolls(values: &[f64]) -> impl FnMut() -> f64 + '_ {
        let mut remaining = values.iter();
        move || *remaining.next().expect("more rolls than expected")
    }

    #[test]
    fn a_shot_hits_when_the_roll_is_below_accuracy_and_misses_otherwise() {
        let mut combat = Combat::new(
            fighter("You", 100, 0, 0, &[(15, 0.9)]),
            fighter("Raider", 60, 0, 0, &[(10, 0.6)]),
        );

        // player hits, raider misses
        combat
            .resolve_round(Action::Attack, &mut rolls(&[0.5, 0.7]))
            .unwrap();
        assert_eq!(combat.enemy.hull, 45);
        assert_eq!(combat.player.hull, 100);

        // player misses, raider hits
        combat
            .resolve_round(Action::Attack, &mut rolls(&[0.95, 0.1]))
            .unwrap();
        assert_eq!(combat.enemy.hull, 45);
        assert_eq!(combat.player.hull, 90);
    }

    #[test]
    fn shields_absorb_damage_before_the_hull() {
        let mut combat = Combat::new(
            fighter("You", 100, 0, 0, &[(15, 1.0)]),
            fighter("Raider", 60, 20, 0, &[(10, 0.0)]),
        );

        combat
            .resolve_round(Action::Attack, &mut rolls(&[0.0, 0.99]))
            .unwrap();

        assert_eq!(combat.enemy.shield, 5);
        assert_eq!(combat.enemy.hull, 60);
    }

    #[test]
    fn damage_beyond_the_shield_carries_over_to_the_hull() {
        let mut combat = Combat::new(
            fighter("You", 100, 0, 0, &[(15, 1.0)]),
            fighter("Raider", 60, 10, 0, &[(10, 0.0)]),
        );

        combat
            .resolve_round(Action::Attack, &mut rolls(&[0.0, 0.99]))
            .unwrap();

        assert_eq!(combat.enemy.shield, 0);
        assert_eq!(combat.enemy.hull, 55);
    }

    #[test]
    fn shields_recharge_each_round_but_never_above_the_maximum() {
        let mut combat = Combat::new(
            fighter("You", 100, 20, 5, &[(1, 0.0)]),
            fighter("Raider", 60, 20, 3, &[(8, 1.0)]),
        );

        // raider hits for 8: shield 20 -> 12, then recharges to 17
        combat
            .resolve_round(Action::Attack, &mut rolls(&[0.99, 0.0]))
            .unwrap();
        assert_eq!(combat.player.shield, 17);

        // raider hits for 8 again: 17 -> 9 -> 14
        combat
            .resolve_round(Action::Attack, &mut rolls(&[0.99, 0.0]))
            .unwrap();
        assert_eq!(combat.player.shield, 14);

        // an undamaged shield stays at its maximum
        assert_eq!(combat.enemy.shield, 20);
    }

    #[test]
    fn every_installed_weapon_fires_each_round() {
        let mut combat = Combat::new(
            fighter("You", 100, 0, 0, &[(10, 1.0), (7, 1.0)]),
            fighter("Raider", 60, 0, 0, &[(1, 0.0)]),
        );

        combat
            .resolve_round(Action::Attack, &mut rolls(&[0.0, 0.0, 0.99]))
            .unwrap();

        assert_eq!(combat.enemy.hull, 43);
    }

    #[test]
    fn a_destroyed_enemy_does_not_fire_back() {
        let mut combat = Combat::new(
            fighter("You", 100, 0, 0, &[(80, 1.0)]),
            fighter("Raider", 60, 0, 0, &[(10, 1.0)]),
        );

        // only one roll is available: the player's shot
        let outcome = combat
            .resolve_round(Action::Attack, &mut rolls(&[0.0]))
            .unwrap();

        assert_eq!(outcome, Outcome::Victory);
        assert_eq!(combat.player.hull, 100);
    }

    #[test]
    fn the_player_is_defeated_when_the_hull_reaches_zero() {
        let mut combat = Combat::new(
            fighter("You", 8, 0, 0, &[(1, 0.0)]),
            fighter("Raider", 60, 0, 0, &[(10, 1.0)]),
        );

        let outcome = combat
            .resolve_round(Action::Attack, &mut rolls(&[0.99, 0.0]))
            .unwrap();

        assert_eq!(outcome, Outcome::Defeat);
        assert_eq!(combat.player.hull, 0);
    }

    #[test]
    fn fleeing_succeeds_below_the_flee_chance_and_ends_the_fight() {
        let mut combat = Combat::new(
            fighter("You", 100, 0, 0, &[(10, 1.0)]),
            fighter("Raider", 60, 0, 0, &[(10, 1.0)]),
        );

        let outcome = combat
            .resolve_round(Action::Flee, &mut rolls(&[0.3]))
            .unwrap();

        assert_eq!(outcome, Outcome::Escaped);
        assert_eq!(combat.player.hull, 100);
        assert_eq!(combat.enemy.hull, 60);
    }

    #[test]
    fn a_failed_escape_gives_the_enemy_a_free_attack() {
        let mut combat = Combat::new(
            fighter("You", 100, 0, 0, &[(10, 1.0)]),
            fighter("Raider", 60, 0, 0, &[(10, 1.0)]),
        );

        let outcome = combat
            .resolve_round(Action::Flee, &mut rolls(&[0.7, 0.0]))
            .unwrap();

        assert_eq!(outcome, Outcome::Ongoing);
        assert_eq!(combat.player.hull, 90);
        assert_eq!(combat.enemy.hull, 60);
    }

    #[test]
    fn surrendering_ends_the_fight_without_damage() {
        let mut combat = Combat::new(
            fighter("You", 100, 0, 0, &[(10, 1.0)]),
            fighter("Raider", 60, 0, 0, &[(10, 1.0)]),
        );

        let outcome = combat
            .resolve_round(Action::Surrender, &mut rolls(&[]))
            .unwrap();

        assert_eq!(outcome, Outcome::Surrendered);
        assert_eq!(combat.player.hull, 100);
    }

    #[test]
    fn attacking_without_weapons_is_refused_and_takes_no_turn() {
        let mut combat = Combat::new(
            fighter("You", 100, 20, 5, &[]),
            fighter("Raider", 60, 20, 3, &[(10, 1.0)]),
        );
        let before = (combat.player.clone(), combat.enemy.clone());

        let result = combat.resolve_round(Action::Attack, &mut rolls(&[]));

        assert_eq!(result, Err(CombatError::NoWeapons));
        assert_eq!((combat.player.clone(), combat.enemy.clone()), before);
        assert_eq!(combat.outcome(), Outcome::Ongoing);
    }

    #[test]
    fn an_unarmed_ship_can_still_flee_or_surrender() {
        let mut combat = Combat::new(
            fighter("You", 100, 0, 0, &[]),
            fighter("Raider", 60, 0, 0, &[(10, 1.0)]),
        );
        assert_eq!(
            combat
                .resolve_round(Action::Flee, &mut rolls(&[0.1]))
                .unwrap(),
            Outcome::Escaped
        );

        let mut combat = Combat::new(
            fighter("You", 100, 0, 0, &[]),
            fighter("Raider", 60, 0, 0, &[(10, 1.0)]),
        );
        assert_eq!(
            combat
                .resolve_round(Action::Surrender, &mut rolls(&[]))
                .unwrap(),
            Outcome::Surrendered
        );
    }

    #[test]
    fn no_more_rounds_can_be_played_after_the_fight_ends() {
        let mut combat = Combat::new(
            fighter("You", 100, 0, 0, &[(10, 1.0)]),
            fighter("Raider", 60, 0, 0, &[(10, 1.0)]),
        );
        combat
            .resolve_round(Action::Surrender, &mut rolls(&[]))
            .unwrap();

        assert_eq!(
            combat.resolve_round(Action::Attack, &mut rolls(&[0.0, 0.0])),
            Err(CombatError::Finished)
        );
    }

    #[test]
    fn the_log_records_hits_misses_and_the_ending() {
        let mut combat = Combat::new(
            fighter("You", 100, 0, 0, &[(80, 1.0)]),
            fighter("Raider", 60, 0, 0, &[(10, 1.0)]),
        );
        combat
            .resolve_round(Action::Attack, &mut rolls(&[0.0]))
            .unwrap();

        assert_eq!(combat.log.len(), 2);
        assert!(combat.log[0].contains("hits"));
        assert!(combat.log[1].contains("destroyed"));
    }

    #[test]
    fn the_pirate_raider_has_the_specified_stats() {
        let raider = Combatant::pirate_raider();
        assert_eq!((raider.hull, raider.shield, raider.recharge), (70, 25, 4));
        assert_eq!(
            raider.weapons,
            vec![Weapon {
                damage: 14,
                accuracy: 0.65
            }]
        );
    }

    fn fitted(ship: &mut Ship, buy: &[&str], sell: &[&str]) {
        let mut commander = crate::game::Commander::new("Test".to_string());
        commander.credits = 10_000;
        for id in buy {
            assert!(crate::game::equipment::EquipmentHandler::handle(
                &["buy".to_string(), id.to_string()],
                &mut commander,
                ship
            ));
        }
        for id in sell {
            assert!(crate::game::equipment::EquipmentHandler::handle(
                &["sell".to_string(), id.to_string()],
                &mut commander,
                ship
            ));
        }
    }

    #[test]
    fn a_new_ship_fights_with_its_starting_equipment() {
        let player = Combatant::from_ship(&Ship::new_cobra_mk3());

        assert_eq!(player.hull, 100);
        assert_eq!(
            (player.shield, player.max_shield, player.recharge),
            (20, 20, 5)
        );
        assert_eq!(
            player.weapons,
            vec![Weapon {
                damage: 15,
                accuracy: 0.9
            }]
        );
    }

    #[test]
    fn every_installed_weapon_becomes_a_weapon() {
        let mut ship = Ship::new_cobra_mk3();
        fitted(&mut ship, &["beam-laser"], &[]);

        let player = Combatant::from_ship(&ship);

        assert_eq!(player.weapons.len(), 2);
        assert!(player.weapons.contains(&Weapon {
            damage: 25,
            accuracy: 0.8
        }));
    }

    #[test]
    fn a_better_shield_changes_protection_and_recharge() {
        let mut ship = Ship::new_cobra_mk3();
        fitted(&mut ship, &["elite-shield"], &[]);

        let player = Combatant::from_ship(&ship);

        assert_eq!((player.shield, player.recharge), (100, 15));
    }

    #[test]
    fn no_shield_means_no_shield_points() {
        let mut ship = Ship::new_cobra_mk3();
        fitted(&mut ship, &[], &["basic-shield"]);

        let player = Combatant::from_ship(&ship);

        assert_eq!(
            (player.shield, player.max_shield, player.recharge),
            (0, 0, 0)
        );
    }

    #[test]
    fn no_weapons_means_nothing_to_fire() {
        let mut ship = Ship::new_cobra_mk3();
        fitted(&mut ship, &[], &["pulse-laser"]);

        assert!(Combatant::from_ship(&ship).weapons.is_empty());
    }

    #[test]
    fn hull_follows_the_ships_damage() {
        let mut ship = Ship::new_cobra_mk3();
        ship.take_damage(30);

        assert_eq!(Combatant::from_ship(&ship).hull, 70);
    }

    #[test]
    fn non_combat_equipment_does_not_change_the_fighter() {
        let mut ship = Ship::new_cobra_mk3();
        let before = Combatant::from_ship(&ship);
        fitted(
            &mut ship,
            &["elite-racks", "performance-drive", "military-scanner"],
            &[],
        );

        assert_eq!(Combatant::from_ship(&ship), before);
    }
}
