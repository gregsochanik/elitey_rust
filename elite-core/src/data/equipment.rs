use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EquipmentCategory {
    Power,
    Weapon,
    Shield,
    Engine,
    Sensor,
    Cargo,
    Special,
}

impl EquipmentCategory {
    pub const ALL: [EquipmentCategory; 7] = [
        EquipmentCategory::Power,
        EquipmentCategory::Weapon,
        EquipmentCategory::Shield,
        EquipmentCategory::Engine,
        EquipmentCategory::Sensor,
        EquipmentCategory::Cargo,
        EquipmentCategory::Special,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            EquipmentCategory::Power => "Power Plant",
            EquipmentCategory::Weapon => "Weapons",
            EquipmentCategory::Shield => "Shields",
            EquipmentCategory::Engine => "Engines",
            EquipmentCategory::Sensor => "Sensors",
            EquipmentCategory::Cargo => "Cargo",
            EquipmentCategory::Special => "Special",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum Quality {
    Basic,
    Standard,
    Military,
    Elite,
}

impl Quality {
    pub fn name(&self) -> &'static str {
        match self {
            Quality::Basic => "Basic",
            Quality::Standard => "Standard",
            Quality::Military => "Military",
            Quality::Elite => "Elite",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct CombatStats {
    pub damage: u32,
    pub accuracy: f64,
    pub protection: u32,
    pub recharge: u32,
}

impl Default for CombatStats {
    fn default() -> Self {
        CombatStats::NONE
    }
}

impl CombatStats {
    pub const NONE: CombatStats = CombatStats {
        damage: 0,
        accuracy: 0.0,
        protection: 0,
        recharge: 0,
    };
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Equipment {
    pub id: String,
    pub name: String,
    pub category: EquipmentCategory,
    pub quality: Quality,
    /// The lowest system tech level that sells this item
    #[serde(default = "lowest_tech")]
    pub min_tech: u8,
    pub price: u32,
    pub power: u32,
    /// Power the item supplies; only reactors have any
    #[serde(default)]
    pub output: u32,
    #[serde(default = "no_cargo_change")]
    pub cargo_multiplier: f64,
    #[serde(default)]
    pub combat: CombatStats,
    pub effect: String,
}

fn lowest_tech() -> u8 {
    1
}

fn no_cargo_change() -> f64 {
    1.0
}

/// Every item for sale, as described in `data/equipment.json`
pub fn catalog() -> &'static [Equipment] {
    static CATALOG: OnceLock<Vec<Equipment>> = OnceLock::new();
    CATALOG.get_or_init(|| {
        serde_json::from_str(include_str!("../../data/equipment.json"))
            .expect("data/equipment.json is valid")
    })
}

pub fn find(id: &str) -> Option<&'static Equipment> {
    catalog().iter().find(|item| item.id == id)
}

pub fn catalog_for(category: EquipmentCategory) -> impl Iterator<Item = &'static Equipment> {
    catalog()
        .iter()
        .filter(move |item| item.category == category)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn ids_are_unique() {
        let ids: HashSet<_> = catalog().iter().map(|item| item.id.clone()).collect();
        assert_eq!(ids.len(), catalog().len());
    }

    #[test]
    fn every_category_has_four_quality_levels() {
        for category in EquipmentCategory::ALL {
            let qualities: Vec<_> = catalog_for(category).map(|item| item.quality).collect();
            assert_eq!(
                qualities,
                vec![
                    Quality::Basic,
                    Quality::Standard,
                    Quality::Military,
                    Quality::Elite
                ],
                "{:?}",
                category
            );
        }
    }

    #[test]
    fn find_returns_item_by_id() {
        assert_eq!(find("pulse-laser").unwrap().name, "Pulse Laser");
        assert!(find("nonsense").is_none());
    }

    #[test]
    fn only_reactors_supply_power_and_they_draw_none() {
        for item in catalog() {
            let is_reactor = item.category == EquipmentCategory::Power;
            assert_eq!(item.output > 0, is_reactor, "{}", item.id);
            if is_reactor {
                assert_eq!(item.power, 0, "{}", item.id);
            }
        }
        let outputs: Vec<u32> = catalog_for(EquipmentCategory::Power)
            .map(|item| item.output)
            .collect();
        assert_eq!(outputs, vec![20, 28, 38, 50]);
    }

    #[test]
    fn only_cargo_racks_change_capacity() {
        for item in catalog() {
            if item.category != EquipmentCategory::Cargo {
                assert_eq!(item.cargo_multiplier, 1.0, "{}", item.id);
            }
        }
        assert_eq!(find("standard-racks").unwrap().cargo_multiplier, 1.0);
        assert_eq!(find("expanded-racks").unwrap().cargo_multiplier, 1.25);
        assert_eq!(find("military-racks").unwrap().cargo_multiplier, 1.5);
        assert_eq!(find("elite-racks").unwrap().cargo_multiplier, 2.0);
    }

    #[test]
    fn weapons_and_shields_carry_the_combat_stats_from_the_spec() {
        let pulse = find("pulse-laser").unwrap().combat;
        assert_eq!((pulse.damage, pulse.accuracy), (15, 0.9));
        let missile = find("missile-launcher").unwrap().combat;
        assert_eq!((missile.damage, missile.accuracy), (80, 0.4));
        let basic = find("basic-shield").unwrap().combat;
        assert_eq!((basic.protection, basic.recharge), (20, 5));
        let elite = find("elite-shield").unwrap().combat;
        assert_eq!((elite.protection, elite.recharge), (100, 15));
    }

    #[test]
    fn only_weapons_and_shields_have_combat_stats() {
        for item in catalog() {
            match item.category {
                EquipmentCategory::Weapon => {
                    assert!(
                        item.combat.damage > 0
                            && item.combat.accuracy > 0.0
                            && item.combat.accuracy <= 1.0,
                        "{}",
                        item.id
                    );
                    assert!(
                        item.effect.contains(&item.combat.damage.to_string()),
                        "{}",
                        item.id
                    );
                }
                EquipmentCategory::Shield => {
                    assert!(
                        item.combat.protection > 0 && item.combat.recharge > 0,
                        "{}",
                        item.id
                    );
                    assert!(
                        item.effect.contains(&item.combat.protection.to_string()),
                        "{}",
                        item.id
                    );
                }
                _ => assert_eq!(item.combat, CombatStats::NONE, "{}", item.id),
            }
        }
    }

    #[test]
    fn prices_and_power_rise_with_quality_in_every_category() {
        for category in EquipmentCategory::ALL {
            let items: Vec<_> = catalog_for(category).collect();
            for pair in items.windows(2) {
                assert!(
                    pair[1].price > pair[0].price,
                    "{} costs no more",
                    pair[1].id
                );
                assert!(
                    pair[1].power >= pair[0].power,
                    "{} needs less power",
                    pair[1].id
                );
            }
        }
    }

    #[test]
    fn names_and_effects_are_filled_in() {
        for item in catalog() {
            assert!(
                !item.name.is_empty() && !item.effect.is_empty(),
                "{}",
                item.id
            );
            assert!(
                item.id.chars().all(|c| c.is_ascii_lowercase() || c == '-'),
                "{}",
                item.id
            );
        }
    }

    #[test]
    fn unknown_fields_in_the_data_are_rejected() {
        let json = r#"[{"id":"x","name":"X","category":"Weapon","quality":"Basic","price":1,"power":1,"effect":"e","speed":3}]"#;
        assert!(serde_json::from_str::<Vec<Equipment>>(json).is_err());
        let json = r#"[{"id":"x","name":"X","category":"Weapon","quality":"Basic","price":1,"power":1,"effect":"e","combat":{"damage":1,"speed":3}}]"#;
        assert!(serde_json::from_str::<Vec<Equipment>>(json).is_err());
    }

    #[test]
    fn an_item_without_combat_stats_or_a_cargo_multiplier_gets_the_defaults() {
        let json = r#"[{"id":"x","name":"X","category":"Special","quality":"Basic","price":1,"power":1,"effect":"e"}]"#;
        let items: Vec<Equipment> = serde_json::from_str(json).unwrap();
        assert_eq!(items[0].combat, CombatStats::NONE);
        assert_eq!(items[0].cargo_multiplier, 1.0);
    }
}
