use crate::data::commodities::TradingSystem;
use crate::game::combat::Outcome;
use crate::game::commander::ReputationChange;
use crate::game::events::{apply_cargo_gain, apply_cargo_loss, CargoGain, CargoLoss};
use crate::game::{Commander, Ship};
use rand::Rng;

pub const VICTORY_REWARD: std::ops::RangeInclusive<u32> = 100..=200;
const DEFEAT_HULL_PERCENT: u32 = 10;
const SURRENDER_CARGO_TONS: u32 = 5;
const SURRENDER_CREDITS: u32 = 200;

fn push_reputation_messages(messages: &mut Vec<String>, change: ReputationChange) {
    if change.applied != 0 {
        messages.push(format!("Reputation {:+}.", change.applied));
    }
    if let Some(standing) = change.new_standing {
        messages.push(format!("Your standing is now {}.", standing.name()));
    }
}

/// Apply the consequences of a finished fight and return what to tell the player.
/// `player_hull` is the hull the player's fighter ended with.
pub fn settle(
    outcome: Outcome,
    player_hull: u32,
    commander: &mut Commander,
    ship: &mut Ship,
    rng: &mut impl Rng,
) -> Vec<String> {
    ship.hull_percentage = player_hull.min(ship.ship_type.max_hull());
    let mut messages = Vec::new();

    match outcome {
        Outcome::Victory => {
            let reward = rng.gen_range(VICTORY_REWARD);
            commander.add_credits(reward);
            let reputation = commander.change_reputation(5);
            messages.push(format!("You claim {} credits from the wreck.", reward));

            let commodities = TradingSystem::new().get_commodities().to_vec();
            let loot = CargoGain {
                commodity: commodities[rng.gen_range(0..commodities.len())].clone(),
                tons: rng.gen_range(1..=3),
            };
            match apply_cargo_gain(ship, &loot) {
                0 => messages.push("There is no room in your hold for the loot.".to_string()),
                tons => messages.push(format!(
                    "You salvage {} tons of {}.",
                    tons,
                    loot.commodity.name()
                )),
            }
            push_reputation_messages(&mut messages, reputation);
        }
        Outcome::Defeat => {
            let held = ship.cargo_hold.current_weight();
            if held > 0 {
                apply_cargo_loss(ship, &CargoLoss::Tons(held));
                messages.push(format!("The pirates strip your hold of {} tons.", held));
            }
            let lost = commander.credits() / 2;
            commander.remove_credits(lost);
            let reputation = commander.change_reputation(-5);
            ship.hull_percentage = ship.ship_type.max_hull() * DEFEAT_HULL_PERCENT / 100;
            messages.push(format!(
                "They take {} credits and leave you adrift with {} hull.",
                lost,
                ship.hull_display()
            ));
            push_reputation_messages(&mut messages, reputation);
        }
        Outcome::Surrendered => {
            let taken = apply_cargo_loss(ship, &CargoLoss::Tons(SURRENDER_CARGO_TONS));
            let tons: u32 = taken.iter().map(|(_, tons)| tons).sum();
            let credits = SURRENDER_CREDITS.min(commander.credits());
            commander.remove_credits(credits);
            messages.push(format!(
                "The pirates take {} tons of cargo and {} credits.",
                tons, credits
            ));
        }
        Outcome::Escaped | Outcome::Ongoing => {}
    }

    messages
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::commodities::CommodityType;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    fn setup(credits: u32, cargo: &[(CommodityType, u32)]) -> (Commander, Ship) {
        let mut commander = Commander::new("Test".to_string());
        commander.credits = credits;
        let mut ship = Ship::new_cobra_mk3();
        for (commodity, tons) in cargo {
            assert!(ship.cargo_hold.add_cargo(commodity.clone(), *tons));
        }
        (commander, ship)
    }

    fn rng() -> StdRng {
        StdRng::seed_from_u64(11)
    }

    #[test]
    fn victory_pays_credits_loot_and_reputation() {
        for seed in 0..50 {
            let (mut commander, mut ship) = setup(100, &[]);
            settle(
                Outcome::Victory,
                80,
                &mut commander,
                &mut ship,
                &mut StdRng::seed_from_u64(seed),
            );

            let reward = commander.credits() - 100;
            assert!(VICTORY_REWARD.contains(&reward), "{}", reward);
            assert!(
                (1..=3).contains(&ship.cargo_used()),
                "{}",
                ship.cargo_used()
            );
            assert_eq!(commander.reputation, 5);
            assert_eq!(ship.hull_percentage, 80);
        }
    }

    #[test]
    fn victory_loot_is_limited_by_free_space() {
        let (mut commander, mut ship) = setup(0, &[(CommodityType::Food, 20)]);

        let messages = settle(Outcome::Victory, 100, &mut commander, &mut ship, &mut rng());

        assert_eq!(ship.cargo_used(), 20);
        assert!(messages.iter().any(|m| m.contains("no room")));
    }

    #[test]
    fn defeat_strips_cargo_halves_credits_and_leaves_ten_percent_hull() {
        let (mut commander, mut ship) =
            setup(501, &[(CommodityType::Food, 6), (CommodityType::Alloys, 4)]);

        settle(Outcome::Defeat, 0, &mut commander, &mut ship, &mut rng());

        assert_eq!(ship.cargo_used(), 0);
        assert_eq!(commander.credits(), 251);
        assert_eq!(ship.hull_percentage, 10);
        assert_eq!(commander.reputation, -5);
    }

    #[test]
    fn defeat_leaves_ten_percent_of_an_upgraded_ships_hull() {
        let (mut commander, mut ship) = setup(100, &[]);
        ship.ship_type = crate::data::ships::ShipType::Python;
        ship.hull_percentage = 150;

        let messages = settle(Outcome::Defeat, 0, &mut commander, &mut ship, &mut rng());

        assert_eq!(ship.hull_percentage, 15);
        assert!(messages.iter().any(|m| m.contains("15/150")));
    }

    #[test]
    fn reputation_message_reports_what_was_actually_applied() {
        let (mut commander, mut ship) = setup(0, &[]);
        commander.reputation = 98;

        let messages = settle(Outcome::Victory, 50, &mut commander, &mut ship, &mut rng());
        assert_eq!(commander.reputation, 100);
        assert!(messages.iter().any(|m| m == "Reputation +2."));

        let messages = settle(Outcome::Victory, 50, &mut commander, &mut ship, &mut rng());
        assert!(!messages.iter().any(|m| m.starts_with("Reputation")));
    }

    #[test]
    fn defeat_with_an_empty_hold_and_no_credits_is_survivable() {
        let (mut commander, mut ship) = setup(0, &[]);

        settle(Outcome::Defeat, 0, &mut commander, &mut ship, &mut rng());

        assert_eq!(commander.credits(), 0);
        assert_eq!(ship.hull_percentage, 10);
    }

    #[test]
    fn surrender_costs_up_to_five_tons_and_two_hundred_credits_without_hull_damage() {
        let (mut commander, mut ship) = setup(500, &[(CommodityType::Computers, 8)]);

        settle(
            Outcome::Surrendered,
            100,
            &mut commander,
            &mut ship,
            &mut rng(),
        );

        assert_eq!(ship.cargo_hold.get_quantity(&CommodityType::Computers), 3);
        assert_eq!(commander.credits(), 300);
        assert_eq!(ship.hull_percentage, 100);
        assert_eq!(commander.reputation, 0);
    }

    #[test]
    fn surrender_with_little_to_lose_takes_what_there_is() {
        let (mut commander, mut ship) = setup(50, &[(CommodityType::Food, 2)]);

        settle(
            Outcome::Surrendered,
            100,
            &mut commander,
            &mut ship,
            &mut rng(),
        );

        assert_eq!(ship.cargo_used(), 0);
        assert_eq!(commander.credits(), 0);
    }

    #[test]
    fn escaping_costs_nothing_but_keeps_the_hull_damage() {
        let (mut commander, mut ship) = setup(500, &[(CommodityType::Food, 4)]);

        settle(Outcome::Escaped, 65, &mut commander, &mut ship, &mut rng());

        assert_eq!(commander.credits(), 500);
        assert_eq!(ship.cargo_used(), 4);
        assert_eq!(ship.hull_percentage, 65);
    }
}
