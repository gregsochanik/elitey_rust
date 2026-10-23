use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EquipmentCategory {
    Weapon,
    Shield,
    Engine,
    Sensor,
    Cargo,
    Special,
}

impl EquipmentCategory {
    pub const ALL: [EquipmentCategory; 6] = [
        EquipmentCategory::Weapon,
        EquipmentCategory::Shield,
        EquipmentCategory::Engine,
        EquipmentCategory::Sensor,
        EquipmentCategory::Cargo,
        EquipmentCategory::Special,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            EquipmentCategory::Weapon => "Weapons",
            EquipmentCategory::Shield => "Shields",
            EquipmentCategory::Engine => "Engines",
            EquipmentCategory::Sensor => "Sensors",
            EquipmentCategory::Cargo => "Cargo",
            EquipmentCategory::Special => "Special",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CombatStats {
    pub damage: u32,
    pub accuracy: f64,
    pub protection: u32,
    pub recharge: u32,
}

impl CombatStats {
    pub const NONE: CombatStats = CombatStats {
        damage: 0,
        accuracy: 0.0,
        protection: 0,
        recharge: 0,
    };

    pub const fn weapon(damage: u32, accuracy: f64) -> CombatStats {
        CombatStats {
            damage,
            accuracy,
            protection: 0,
            recharge: 0,
        }
    }

    pub const fn shield(protection: u32, recharge: u32) -> CombatStats {
        CombatStats {
            damage: 0,
            accuracy: 0.0,
            protection,
            recharge,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Equipment {
    pub id: &'static str,
    pub name: &'static str,
    pub category: EquipmentCategory,
    pub quality: Quality,
    pub price: u32,
    pub power: u32,
    pub cargo_multiplier: f64,
    pub combat: CombatStats,
    pub effect: &'static str,
}

pub fn find(id: &str) -> Option<&'static Equipment> {
    CATALOG.iter().find(|item| item.id == id)
}

pub fn catalog_for(category: EquipmentCategory) -> impl Iterator<Item = &'static Equipment> {
    CATALOG.iter().filter(move |item| item.category == category)
}

pub const CATALOG: &[Equipment] = &[
    Equipment {
        id: "pulse-laser",
        name: "Pulse Laser",
        category: EquipmentCategory::Weapon,
        quality: Quality::Basic,
        price: 50,
        power: 2,
        cargo_multiplier: 1.0,
        combat: CombatStats::weapon(15, 0.9),
        effect: "Damage 15, accuracy 0.9",
    },
    Equipment {
        id: "beam-laser",
        name: "Beam Laser",
        category: EquipmentCategory::Weapon,
        quality: Quality::Standard,
        price: 120,
        power: 4,
        cargo_multiplier: 1.0,
        combat: CombatStats::weapon(25, 0.8),
        effect: "Damage 25, accuracy 0.8",
    },
    Equipment {
        id: "plasma-cannon",
        name: "Plasma Cannon",
        category: EquipmentCategory::Weapon,
        quality: Quality::Military,
        price: 250,
        power: 6,
        cargo_multiplier: 1.0,
        combat: CombatStats::weapon(45, 0.6),
        effect: "Damage 45, accuracy 0.6",
    },
    Equipment {
        id: "missile-launcher",
        name: "Missile Launcher",
        category: EquipmentCategory::Weapon,
        quality: Quality::Elite,
        price: 500,
        power: 8,
        cargo_multiplier: 1.0,
        combat: CombatStats::weapon(80, 0.4),
        effect: "Damage 80, accuracy 0.4",
    },
    Equipment {
        id: "basic-shield",
        name: "Basic Shield",
        category: EquipmentCategory::Shield,
        quality: Quality::Basic,
        price: 30,
        power: 3,
        cargo_multiplier: 1.0,
        combat: CombatStats::shield(20, 5),
        effect: "Protection 20, recharge 5",
    },
    Equipment {
        id: "standard-shield",
        name: "Standard Shield",
        category: EquipmentCategory::Shield,
        quality: Quality::Standard,
        price: 80,
        power: 5,
        cargo_multiplier: 1.0,
        combat: CombatStats::shield(40, 8),
        effect: "Protection 40, recharge 8",
    },
    Equipment {
        id: "military-shield",
        name: "Military Shield",
        category: EquipmentCategory::Shield,
        quality: Quality::Military,
        price: 180,
        power: 7,
        cargo_multiplier: 1.0,
        combat: CombatStats::shield(70, 12),
        effect: "Protection 70, recharge 12",
    },
    Equipment {
        id: "elite-shield",
        name: "Elite Shield",
        category: EquipmentCategory::Shield,
        quality: Quality::Elite,
        price: 350,
        power: 10,
        cargo_multiplier: 1.0,
        combat: CombatStats::shield(100, 15),
        effect: "Protection 100, recharge 15",
    },
    Equipment {
        id: "standard-drive",
        name: "Standard Drive",
        category: EquipmentCategory::Engine,
        quality: Quality::Basic,
        price: 20,
        power: 2,
        cargo_multiplier: 1.0,
        combat: CombatStats::NONE,
        effect: "Speed 1.0x, fuel 1.0x",
    },
    Equipment {
        id: "performance-drive",
        name: "Performance Drive",
        category: EquipmentCategory::Engine,
        quality: Quality::Standard,
        price: 60,
        power: 3,
        cargo_multiplier: 1.0,
        combat: CombatStats::NONE,
        effect: "Speed 1.2x, fuel 1.1x",
    },
    Equipment {
        id: "military-drive",
        name: "Military Drive",
        category: EquipmentCategory::Engine,
        quality: Quality::Military,
        price: 150,
        power: 5,
        cargo_multiplier: 1.0,
        combat: CombatStats::NONE,
        effect: "Speed 1.4x, fuel 1.2x",
    },
    Equipment {
        id: "elite-drive",
        name: "Elite Drive",
        category: EquipmentCategory::Engine,
        quality: Quality::Elite,
        price: 300,
        power: 8,
        cargo_multiplier: 1.0,
        combat: CombatStats::NONE,
        effect: "Speed 1.6x, fuel 1.3x",
    },
    Equipment {
        id: "basic-scanner",
        name: "Basic Scanner",
        category: EquipmentCategory::Sensor,
        quality: Quality::Basic,
        price: 15,
        power: 1,
        cargo_multiplier: 1.0,
        combat: CombatStats::NONE,
        effect: "Range 1.0x, targeting +0",
    },
    Equipment {
        id: "advanced-scanner",
        name: "Advanced Scanner",
        category: EquipmentCategory::Sensor,
        quality: Quality::Standard,
        price: 40,
        power: 2,
        cargo_multiplier: 1.0,
        combat: CombatStats::NONE,
        effect: "Range 1.5x, targeting +10",
    },
    Equipment {
        id: "military-scanner",
        name: "Military Scanner",
        category: EquipmentCategory::Sensor,
        quality: Quality::Military,
        price: 100,
        power: 4,
        cargo_multiplier: 1.0,
        combat: CombatStats::NONE,
        effect: "Range 2.0x, targeting +25",
    },
    Equipment {
        id: "elite-scanner",
        name: "Elite Scanner",
        category: EquipmentCategory::Sensor,
        quality: Quality::Elite,
        price: 200,
        power: 6,
        cargo_multiplier: 1.0,
        combat: CombatStats::NONE,
        effect: "Range 3.0x, targeting +50",
    },
    Equipment {
        id: "standard-racks",
        name: "Standard Racks",
        category: EquipmentCategory::Cargo,
        quality: Quality::Basic,
        price: 10,
        power: 0,
        cargo_multiplier: 1.0,
        combat: CombatStats::NONE,
        effect: "Cargo capacity 1.0x",
    },
    Equipment {
        id: "expanded-racks",
        name: "Expanded Racks",
        category: EquipmentCategory::Cargo,
        quality: Quality::Standard,
        price: 30,
        power: 1,
        cargo_multiplier: 1.25,
        combat: CombatStats::NONE,
        effect: "Cargo capacity 1.25x",
    },
    Equipment {
        id: "military-racks",
        name: "Military Racks",
        category: EquipmentCategory::Cargo,
        quality: Quality::Military,
        price: 80,
        power: 2,
        cargo_multiplier: 1.5,
        combat: CombatStats::NONE,
        effect: "Cargo capacity 1.5x",
    },
    Equipment {
        id: "elite-racks",
        name: "Elite Racks",
        category: EquipmentCategory::Cargo,
        quality: Quality::Elite,
        price: 150,
        power: 3,
        cargo_multiplier: 2.0,
        combat: CombatStats::NONE,
        effect: "Cargo capacity 2.0x",
    },
    Equipment {
        id: "heat-sink",
        name: "Heat Sink",
        category: EquipmentCategory::Special,
        quality: Quality::Basic,
        price: 20,
        power: 1,
        cargo_multiplier: 1.0,
        combat: CombatStats::NONE,
        effect: "-30% heat",
    },
    Equipment {
        id: "chaff-launcher",
        name: "Chaff Launcher",
        category: EquipmentCategory::Special,
        quality: Quality::Standard,
        price: 50,
        power: 2,
        cargo_multiplier: 1.0,
        combat: CombatStats::NONE,
        effect: "+40% evasion",
    },
    Equipment {
        id: "ecm-system",
        name: "ECM System",
        category: EquipmentCategory::Special,
        quality: Quality::Military,
        price: 120,
        power: 4,
        cargo_multiplier: 1.0,
        combat: CombatStats::NONE,
        effect: "+60% jamming",
    },
    Equipment {
        id: "cloaking-device",
        name: "Cloaking Device",
        category: EquipmentCategory::Special,
        quality: Quality::Elite,
        price: 250,
        power: 8,
        cargo_multiplier: 1.0,
        combat: CombatStats::NONE,
        effect: "Temporary invisibility",
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn ids_are_unique() {
        let ids: HashSet<_> = CATALOG.iter().map(|item| item.id).collect();
        assert_eq!(ids.len(), CATALOG.len());
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
    fn only_cargo_racks_change_capacity() {
        for item in CATALOG {
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
        for item in CATALOG {
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
}
