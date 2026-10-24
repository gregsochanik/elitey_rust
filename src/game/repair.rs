use crate::game::{Commander, Ship};
use crate::utils::display::{show_error, show_info, show_success};

pub const CREDITS_PER_POINT: u32 = 2;

pub fn handle(args: &[String], commander: &mut Commander, ship: &mut Ship) -> bool {
    let requested = match args.first() {
        None => None,
        Some(arg) => match arg.parse::<u32>() {
            Ok(points) if points > 0 && args.len() == 1 => Some(points),
            _ => {
                show_error("Usage: repair [points]");
                return false;
            }
        },
    };

    let damage = ship.damage();
    if damage == 0 {
        show_info("Your hull is already at full strength.");
        return false;
    }

    let affordable = commander.credits() / CREDITS_PER_POINT;
    if affordable == 0 {
        show_error(&format!(
            "Repairs cost {} credits per hull point; you can't afford any.",
            CREDITS_PER_POINT
        ));
        return false;
    }

    let points = requested.unwrap_or(damage).min(damage).min(affordable);
    commander.remove_credits(points * CREDITS_PER_POINT);
    ship.repair_hull(points);
    show_success(&format!(
        "Repaired {} hull points for {} credits. Hull: {}",
        points,
        points * CREDITS_PER_POINT,
        ship.hull_display()
    ));
    true
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

    fn args(text: &str) -> Vec<String> {
        text.split_whitespace().map(String::from).collect()
    }

    #[test]
    fn repairs_to_full_at_two_credits_per_point() {
        let (mut commander, mut ship) = damaged(500, 70);
        assert!(handle(&[], &mut commander, &mut ship));
        assert_eq!(ship.hull_percentage(), 100);
        assert_eq!(commander.credits(), 440);
    }

    #[test]
    fn repairs_as_much_as_affordable() {
        let (mut commander, mut ship) = damaged(21, 50);
        assert!(handle(&[], &mut commander, &mut ship));
        assert_eq!(ship.hull_percentage(), 60);
        assert_eq!(commander.credits(), 1);
    }

    #[test]
    fn repairs_only_the_requested_points() {
        let (mut commander, mut ship) = damaged(500, 50);
        assert!(handle(&args("10"), &mut commander, &mut ship));
        assert_eq!(ship.hull_percentage(), 60);
        assert_eq!(commander.credits(), 480);
    }

    #[test]
    fn requested_points_are_capped_at_the_damage() {
        let (mut commander, mut ship) = damaged(500, 90);
        assert!(handle(&args("50"), &mut commander, &mut ship));
        assert_eq!(ship.hull_percentage(), 100);
        assert_eq!(commander.credits(), 480);
    }

    #[test]
    fn full_hull_is_refused_and_free() {
        let (mut commander, mut ship) = damaged(500, 100);
        assert!(!handle(&[], &mut commander, &mut ship));
        assert_eq!(commander.credits(), 500);
    }

    #[test]
    fn no_credits_leaves_the_hull_alone() {
        let (mut commander, mut ship) = damaged(1, 50);
        assert!(!handle(&[], &mut commander, &mut ship));
        assert_eq!(ship.hull_percentage(), 50);
        assert_eq!(commander.credits(), 1);
    }

    #[test]
    fn upgraded_ships_repair_up_to_their_own_maximum() {
        let (mut commander, mut ship) = damaged(1000, 100);
        ship.ship_type = crate::data::ships::ShipType::Python;
        assert!(handle(&[], &mut commander, &mut ship));
        assert_eq!(ship.hull_percentage(), 150);
        assert_eq!(commander.credits(), 900);
    }

    #[test]
    fn invalid_arguments_change_nothing() {
        for bad in ["abc", "0", "-5", "5 6"] {
            let (mut commander, mut ship) = damaged(500, 50);
            assert!(!handle(&args(bad), &mut commander, &mut ship));
            assert_eq!(ship.hull_percentage(), 50);
            assert_eq!(commander.credits(), 500);
        }
    }
}
