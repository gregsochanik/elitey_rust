use crate::data::commodities::{CommodityType, TradingSystem};
use crate::game::combat::Outcome;
use crate::game::commander::{CombatRating, ReputationChange};
use crate::game::events::{apply_cargo_gain, apply_cargo_loss, CargoGain, CargoLoss, EventType};
use crate::game::{Commander, Ship};
use rand::Rng;

pub const VICTORY_REWARD: std::ops::RangeInclusive<u32> = 100..=200;
const DEFEAT_HULL_PERCENT: u32 = 10;
const SURRENDER_CARGO_TONS: u32 = 5;
const SURRENDER_CREDITS: u32 = 200;

pub fn push_promotion_message(messages: &mut Vec<String>, promotion: Option<CombatRating>) {
    if let Some(rating) = promotion {
        messages.push(format!("Your combat rating is now {}.", rating.name()));
    }
}

fn push_reputation_messages(messages: &mut Vec<String>, change: ReputationChange) {
    if change.applied != 0 {
        messages.push(format!("Reputation {:+}.", change.applied));
    }
    if let Some(standing) = change.new_standing {
        messages.push(format!("Your standing is now {}.", standing.name()));
    }
}

/// Who the player fought, which decides how the fight is settled
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Foe {
    Pirate,
    Police,
    Trader,
}

impl Foe {
    pub fn of_event(event_type: &EventType) -> Self {
        match event_type {
            EventType::MilitaryPatrol => Foe::Police,
            EventType::TraderConvoy => Foe::Trader,
            _ => Foe::Pirate,
        }
    }

    /// The entry in `data/enemies.json` that plays this foe
    pub fn enemy_id(self) -> &'static str {
        match self {
            Foe::Pirate => "pirate-raider",
            Foe::Police => "patrol-vessel",
            Foe::Trader => "trader",
        }
    }
}

const POLICE_FINE_PERCENT: u32 = 25;
const TRADER_LOOT_CREDITS: std::ops::RangeInclusive<u32> = 60..=120;
const TRADER_PIRACY_REPUTATION: i32 = -10;

/// Settle a fight against `foe`: pirates as `settle`; police fine and confiscate instead of
/// robbing; traders give up goods, and robbing them costs reputation and earns no kill.
pub fn settle_against(
    foe: Foe,
    outcome: Outcome,
    player_hull: u32,
    commander: &mut Commander,
    ship: &mut Ship,
    rng: &mut impl Rng,
) -> Vec<String> {
    match (foe, outcome) {
        (Foe::Police, Outcome::Defeat) => {
            ship.hull_percentage = DEFEAT_HULL_PERCENT * ship.ship_type.max_hull() / 100;
            let mut messages = vec![];
            confiscate_and_fine(commander, ship, POLICE_FINE_PERCENT, &mut messages);
            messages
        }
        (Foe::Police, Outcome::Surrendered) => {
            ship.hull_percentage = player_hull.min(ship.ship_type.max_hull());
            let mut messages = vec![];
            confiscate_and_fine(commander, ship, POLICE_FINE_PERCENT, &mut messages);
            messages
        }
        (Foe::Trader, Outcome::Victory | Outcome::EnemySurrendered) => {
            ship.hull_percentage = player_hull.min(ship.ship_type.max_hull());
            let credits = rng.gen_range(TRADER_LOOT_CREDITS);
            commander.add_credits(credits);
            let mut messages = vec![format!("You take {} credits from the convoy.", credits)];
            let commodities = TradingSystem::new().get_commodities().to_vec();
            let loot = CargoGain {
                commodity: commodities[rng.gen_range(0..commodities.len())].clone(),
                tons: rng.gen_range(2..=4),
            };
            match apply_cargo_gain(ship, &loot) {
                0 => messages.push("There is no room in your hold for the goods.".to_string()),
                tons => messages.push(format!(
                    "You seize {} tons of {}.",
                    tons,
                    loot.commodity.name()
                )),
            }
            push_reputation_messages(
                &mut messages,
                commander.change_reputation(TRADER_PIRACY_REPUTATION),
            );
            messages
        }
        (Foe::Trader, Outcome::Surrendered) => {
            ship.hull_percentage = player_hull.min(ship.ship_type.max_hull());
            vec!["The convoy lets you go.".to_string()]
        }
        _ => settle(outcome, player_hull, commander, ship, rng),
    }
}

fn confiscate_and_fine(
    commander: &mut Commander,
    ship: &mut Ship,
    percent: u32,
    messages: &mut Vec<String>,
) {
    let taken = apply_cargo_loss(ship, &CargoLoss::All(CommodityType::Narcotics));
    let tons: u32 = taken.iter().map(|(_, tons)| tons).sum();
    if tons > 0 {
        messages.push(format!(
            "The patrol confiscates {} tons of narcotics.",
            tons
        ));
    }
    let fine = commander.credits() * percent / 100;
    commander.remove_credits(fine);
    messages.push(format!("They fine you {} credits.", fine));
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
    settle_with_reward(outcome, player_hull, commander, ship, rng, None)
}

/// Like `settle`, but a victory pays `victory_reward` instead of the random payout
pub fn settle_with_reward(
    outcome: Outcome,
    player_hull: u32,
    commander: &mut Commander,
    ship: &mut Ship,
    rng: &mut impl Rng,
    victory_reward: Option<u32>,
) -> Vec<String> {
    ship.hull_percentage = player_hull.min(ship.ship_type.max_hull());
    let mut messages = Vec::new();

    match outcome {
        Outcome::Victory | Outcome::EnemySurrendered => {
            let surrendered = outcome == Outcome::EnemySurrendered;
            let mut reward = victory_reward.unwrap_or_else(|| rng.gen_range(VICTORY_REWARD));
            if surrendered && victory_reward.is_none() {
                reward /= 2;
            }
            commander.add_credits(reward);
            let reputation = commander.change_reputation(5);
            messages.push(if surrendered {
                format!("They pay {} credits for their lives.", reward)
            } else {
                format!("You claim {} credits from the wreck.", reward)
            });

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
            push_promotion_message(&mut messages, commander.record_kill());
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
        Outcome::Escaped | Outcome::EnemyFled | Outcome::Ongoing => {}
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
    fn losing_to_the_police_costs_a_fine_and_the_narcotics_but_not_the_rest() {
        let (mut commander, mut ship) = setup(
            1_000,
            &[(CommodityType::Narcotics, 3), (CommodityType::Food, 4)],
        );

        let messages = settle_against(
            Foe::Police,
            Outcome::Defeat,
            0,
            &mut commander,
            &mut ship,
            &mut rng(),
        );

        assert_eq!(commander.credits(), 750);
        assert_eq!(ship.cargo_hold.get_quantity(&CommodityType::Narcotics), 0);
        assert_eq!(ship.cargo_hold.get_quantity(&CommodityType::Food), 4);
        assert_eq!(commander.reputation, 0);
        assert!(messages.iter().any(|m| m.contains("fine")));
    }

    #[test]
    fn robbing_a_convoy_pays_but_costs_reputation_and_earns_no_kill() {
        for seed in 0..30 {
            let (mut commander, mut ship) = setup(100, &[]);
            settle_against(
                Foe::Trader,
                Outcome::Victory,
                90,
                &mut commander,
                &mut ship,
                &mut StdRng::seed_from_u64(seed),
            );

            assert!((60..=120).contains(&(commander.credits() - 100)));
            assert!((2..=4).contains(&ship.cargo_used()));
            assert_eq!(commander.reputation, -10);
            assert_eq!(commander.kills, 0);
        }
    }

    #[test]
    fn a_pirate_fight_settles_as_before() {
        let (mut a, mut ship_a) = setup(100, &[]);
        let (mut b, mut ship_b) = setup(100, &[]);

        let direct = settle(Outcome::Defeat, 0, &mut a, &mut ship_a, &mut rng());
        let against = settle_against(
            Foe::Pirate,
            Outcome::Defeat,
            0,
            &mut b,
            &mut ship_b,
            &mut rng(),
        );

        assert_eq!(direct, against);
    }

    #[test]
    fn a_surrendering_enemy_pays_half_and_counts_as_a_kill() {
        for seed in 0..50 {
            let (mut commander, mut ship) = setup(100, &[]);
            settle(
                Outcome::EnemySurrendered,
                80,
                &mut commander,
                &mut ship,
                &mut StdRng::seed_from_u64(seed),
            );

            let reward = commander.credits() - 100;
            assert!(
                (VICTORY_REWARD.start() / 2..=VICTORY_REWARD.end() / 2).contains(&reward),
                "{}",
                reward
            );
            assert_eq!(commander.kills, 1);
            assert_eq!(commander.reputation, 5);
        }
    }

    #[test]
    fn an_enemy_that_flees_changes_nothing_but_the_hull() {
        let (mut commander, mut ship) = setup(100, &[]);

        let messages = settle(
            Outcome::EnemyFled,
            65,
            &mut commander,
            &mut ship,
            &mut rng(),
        );

        assert!(messages.is_empty());
        assert_eq!(commander.credits(), 100);
        assert_eq!(commander.kills, 0);
        assert_eq!(ship.hull_percentage, 65);
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
        ship.ship_type = crate::data::ships::ShipType::from_string("python").unwrap();
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

    #[test]
    fn defeat_also_strips_contract_cargo() {
        let (mut commander, mut ship) = setup(100, &[(CommodityType::Food, 3)]);
        assert!(ship.cargo_hold.reserve(8));

        settle(Outcome::Defeat, 0, &mut commander, &mut ship, &mut rng());

        assert_eq!(ship.cargo_used(), 0);
        assert_eq!(ship.cargo_hold.reserved, 0);
    }

    #[test]
    fn only_victories_count_as_kills() {
        for outcome in [Outcome::Escaped, Outcome::Surrendered, Outcome::Defeat] {
            let (mut commander, mut ship) = setup(100, &[]);
            settle(outcome, 50, &mut commander, &mut ship, &mut rng());
            assert_eq!(commander.kills, 0, "{outcome:?}");
        }

        let (mut commander, mut ship) = setup(100, &[]);
        settle(Outcome::Victory, 50, &mut commander, &mut ship, &mut rng());
        assert_eq!(commander.kills, 1);
    }

    #[test]
    fn a_promotion_is_announced_after_the_fight() {
        let (mut commander, mut ship) = setup(100, &[]);
        commander.kills = 2;

        let messages = settle(Outcome::Victory, 50, &mut commander, &mut ship, &mut rng());

        assert!(messages.contains(&"Your combat rating is now Mostly Harmless.".to_string()));
    }
}
