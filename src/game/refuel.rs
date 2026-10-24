use crate::game::{Commander, Ship};
use crate::utils::display::{show_error, show_info, show_success};

pub const CREDITS_PER_UNIT: u32 = 2;
const RESCUE_REPUTATION: i32 = -2;

fn is_stranded(commander: &Commander, ship: &Ship, nearest_jump_cost: u32) -> bool {
    ship.cargo_used() == 0
        && ship.fuel_current + commander.credits() / CREDITS_PER_UNIT < nearest_jump_cost
}

fn rescue(commander: &mut Commander, ship: &mut Ship, nearest_jump_cost: u32) {
    ship.fuel_current = nearest_jump_cost.min(ship.fuel_capacity());
    let change = commander.change_reputation(RESCUE_REPUTATION);
    show_info(&format!(
        "Stranded! A rescue crew tops up your tank to {}/{} and files a report. Reputation {:+}.",
        ship.fuel_current,
        ship.fuel_capacity(),
        change.applied
    ));
    if let Some(standing) = change.new_standing {
        show_info(&format!("Your standing is now {}.", standing.name()));
    }
}

pub fn handle(
    args: &[String],
    commander: &mut Commander,
    ship: &mut Ship,
    nearest_jump_cost: u32,
) -> bool {
    let requested = match args.first() {
        None => None,
        Some(arg) => match arg.parse::<u32>() {
            Ok(units) if units > 0 && args.len() == 1 => Some(units),
            _ => {
                show_error("Usage: refuel [units]");
                return false;
            }
        },
    };

    if is_stranded(commander, ship, nearest_jump_cost) {
        rescue(commander, ship, nearest_jump_cost);
        return true;
    }

    let missing = ship.fuel_capacity().saturating_sub(ship.fuel_current);
    if missing == 0 {
        show_info("Your tank is already full.");
        return false;
    }

    let affordable = commander.credits() / CREDITS_PER_UNIT;
    if affordable == 0 {
        show_error(&format!(
            "Fuel costs {} credits per light year; you can't afford any.",
            CREDITS_PER_UNIT
        ));
        return false;
    }

    let units = requested.unwrap_or(missing).min(missing).min(affordable);
    commander.remove_credits(units * CREDITS_PER_UNIT);
    ship.fuel_current += units;
    show_success(&format!(
        "Bought {} light years of fuel for {} credits. Fuel: {}/{}",
        units,
        units * CREDITS_PER_UNIT,
        ship.fuel_current,
        ship.fuel_capacity()
    ));
    true
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

    fn args(text: &str) -> Vec<String> {
        text.split_whitespace().map(String::from).collect()
    }

    #[test]
    fn fills_the_tank_at_the_unit_price() {
        let (mut commander, mut ship) = low_on_fuel(100, 2);
        assert!(handle(&[], &mut commander, &mut ship, 3));
        assert_eq!(ship.fuel_current, 7);
        assert_eq!(commander.credits(), 90);
    }

    #[test]
    fn refuels_as_much_as_affordable() {
        let (mut commander, mut ship) = low_on_fuel(7, 0);
        assert!(handle(&[], &mut commander, &mut ship, 3));
        assert_eq!(ship.fuel_current, 3);
        assert_eq!(commander.credits(), 1);
    }

    #[test]
    fn refuels_only_the_requested_units() {
        let (mut commander, mut ship) = low_on_fuel(100, 1);
        assert!(handle(&args("2"), &mut commander, &mut ship, 3));
        assert_eq!(ship.fuel_current, 3);
        assert_eq!(commander.credits(), 96);
    }

    #[test]
    fn requested_units_are_capped_at_capacity() {
        let (mut commander, mut ship) = low_on_fuel(100, 6);
        assert!(handle(&args("50"), &mut commander, &mut ship, 3));
        assert_eq!(ship.fuel_current, 7);
        assert_eq!(commander.credits(), 98);
    }

    #[test]
    fn full_tank_is_refused_and_free() {
        let (mut commander, mut ship) = low_on_fuel(100, 7);
        assert!(!handle(&[], &mut commander, &mut ship, 3));
        assert_eq!(commander.credits(), 100);
    }

    #[test]
    fn no_credits_leaves_the_tank_alone() {
        let (mut commander, mut ship) = low_on_fuel(1, 4);
        assert!(!handle(&[], &mut commander, &mut ship, 3));
        assert_eq!(ship.fuel_current, 4);
        assert_eq!(commander.credits(), 1);
    }

    #[test]
    fn stranded_ship_is_rescued_for_a_reputation_cost() {
        let (mut commander, mut ship) = low_on_fuel(1, 1);
        assert!(handle(&[], &mut commander, &mut ship, 3));
        assert_eq!(ship.fuel_current, 3);
        assert_eq!(commander.credits(), 1);
        assert_eq!(commander.reputation, -2);
    }

    #[test]
    fn rescue_is_not_given_when_the_player_can_still_afford_to_move() {
        let (mut commander, mut ship) = low_on_fuel(4, 1);
        assert!(handle(&[], &mut commander, &mut ship, 3));
        assert_eq!(ship.fuel_current, 3);
        assert_eq!(commander.credits(), 0);
        assert_eq!(commander.reputation, 0);
    }

    #[test]
    fn rescue_is_not_given_while_there_is_cargo_to_sell() {
        let (mut commander, mut ship) = low_on_fuel(1, 1);
        ship.add_cargo(crate::data::commodities::CommodityType::Food, 2);
        assert!(!handle(&[], &mut commander, &mut ship, 3));
        assert_eq!(ship.fuel_current, 1);
        assert_eq!(commander.reputation, 0);
    }

    #[test]
    fn rescue_never_exceeds_the_tank() {
        let (mut commander, mut ship) = low_on_fuel(0, 0);
        assert!(handle(&[], &mut commander, &mut ship, 50));
        assert_eq!(ship.fuel_current, ship.fuel_capacity());
    }

    #[test]
    fn upgraded_ships_hold_more() {
        let (mut commander, mut ship) = low_on_fuel(1000, 0);
        ship.ship_type = crate::data::ships::ShipType::Python;
        assert!(handle(&[], &mut commander, &mut ship, 3));
        assert_eq!(ship.fuel_current, 12);
        assert_eq!(commander.credits(), 976);
    }

    #[test]
    fn invalid_arguments_change_nothing() {
        for bad in ["abc", "0", "-5", "5 6"] {
            let (mut commander, mut ship) = low_on_fuel(100, 1);
            assert!(!handle(&args(bad), &mut commander, &mut ship, 3));
            assert_eq!(ship.fuel_current, 1);
            assert_eq!(commander.credits(), 100);
        }
    }
}
