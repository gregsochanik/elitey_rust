use crate::game::commander::ReputationChange;
use crate::game::{Commander, Ship};
use thiserror::Error;

pub const CREDITS_PER_UNIT: u32 = 2;
const RESCUE_REPUTATION: i32 = -2;

/// What a refuel did
#[derive(Debug, Clone, PartialEq)]
pub enum RefuelOutcome {
    Bought {
        units: u32,
        cost: u32,
        fuel: u32,
        capacity: u32,
    },
    /// A stranded ship was topped up by a rescue crew at a reputation cost
    Rescued {
        fuel: u32,
        capacity: u32,
        change: ReputationChange,
    },
}

/// Why a refuel was refused
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RefuelError {
    #[error("Your tank is already full.")]
    TankFull,
    #[error(
        "Fuel costs {} credits per light year; you can't afford any.",
        CREDITS_PER_UNIT
    )]
    CannotAfford {
        /// The ship could ask for a rescue crew instead
        stranded: bool,
    },
}

fn is_stranded(commander: &Commander, ship: &Ship, nearest_jump_cost: u32) -> bool {
    ship.cargo_hold().goods_weight() == 0
        && ship.fuel_current + commander.credits() / CREDITS_PER_UNIT < nearest_jump_cost
}

fn rescue(commander: &mut Commander, ship: &mut Ship, nearest_jump_cost: u32) -> RefuelOutcome {
    ship.fuel_current = nearest_jump_cost.min(ship.fuel_capacity());
    RefuelOutcome::Rescued {
        fuel: ship.fuel_current,
        capacity: ship.fuel_capacity(),
        change: commander.change_reputation(RESCUE_REPUTATION),
    }
}

/// Refuel `units` (all that fits and is affordable when `None`). With no amount, a stranded
/// ship is rescued instead.
pub fn refuel(
    commander: &mut Commander,
    ship: &mut Ship,
    units: Option<u32>,
    nearest_jump_cost: u32,
) -> Result<RefuelOutcome, RefuelError> {
    let stranded = is_stranded(commander, ship, nearest_jump_cost);
    if stranded && units.is_none() {
        return Ok(rescue(commander, ship, nearest_jump_cost));
    }

    let missing = ship.fuel_capacity().saturating_sub(ship.fuel_current);
    if missing == 0 {
        return Err(RefuelError::TankFull);
    }

    let affordable = commander.credits() / CREDITS_PER_UNIT;
    if affordable == 0 {
        return Err(RefuelError::CannotAfford { stranded });
    }

    let units = units.unwrap_or(missing).min(missing).min(affordable);
    commander.remove_credits(units * CREDITS_PER_UNIT);
    ship.fuel_current += units;
    Ok(RefuelOutcome::Bought {
        units,
        cost: units * CREDITS_PER_UNIT,
        fuel: ship.fuel_current,
        capacity: ship.fuel_capacity(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn low_on_fuel(credits: u32, fuel: u32) -> (Commander, Ship) {
        let mut commander = Commander::new("Test".to_string());
        commander.credits = credits;
        let mut ship = Ship::new_cobra_mk3();
        ship.fuel_current = fuel;
        (commander, ship)
    }

    #[test]
    fn fills_the_tank_at_the_unit_price() {
        let (mut commander, mut ship) = low_on_fuel(100, 2);
        assert!(refuel(&mut commander, &mut ship, None, 3).is_ok());
        assert_eq!(ship.fuel_current, 7);
        assert_eq!(commander.credits(), 90);
    }

    #[test]
    fn refuels_as_much_as_affordable() {
        let (mut commander, mut ship) = low_on_fuel(7, 0);
        assert!(refuel(&mut commander, &mut ship, None, 3).is_ok());
        assert_eq!(ship.fuel_current, 3);
        assert_eq!(commander.credits(), 1);
    }

    #[test]
    fn refuels_only_the_requested_units() {
        let (mut commander, mut ship) = low_on_fuel(100, 1);
        assert!(refuel(&mut commander, &mut ship, Some(2), 3).is_ok());
        assert_eq!(ship.fuel_current, 3);
        assert_eq!(commander.credits(), 96);
    }

    #[test]
    fn requested_units_are_capped_at_capacity() {
        let (mut commander, mut ship) = low_on_fuel(100, 6);
        assert!(refuel(&mut commander, &mut ship, Some(50), 3).is_ok());
        assert_eq!(ship.fuel_current, 7);
        assert_eq!(commander.credits(), 98);
    }

    #[test]
    fn full_tank_is_refused_and_free() {
        let (mut commander, mut ship) = low_on_fuel(100, 7);
        assert!(refuel(&mut commander, &mut ship, None, 3).is_err());
        assert_eq!(commander.credits(), 100);
    }

    #[test]
    fn no_credits_leaves_the_tank_alone() {
        let (mut commander, mut ship) = low_on_fuel(1, 4);
        assert!(refuel(&mut commander, &mut ship, None, 3).is_err());
        assert_eq!(ship.fuel_current, 4);
        assert_eq!(commander.credits(), 1);
    }

    #[test]
    fn stranded_ship_is_rescued_for_a_reputation_cost() {
        let (mut commander, mut ship) = low_on_fuel(1, 1);
        assert!(refuel(&mut commander, &mut ship, None, 3).is_ok());
        assert_eq!(ship.fuel_current, 3);
        assert_eq!(commander.credits(), 1);
        assert_eq!(commander.reputation, -2);
    }

    #[test]
    fn an_explicit_amount_never_triggers_the_rescue() {
        let (mut commander, mut ship) = low_on_fuel(1, 1);
        assert!(refuel(&mut commander, &mut ship, Some(2), 3).is_err());
        assert_eq!(ship.fuel_current, 1);
        assert_eq!(commander.reputation, 0);
    }

    #[test]
    fn rescue_is_not_given_when_the_player_can_still_afford_to_move() {
        let (mut commander, mut ship) = low_on_fuel(4, 1);
        assert!(refuel(&mut commander, &mut ship, None, 3).is_ok());
        assert_eq!(ship.fuel_current, 3);
        assert_eq!(commander.credits(), 0);
        assert_eq!(commander.reputation, 0);
    }

    #[test]
    fn rescue_is_not_given_while_there_is_cargo_to_sell() {
        let (mut commander, mut ship) = low_on_fuel(1, 1);
        ship.add_cargo(crate::data::commodities::CommodityType::Food, 2);
        assert!(refuel(&mut commander, &mut ship, None, 3).is_err());
        assert_eq!(ship.fuel_current, 1);
        assert_eq!(commander.reputation, 0);
    }

    #[test]
    fn rescue_never_exceeds_the_tank() {
        let (mut commander, mut ship) = low_on_fuel(0, 0);
        assert!(refuel(&mut commander, &mut ship, None, 50).is_ok());
        assert_eq!(ship.fuel_current, ship.fuel_capacity());
    }

    #[test]
    fn upgraded_ships_hold_more() {
        let (mut commander, mut ship) = low_on_fuel(1000, 0);
        ship.ship_type = crate::data::ships::ShipType::from_string("python").unwrap();
        assert!(refuel(&mut commander, &mut ship, None, 3).is_ok());
        assert_eq!(ship.fuel_current, 12);
        assert_eq!(commander.credits(), 976);
    }

    #[test]
    fn contract_cargo_does_not_stop_a_rescue_because_it_cannot_be_sold() {
        let (mut commander, mut ship) = low_on_fuel(1, 1);
        assert!(ship.cargo_hold.reserve(10));
        assert!(refuel(&mut commander, &mut ship, None, 3).is_ok());
        assert_eq!(ship.fuel_current, 3);
    }

    #[test]
    fn the_outcome_reports_units_cost_and_the_tank() {
        let (mut commander, mut ship) = low_on_fuel(100, 2);
        assert_eq!(
            refuel(&mut commander, &mut ship, None, 3),
            Ok(RefuelOutcome::Bought {
                units: 5,
                cost: 10,
                fuel: 7,
                capacity: 7
            })
        );
    }

    #[test]
    fn refusals_say_whether_the_ship_is_stranded() {
        let (mut commander, mut ship) = low_on_fuel(1, 1);
        assert_eq!(
            refuel(&mut commander, &mut ship, Some(2), 3),
            Err(RefuelError::CannotAfford { stranded: true })
        );
        let (mut full_commander, mut full_ship) = low_on_fuel(100, 7);
        assert_eq!(
            refuel(&mut full_commander, &mut full_ship, None, 3),
            Err(RefuelError::TankFull)
        );
    }
}
