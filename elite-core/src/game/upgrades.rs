use crate::data::ships::{ShipType, ShipUpgradeManager};
use crate::game::{Commander, Ship};
use thiserror::Error;

/// Upgrade command handler
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UpgradeHandler {
    upgrade_manager: ShipUpgradeManager,
}

/// Why a ship purchase was refused
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum UpgradeError {
    #[error("Unknown ship type: {0}")]
    UnknownShip(String),
    #[error("{0} is not an upgrade from your current ship")]
    NotAnUpgrade(&'static str),
    #[error("Insufficient credits. Need {needed}cr, have {have}cr")]
    NotEnoughCredits { needed: u32, have: u32 },
    #[error("Upgrade failed: {0}")]
    Failed(String),
}

/// What buying a ship did
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpgradeReceipt {
    pub ship: ShipType,
    pub cost: u32,
    /// Credits from selling the old ship's equipment
    pub equipment_refund: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpgradeOption {
    pub ship: ShipType,
    pub cost: u32,
    pub affordable: bool,
}

/// The ships the current one can be upgraded to
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpgradeOptions {
    pub current: ShipType,
    pub credits: u32,
    pub options: Vec<UpgradeOption>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShipListing {
    pub ship: ShipType,
    pub is_current: bool,
    /// What it costs to switch to this ship, if that costs anything
    pub upgrade_cost: Option<u32>,
    pub affordable: bool,
}

/// Every ship in the game
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShipList {
    pub current: ShipType,
    pub credits: u32,
    pub ships: Vec<ShipListing>,
}

impl UpgradeHandler {
    pub fn new() -> Self {
        Self {
            upgrade_manager: ShipUpgradeManager::new(),
        }
    }

    pub fn upgrade_options(&self, commander: &Commander, ship: &Ship) -> UpgradeOptions {
        UpgradeOptions {
            current: *ship.ship_type(),
            credits: commander.credits(),
            options: self
                .upgrade_manager
                .get_available_upgrades(ship.ship_type())
                .into_iter()
                .map(|upgrade| {
                    let cost = self
                        .upgrade_manager
                        .calculate_upgrade_cost(ship.ship_type(), upgrade);
                    UpgradeOption {
                        ship: *upgrade,
                        cost,
                        affordable: commander.has_credits(cost),
                    }
                })
                .collect(),
        }
    }

    pub fn ship_list(&self, commander: &Commander, ship: &Ship) -> ShipList {
        ShipList {
            current: *ship.ship_type(),
            credits: commander.credits(),
            ships: self
                .upgrade_manager
                .get_available_ships()
                .iter()
                .map(|candidate| {
                    let is_current = candidate == ship.ship_type();
                    let cost = self
                        .upgrade_manager
                        .calculate_upgrade_cost(ship.ship_type(), candidate);
                    ShipListing {
                        ship: *candidate,
                        is_current,
                        upgrade_cost: (!is_current && cost > 0).then_some(cost),
                        affordable: commander.has_credits(cost),
                    }
                })
                .collect(),
        }
    }

    /// Buy `new_ship_type`, selling the installed equipment at half price
    pub fn upgrade(
        &self,
        new_ship_type: ShipType,
        commander: &mut Commander,
        ship: &mut Ship,
    ) -> Result<UpgradeReceipt, UpgradeError> {
        if !new_ship_type.is_upgrade_from(ship.ship_type()) {
            return Err(UpgradeError::NotAnUpgrade(new_ship_type.display_name()));
        }

        let cost = self
            .upgrade_manager
            .calculate_upgrade_cost(ship.ship_type(), &new_ship_type);
        if !commander.has_credits(cost) {
            return Err(UpgradeError::NotEnoughCredits {
                needed: cost,
                have: commander.credits(),
            });
        }

        let equipment_refund = ship.equipment.total_refund();
        ship.upgrade_to(new_ship_type)
            .map_err(UpgradeError::Failed)?;
        commander.remove_credits(cost);
        commander.add_credits(equipment_refund);
        Ok(UpgradeReceipt {
            ship: new_ship_type,
            cost,
            equipment_refund,
        })
    }
}

impl Default for UpgradeHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upgrade_handler_creation() {
        let handler = UpgradeHandler::new();
        assert_eq!(
            handler
                .upgrade_manager
                .get_available_upgrades(&ShipType::starting())
                .len(),
            3
        );
    }

    #[test]
    fn changing_ship_sells_equipment_at_half_price_and_resets_loadout() {
        let handler = UpgradeHandler::new();
        let mut commander = Commander::new("Test".to_string());
        commander.credits = 5_000;
        let mut ship = Ship::new_cobra_mk3();
        let mut loadout = ship.equipment.clone();
        loadout
            .install(crate::data::equipment::find("expanded-racks").unwrap(), 30)
            .unwrap();
        ship.set_equipment(loadout).unwrap();

        let cost = handler.upgrade_manager.calculate_upgrade_cost(
            &ShipType::starting(),
            &ShipType::from_string("cobra4").unwrap(),
        );
        let receipt = handler
            .upgrade(
                ShipType::from_string("cobra4").unwrap(),
                &mut commander,
                &mut ship,
            )
            .unwrap();

        assert_eq!(receipt.cost, cost);
        assert_eq!(receipt.equipment_refund, 15);
        assert_eq!(commander.credits(), 5_000 - cost + 15);
        assert_eq!(ship.equipment, crate::game::equipment::Loadout::starting());
        assert_eq!(ship.cargo_capacity(), 35);
    }

    #[test]
    fn refusals_say_why() {
        let handler = UpgradeHandler::new();
        let mut commander = Commander::new("Test".to_string());
        commander.credits = 100;
        let mut ship = Ship::new_cobra_mk3();

        assert_eq!(
            handler.upgrade(ShipType::starting(), &mut commander, &mut ship),
            Err(UpgradeError::NotAnUpgrade("Cobra Mk III"))
        );
        assert_eq!(
            handler.upgrade(
                ShipType::from_string("python").unwrap(),
                &mut commander,
                &mut ship
            ),
            Err(UpgradeError::NotEnoughCredits {
                needed: 2000,
                have: 100
            })
        );
    }

    #[test]
    fn the_options_mark_what_can_be_afforded() {
        let handler = UpgradeHandler::new();
        let mut commander = Commander::new("Test".to_string());
        commander.credits = 600;
        let ship = Ship::new_cobra_mk3();

        let options = handler.upgrade_options(&commander, &ship).options;

        assert_eq!(options.len(), 3);
        assert!(options[0].affordable && !options[1].affordable);
        let list = handler.ship_list(&commander, &ship);
        assert!(list.ships[0].is_current && list.ships[0].upgrade_cost.is_none());
    }
}
