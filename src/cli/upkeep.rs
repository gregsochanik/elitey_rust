use crate::game::refuel::{RefuelError, RefuelOutcome};
use crate::game::GameState;
use crate::utils::display::{show_error, show_info, show_success};

/// A single optional positive number: `None` for no argument, `Err` for anything else
fn parse_amount(args: &[String]) -> Result<Option<u32>, ()> {
    match args.first() {
        None => Ok(None),
        Some(arg) => match arg.parse::<u32>() {
            Ok(amount) if amount > 0 && args.len() == 1 => Ok(Some(amount)),
            _ => Err(()),
        },
    }
}

/// `refuel [units]`
pub fn refuel(game: &mut GameState, args: &[String]) -> bool {
    let Ok(units) = parse_amount(args) else {
        show_error("Usage: refuel [units]");
        return false;
    };
    match game.refuel(units) {
        Ok(RefuelOutcome::Bought {
            units,
            cost,
            fuel,
            capacity,
        }) => {
            show_success(&format!(
                "Bought {} light years of fuel for {} credits. Fuel: {}/{}",
                units, cost, fuel, capacity
            ));
            true
        }
        Ok(RefuelOutcome::Rescued {
            fuel,
            capacity,
            change,
        }) => {
            show_info(&format!(
                "Stranded! A rescue crew tops up your tank to {}/{} and files a report. Reputation {:+}.",
                fuel, capacity, change.applied
            ));
            if let Some(standing) = change.new_standing {
                show_info(&format!("Your standing is now {}.", standing.name()));
            }
            true
        }
        Err(RefuelError::TankFull) => {
            show_info(&RefuelError::TankFull.to_string());
            false
        }
        Err(error @ RefuelError::CannotAfford { stranded }) => {
            show_error(&error.to_string());
            if stranded {
                show_info("You are stranded: run 'refuel' with no amount to call a rescue crew.");
            }
            false
        }
    }
}

/// `repair [points]`
pub fn repair(game: &mut GameState, args: &[String]) -> bool {
    let Ok(points) = parse_amount(args) else {
        show_error("Usage: repair [points]");
        return false;
    };
    match game.repair(points) {
        Ok(receipt) => {
            show_success(&format!(
                "Repaired {} hull points for {} credits. Hull: {}/{}",
                receipt.points, receipt.cost, receipt.hull, receipt.max_hull
            ));
            true
        }
        Err(error @ crate::game::repair::RepairError::HullFull) => {
            show_info(&error.to_string());
            false
        }
        Err(error) => {
            show_error(&error.to_string());
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(text: &str) -> Vec<String> {
        text.split_whitespace().map(String::from).collect()
    }

    #[test]
    fn no_argument_means_everything() {
        assert_eq!(parse_amount(&[]), Ok(None));
    }

    #[test]
    fn a_positive_number_is_an_amount() {
        assert_eq!(parse_amount(&args("12")), Ok(Some(12)));
    }

    #[test]
    fn anything_else_is_refused() {
        for bad in ["abc", "0", "-5", "5 6"] {
            assert_eq!(parse_amount(&args(bad)), Err(()), "{bad}");
        }
    }
}
