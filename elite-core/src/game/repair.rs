use crate::game::{Commander, Ship};
use thiserror::Error;

pub const CREDITS_PER_POINT: u32 = 2;

/// What a repair did
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepairReceipt {
    pub points: u32,
    pub cost: u32,
    pub hull: u32,
    pub max_hull: u32,
}

/// Why a repair was refused
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RepairError {
    #[error("Your hull is already at full strength.")]
    HullFull,
    #[error(
        "Repairs cost {} credits per hull point; you can't afford any.",
        CREDITS_PER_POINT
    )]
    CannotAfford,
}

/// Repair `points` of hull damage (all of it when `None`), as far as credits allow
pub fn repair(
    commander: &mut Commander,
    ship: &mut Ship,
    points: Option<u32>,
) -> Result<RepairReceipt, RepairError> {
    let damage = ship.damage();
    if damage == 0 {
        return Err(RepairError::HullFull);
    }

    let affordable = commander.credits() / CREDITS_PER_POINT;
    if affordable == 0 {
        return Err(RepairError::CannotAfford);
    }

    let points = points.unwrap_or(damage).min(damage).min(affordable);
    commander.remove_credits(points * CREDITS_PER_POINT);
    ship.repair_hull(points);
    Ok(RepairReceipt {
        points,
        cost: points * CREDITS_PER_POINT,
        hull: ship.hull_percentage(),
        max_hull: ship.ship_type.max_hull(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn damaged(credits: u32, hull: u32) -> (Commander, Ship) {
        let mut commander = Commander::new("Test".to_string());
        commander.credits = credits;
        let mut ship = Ship::new_cobra_mk3();
        ship.hull_percentage = hull;
        (commander, ship)
    }

    #[test]
    fn repairs_to_full_at_two_credits_per_point() {
        let (mut commander, mut ship) = damaged(500, 70);
        assert!(repair(&mut commander, &mut ship, None).is_ok());
        assert_eq!(ship.hull_percentage(), 100);
        assert_eq!(commander.credits(), 440);
    }

    #[test]
    fn repairs_as_much_as_affordable() {
        let (mut commander, mut ship) = damaged(21, 50);
        assert!(repair(&mut commander, &mut ship, None).is_ok());
        assert_eq!(ship.hull_percentage(), 60);
        assert_eq!(commander.credits(), 1);
    }

    #[test]
    fn repairs_only_the_requested_points() {
        let (mut commander, mut ship) = damaged(500, 50);
        assert!(repair(&mut commander, &mut ship, Some(10)).is_ok());
        assert_eq!(ship.hull_percentage(), 60);
        assert_eq!(commander.credits(), 480);
    }

    #[test]
    fn requested_points_are_capped_at_the_damage() {
        let (mut commander, mut ship) = damaged(500, 90);
        assert!(repair(&mut commander, &mut ship, Some(50)).is_ok());
        assert_eq!(ship.hull_percentage(), 100);
        assert_eq!(commander.credits(), 480);
    }

    #[test]
    fn full_hull_is_refused_and_free() {
        let (mut commander, mut ship) = damaged(500, 100);
        assert!(repair(&mut commander, &mut ship, None).is_err());
        assert_eq!(commander.credits(), 500);
    }

    #[test]
    fn no_credits_leaves_the_hull_alone() {
        let (mut commander, mut ship) = damaged(1, 50);
        assert!(repair(&mut commander, &mut ship, None).is_err());
        assert_eq!(ship.hull_percentage(), 50);
        assert_eq!(commander.credits(), 1);
    }

    #[test]
    fn upgraded_ships_repair_up_to_their_own_maximum() {
        let (mut commander, mut ship) = damaged(1000, 100);
        ship.ship_type = crate::data::ships::ShipType::from_string("python").unwrap();
        assert!(repair(&mut commander, &mut ship, None).is_ok());
        assert_eq!(ship.hull_percentage(), 150);
        assert_eq!(commander.credits(), 900);
    }

    #[test]
    fn the_receipt_reports_points_cost_and_hull() {
        let (mut commander, mut ship) = damaged(500, 70);
        assert_eq!(
            repair(&mut commander, &mut ship, None),
            Ok(RepairReceipt {
                points: 30,
                cost: 60,
                hull: 100,
                max_hull: 100
            })
        );
        assert_eq!(
            repair(&mut commander, &mut ship, None),
            Err(RepairError::HullFull)
        );
    }
}
