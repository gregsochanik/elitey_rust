use super::*;
use crate::data::systems::{EconomyType, GovernmentType, StarSystem};

#[test]
fn test_event_manager_creation() {
    let manager = EventManager::new();
    assert_eq!(manager.events.len(), 10); // 10 event types
}

#[test]
fn test_phase_event_filtering() {
    let manager = EventManager::new();

    let jump_events = manager.get_available_events(&EventPhase::Jump);
    let approach_events = manager.get_available_events(&EventPhase::Approach);

    assert_eq!(jump_events.len(), 4); // 4 jump events
    assert_eq!(approach_events.len(), 6); // 6 approach events
}

#[test]
fn test_contextual_modifier_calculation() {
    let manager = EventManager::new();

    // Test anarchy system (should have higher modifier)
    let anarchy_system = StarSystem {
        name: "Test".to_string(),
        x: 0,
        y: 0,
        economy: EconomyType::MainlyAgricultural,
        government: GovernmentType::Anarchy,
        tech_level: 5,
        turnover: 1000,
        radius: 100,
        population: "1000000".to_string(),
        description: "Test system".to_string(),
        star_class: String::new(),
    };

    let modifier = manager.calculate_contextual_modifier(&anarchy_system);
    assert!(modifier > 1.0); // Should be higher than base
}

#[test]
fn test_event_processing() {
    let manager = EventManager::new();
    let mut commander = Commander::new("Test".to_string());
    let mut ship = Ship::new_cobra_mk3();

    let event = manager.events.get(&EventType::Wormhole).unwrap();
    let initial_credits = commander.credits;
    let initial_fuel = ship.fuel_current;

    let result = manager.process_event(event, &mut commander, &mut ship, 0);
    assert!(result.is_ok());

    // Check that effects were applied
    assert_eq!(commander.credits, initial_credits);
    assert_eq!(ship.fuel_current, initial_fuel - 2);
}

#[test]
fn event_outcome_changes_commander_reputation() {
    let manager = EventManager::new();
    let mut commander = Commander::new("Test".to_string());
    let mut ship = Ship::new_cobra_mk3();

    let event = manager.events.get(&EventType::AlienEncounter).unwrap();
    manager
        .process_event(event, &mut commander, &mut ship, 0)
        .unwrap();

    assert_eq!(commander.reputation, 15);
}

#[test]
fn reputation_can_go_negative() {
    let manager = EventManager::new();
    let mut commander = Commander::new("Test".to_string());
    let mut ship = Ship::new_cobra_mk3();

    let event = manager.events.get(&EventType::MilitaryPatrol).unwrap();
    for index in 0..event.choices.len() {
        let before = commander.reputation;
        manager
            .process_event(event, &mut commander, &mut ship, index)
            .unwrap();
        if commander.reputation < before {
            return;
        }
    }
    panic!("no military patrol choice lowers reputation");
}

fn ship_with(cargo: &[(CommodityType, u32)]) -> Ship {
    let mut ship = Ship::new_cobra_mk3();
    for (commodity, tons) in cargo {
        assert!(ship.cargo_hold.add_cargo(commodity.clone(), *tons));
    }
    ship
}

#[test]
fn losing_tons_takes_from_the_largest_cargo_first() {
    let mut ship = ship_with(&[(CommodityType::Food, 2), (CommodityType::Minerals, 8)]);

    let taken = apply_cargo_loss(&mut ship, &CargoLoss::Tons(5));

    assert_eq!(taken, vec![(CommodityType::Minerals, 5)]);
    assert_eq!(ship.cargo_hold.get_quantity(&CommodityType::Minerals), 3);
    assert_eq!(ship.cargo_hold.get_quantity(&CommodityType::Food), 2);
}

#[test]
fn losing_more_than_held_takes_everything_across_commodities() {
    let mut ship = ship_with(&[(CommodityType::Food, 2), (CommodityType::Minerals, 3)]);

    let taken = apply_cargo_loss(&mut ship, &CargoLoss::Tons(10));

    assert_eq!(taken.iter().map(|(_, tons)| tons).sum::<u32>(), 5);
    assert!(ship.cargo_hold.is_empty());
}

#[test]
fn losing_cargo_from_an_empty_hold_takes_nothing() {
    let mut ship = ship_with(&[]);
    assert!(apply_cargo_loss(&mut ship, &CargoLoss::Tons(3)).is_empty());
    assert!(apply_cargo_loss(&mut ship, &CargoLoss::All(CommodityType::Narcotics)).is_empty());
}

#[test]
fn losing_all_of_a_commodity_leaves_other_cargo() {
    let mut ship = ship_with(&[(CommodityType::Narcotics, 4), (CommodityType::Food, 2)]);

    let taken = apply_cargo_loss(&mut ship, &CargoLoss::All(CommodityType::Narcotics));

    assert_eq!(taken, vec![(CommodityType::Narcotics, 4)]);
    assert_eq!(ship.cargo_hold.get_quantity(&CommodityType::Narcotics), 0);
    assert_eq!(ship.cargo_hold.get_quantity(&CommodityType::Food), 2);
}

#[test]
fn cargo_gain_is_limited_by_free_space() {
    let mut ship = ship_with(&[(CommodityType::Food, 19)]);
    let gain = CargoGain {
        commodity: CommodityType::Luxury,
        tons: 2,
    };

    assert_eq!(apply_cargo_gain(&mut ship, &gain), 1);
    assert_eq!(ship.cargo_hold.get_quantity(&CommodityType::Luxury), 1);
    assert_eq!(apply_cargo_gain(&mut ship, &gain), 0);
}

#[test]
fn surrendering_to_pirates_costs_cargo_and_credits() {
    let manager = EventManager::new();
    let mut commander = Commander::new("Test".to_string());
    commander.credits = 500;
    let mut ship = ship_with(&[(CommodityType::Computers, 8)]);

    let event = manager.events.get(&EventType::PirateAttack).unwrap();
    let surrender = event
        .choices
        .iter()
        .position(|c| c.description == "Surrender")
        .unwrap();
    let outcome = manager
        .process_event(event, &mut commander, &mut ship, surrender)
        .unwrap();

    let Some(CargoLoss::Tons(tons)) = outcome.cargo_loss else {
        panic!("surrender loses cargo");
    };
    assert_eq!(
        ship.cargo_hold.get_quantity(&CommodityType::Computers),
        8 - tons
    );
    assert_eq!(commander.credits as i32, 500 + outcome.credits_change);
    assert!(outcome.credits_change < 0);
}

#[test]
fn resisting_inspection_confiscates_narcotics_only() {
    let manager = EventManager::new();
    let mut commander = Commander::new("Test".to_string());
    let mut ship = ship_with(&[(CommodityType::Narcotics, 3), (CommodityType::Food, 5)]);

    let event = manager
        .events
        .get(&EventType::GovernmentInspection)
        .unwrap();
    let resist = event
        .choices
        .iter()
        .position(|c| c.description.starts_with("Resist"))
        .unwrap();
    manager
        .process_event(event, &mut commander, &mut ship, resist)
        .unwrap();

    assert_eq!(ship.cargo_hold.get_quantity(&CommodityType::Narcotics), 0);
    assert_eq!(ship.cargo_hold.get_quantity(&CommodityType::Food), 5);
}

#[test]
fn rescuing_the_crew_rewards_cargo() {
    let manager = EventManager::new();
    let mut commander = Commander::new("Test".to_string());
    let mut ship = ship_with(&[]);

    let event = manager.events.get(&EventType::RescueSignal).unwrap();
    let outcome = manager
        .process_event(event, &mut commander, &mut ship, 0)
        .unwrap();

    let gained = outcome.cargo_gain.unwrap().tons;
    assert_eq!(ship.cargo_hold.get_quantity(&CommodityType::Luxury), gained);
}

fn system(government: GovernmentType, economy: EconomyType) -> StarSystem {
    StarSystem {
        name: "Test".to_string(),
        x: 0,
        y: 0,
        economy,
        government,
        tech_level: 5,
        turnover: 1000,
        radius: 100,
        population: "1000000".to_string(),
        description: "Test system".to_string(),
        star_class: String::new(),
    }
}

fn share_of(event_type: EventType, phase: EventPhase, destination: &StarSystem) -> f64 {
    share_of_for(event_type, phase, destination, Standing::Neutral)
}

fn share_of_for(
    event_type: EventType,
    phase: EventPhase,
    destination: &StarSystem,
    standing: Standing,
) -> f64 {
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    let manager = EventManager::new();
    let mut rng = StdRng::seed_from_u64(7);
    let draws = 20_000;
    let hits = (0..draws)
        .filter(|_| {
            manager
                .pick_event(&phase, destination, standing, &mut rng)
                .unwrap()
                .event_type
                == event_type
        })
        .count();
    hits as f64 / f64::from(draws)
}

#[test]
fn anarchy_attracts_pirates_more_than_democracy() {
    let anarchy = system(GovernmentType::Anarchy, EconomyType::AverageAgricultural);
    let democracy = system(GovernmentType::Democracy, EconomyType::AverageAgricultural);

    let lawless = share_of(EventType::PirateAttack, EventPhase::Approach, &anarchy);
    let ordered = share_of(EventType::PirateAttack, EventPhase::Approach, &democracy);

    assert!(lawless > ordered * 1.8, "{} vs {}", lawless, ordered);
}

#[test]
fn communist_systems_inspect_more_often() {
    let communist = system(GovernmentType::Communist, EconomyType::AverageAgricultural);
    let democracy = system(GovernmentType::Democracy, EconomyType::AverageAgricultural);

    let strict = share_of(
        EventType::GovernmentInspection,
        EventPhase::Approach,
        &communist,
    );
    let normal = share_of(
        EventType::GovernmentInspection,
        EventPhase::Approach,
        &democracy,
    );

    assert!(strict > normal * 1.8, "{} vs {}", strict, normal);
}

#[test]
fn dictatorships_send_more_patrols() {
    let dictatorship = system(
        GovernmentType::Dictatorship,
        EconomyType::AverageAgricultural,
    );
    let democracy = system(GovernmentType::Democracy, EconomyType::AverageAgricultural);

    let patrolled = share_of(
        EventType::MilitaryPatrol,
        EventPhase::Approach,
        &dictatorship,
    );
    let normal = share_of(EventType::MilitaryPatrol, EventPhase::Approach, &democracy);

    assert!(patrolled > normal * 1.8, "{} vs {}", patrolled, normal);
}

#[test]
fn poor_economies_have_more_pirates_and_rich_ones_fewer() {
    let poor = system(GovernmentType::Democracy, EconomyType::PoorAgricultural);
    let average = system(GovernmentType::Democracy, EconomyType::AverageAgricultural);
    let rich = system(GovernmentType::Democracy, EconomyType::RichIndustrial);

    let poor_share = share_of(EventType::PirateAttack, EventPhase::Approach, &poor);
    let average_share = share_of(EventType::PirateAttack, EventPhase::Approach, &average);
    let rich_share = share_of(EventType::PirateAttack, EventPhase::Approach, &rich);

    assert!(poor_share > average_share && average_share > rich_share);
}

#[test]
fn mainly_industrial_systems_have_more_debris() {
    let industrial = system(GovernmentType::Democracy, EconomyType::MainlyIndustrial);
    let average = system(GovernmentType::Democracy, EconomyType::AverageAgricultural);

    let debris = share_of(EventType::SpaceDebris, EventPhase::Jump, &industrial);
    let normal = share_of(EventType::SpaceDebris, EventPhase::Jump, &average);

    assert!(debris > normal);
}

#[test]
fn every_event_stays_possible_everywhere() {
    let manager = EventManager::new();
    for government in [
        GovernmentType::Anarchy,
        GovernmentType::Communist,
        GovernmentType::CorporateState,
        GovernmentType::Democracy,
        GovernmentType::Dictatorship,
        GovernmentType::Feudal,
        GovernmentType::MultiGovernment,
        GovernmentType::Confederacy,
    ] {
        let destination = system(government, EconomyType::RichIndustrial);
        for phase in [EventPhase::Jump, EventPhase::Approach] {
            for event in manager.get_available_events(&phase) {
                assert!(event_weight(&event.event_type, &destination, Standing::Neutral) > 0.0);
            }
        }
    }
}

#[test]
fn picking_an_event_always_matches_the_phase() {
    let manager = EventManager::new();
    let destination = system(GovernmentType::Anarchy, EconomyType::PoorAgricultural);
    for _ in 0..200 {
        let event = manager
            .select_random_event(&EventPhase::Jump, &destination, Standing::Neutral)
            .unwrap();
        assert_eq!(event.phase, EventPhase::Jump);
    }
}

#[test]
fn only_fighting_pirates_patrols_and_convoys_starts_combat() {
    let manager = EventManager::new();
    let mut starters = Vec::new();
    for (event_type, event) in &manager.events {
        for choice in &event.choices {
            if choice.outcome.starts_combat {
                starters.push((event_type.clone(), choice.description.clone()));
            }
        }
    }

    starters.sort_by_key(|(event_type, _)| format!("{:?}", event_type));
    assert_eq!(
        starters,
        vec![
            (EventType::MilitaryPatrol, "Engage the patrol".to_string()),
            (EventType::PirateAttack, "Fight the pirates".to_string()),
            (EventType::TraderConvoy, "Rob the convoy".to_string()),
        ]
    );
}

#[test]
fn fighting_pirates_has_no_instant_effects() {
    let manager = EventManager::new();
    let mut commander = Commander::new("Test".to_string());
    let mut ship = ship_with(&[]);
    let event = manager.events.get(&EventType::PirateAttack).unwrap();

    let outcome = manager
        .process_event(event, &mut commander, &mut ship, 0)
        .unwrap();

    assert!(outcome.starts_combat);
    assert_eq!(commander.credits, 100);
    assert_eq!(commander.reputation, 0);
    assert_eq!(ship.hull_percentage, 100);
}

fn law_enforcement_share(destination: &StarSystem, standing: Standing) -> f64 {
    share_of_for(
        EventType::MilitaryPatrol,
        EventPhase::Approach,
        destination,
        standing,
    ) + share_of_for(
        EventType::GovernmentInspection,
        EventPhase::Approach,
        destination,
        standing,
    )
}

#[test]
fn infamy_brings_more_patrols_and_inspections_in_lawful_systems() {
    let lawful = system(GovernmentType::Democracy, EconomyType::RichIndustrial);
    let neutral = law_enforcement_share(&lawful, Standing::Neutral);
    let disreputable = law_enforcement_share(&lawful, Standing::Disreputable);
    let criminal = law_enforcement_share(&lawful, Standing::Criminal);

    assert!(neutral < disreputable && disreputable < criminal);
    assert!(criminal > 1.3 * neutral, "{} vs {}", criminal, neutral);
}

#[test]
fn infamy_brings_fewer_patrols_and_inspections_in_lawless_systems() {
    let lawless = system(GovernmentType::Anarchy, EconomyType::RichIndustrial);
    let neutral = law_enforcement_share(&lawless, Standing::Neutral);
    let disreputable = law_enforcement_share(&lawless, Standing::Disreputable);
    let criminal = law_enforcement_share(&lawless, Standing::Criminal);

    assert!(criminal < disreputable && disreputable < neutral);
}

#[test]
fn a_good_name_brings_fewer_patrols_and_inspections() {
    for government in [GovernmentType::Democracy, GovernmentType::Anarchy] {
        let destination = system(government, EconomyType::RichIndustrial);
        let neutral = law_enforcement_share(&destination, Standing::Neutral);
        let trusted = law_enforcement_share(&destination, Standing::Trusted);
        let respected = law_enforcement_share(&destination, Standing::Respected);
        assert!(respected < trusted && trusted < neutral);
    }
}

#[test]
fn standing_leaves_every_other_event_alone() {
    for event_type in [
        EventType::PirateAttack,
        EventType::AlienEncounter,
        EventType::RescueSignal,
        EventType::Wormhole,
    ] {
        for standing in [Standing::Criminal, Standing::Respected] {
            assert_eq!(
                standing_weight(&event_type, standing, &GovernmentType::Democracy),
                1.0
            );
        }
    }
}

fn outcome_of(manager: &EventManager, event_type: EventType, choice: usize) -> &EventOutcome {
    &manager.events.get(&event_type).unwrap().choices[choice].outcome
}

fn rolled_credits(event_type: EventType, choice: usize, held: u32, seed: u64) -> i32 {
    use rand::rngs::StdRng;
    use rand::SeedableRng;
    let manager = EventManager::new();
    roll_outcome(
        outcome_of(&manager, event_type, choice),
        held,
        &mut StdRng::seed_from_u64(seed),
    )
    .credits_change
}

#[test]
fn credit_gains_are_rolled_within_the_range() {
    let rolls: Vec<i32> = (0..300)
        .map(|seed| rolled_credits(EventType::AlienEncounter, 0, 10_000, seed))
        .collect();
    assert!(rolls.iter().all(|credits| (90..=210).contains(credits)));
    assert!(rolls.iter().min() < Some(&110) && rolls.iter().max() > Some(&190));
}

#[test]
fn fines_are_rolled_within_the_range() {
    let rolls: Vec<i32> = (0..300)
        .map(|seed| rolled_credits(EventType::GovernmentInspection, 1, 10_000, seed))
        .collect();
    assert!(rolls.iter().all(|credits| (-140..=-60).contains(credits)));
    assert!(rolls.iter().min() != rolls.iter().max());
}

#[test]
fn windfalls_are_capped_for_poor_commanders() {
    for seed in 0..300 {
        assert!(rolled_credits(EventType::HyperspaceAnomaly, 0, 100, seed) <= 150);
    }
    let rich_max = (0..300)
        .map(|seed| rolled_credits(EventType::HyperspaceAnomaly, 0, 10_000, seed))
        .max()
        .unwrap();
    assert!(rich_max > 240);
}

#[test]
fn cargo_tons_are_rolled_within_one_of_the_base() {
    use rand::rngs::StdRng;
    use rand::SeedableRng;
    let manager = EventManager::new();
    let outcome = outcome_of(&manager, EventType::SpaceDebris, 0);
    let tons: Vec<u32> = (0..300)
        .map(|seed| {
            roll_outcome(outcome, 1000, &mut StdRng::seed_from_u64(seed))
                .cargo_gain
                .unwrap()
                .tons
        })
        .collect();
    assert!(tons.iter().all(|tons| (2..=4).contains(tons)));
    assert!(tons.contains(&2) && tons.contains(&4));
}

#[test]
fn the_message_shows_the_rolled_amounts() {
    use rand::rngs::StdRng;
    use rand::SeedableRng;
    let manager = EventManager::new();
    let outcome = outcome_of(&manager, EventType::SpaceDebris, 0);
    for seed in 0..50 {
        let rolled = roll_outcome(outcome, 1000, &mut StdRng::seed_from_u64(seed));
        assert!(rolled
            .message
            .contains(&format!("+{} credits", rolled.credits_change)));
        assert!(rolled
            .message
            .contains(&format!("+{} tons", rolled.cargo_gain.unwrap().tons)));
        assert!(!rolled.message.contains('{'));
    }
}

#[test]
fn processing_an_event_applies_the_rolled_credits() {
    use rand::rngs::StdRng;
    use rand::SeedableRng;
    let manager = EventManager::new();
    let event = manager.events.get(&EventType::AlienEncounter).unwrap();
    let mut commander = Commander::new("Test".to_string());
    commander.credits = 1000;
    let mut ship = Ship::new_cobra_mk3();
    let outcome = manager
        .process_event_with(
            event,
            &mut commander,
            &mut ship,
            0,
            &mut StdRng::seed_from_u64(7),
        )
        .unwrap();
    assert_eq!(commander.credits as i32, 1000 + outcome.credits_change);
}

#[test]
fn every_outcome_message_placeholder_is_filled() {
    use rand::rngs::StdRng;
    use rand::SeedableRng;
    let manager = EventManager::new();
    for event in manager.events.values() {
        for choice in &event.choices {
            let rolled = roll_outcome(&choice.outcome, 500, &mut StdRng::seed_from_u64(1));
            assert!(!rolled.message.contains('{'), "{}", choice.outcome.message);
        }
    }
}

fn mean_event_credits_per_jump(held: u32) -> f64 {
    use rand::rngs::StdRng;
    use rand::SeedableRng;
    let manager = EventManager::new();
    let systems = crate::data::systems::SystemManager::new().unwrap();
    let mut rng = StdRng::seed_from_u64(1);
    let (mut total, mut jumps) = (0.0, 0.0);
    for destination in systems.get_all_systems() {
        for _ in 0..400 {
            for phase in [EventPhase::Jump, EventPhase::Approach] {
                let base = match phase {
                    EventPhase::Jump => 0.10,
                    EventPhase::Approach => 0.12,
                };
                let chance = base * manager.calculate_contextual_modifier(destination);
                if rng.gen_bool(chance) {
                    let event = manager
                        .pick_event(&phase, destination, Standing::Neutral, &mut rng)
                        .unwrap();
                    let best = event
                        .choices
                        .iter()
                        .filter(|choice| !choice.outcome.starts_combat)
                        .map(|choice| {
                            let outcome = roll_outcome(&choice.outcome, held, &mut rng);
                            outcome.credits_change - 2 * outcome.hull_damage.abs()
                        })
                        .max()
                        .unwrap();
                    total += f64::from(best);
                }
            }
            jumps += 1.0;
        }
    }
    total / jumps
}

#[test]
fn events_add_a_modest_income_next_to_trading() {
    let poor = mean_event_credits_per_jump(100);
    let mid = mean_event_credits_per_jump(500);
    let rich = mean_event_credits_per_jump(2000);
    assert!((5.0..=30.0).contains(&mid), "500cr: {mid:.1}");
    assert!(poor <= mid, "100cr: {poor:.1}");
    assert!(rich <= 38.0, "2000cr: {rich:.1}");
}

#[test]
fn trigger_chances_are_ten_and_twelve_percent_at_a_standard_destination() {
    let manager = EventManager::new();
    let standard = system(GovernmentType::Democracy, EconomyType::AverageIndustrial);
    assert!((manager.trigger_probability(&EventPhase::Jump, &standard) - 0.10).abs() < 1e-9);
    assert!((manager.trigger_probability(&EventPhase::Approach, &standard) - 0.12).abs() < 1e-9);
}

#[test]
fn trigger_chances_scale_with_the_destination() {
    let manager = EventManager::new();
    let lawless_poor = system(GovernmentType::Anarchy, EconomyType::PoorAgricultural);
    let rich_corporate = system(GovernmentType::CorporateState, EconomyType::RichIndustrial);
    assert!(
        (manager.trigger_probability(&EventPhase::Jump, &lawless_poor) - 0.10 * 1.3 * 1.4).abs()
            < 1e-9
    );
    assert!(
        (manager.trigger_probability(&EventPhase::Approach, &rich_corporate) - 0.12 * 0.9 * 0.8)
            .abs()
            < 1e-9
    );
}

#[test]
fn each_phase_triggers_at_its_own_rate_and_the_two_roll_independently() {
    use rand::rngs::StdRng;
    use rand::SeedableRng;
    let manager = EventManager::new();
    let standard = system(GovernmentType::Democracy, EconomyType::AverageIndustrial);
    let mut rng = StdRng::seed_from_u64(5);
    let trials = 100_000;
    let (mut jump, mut approach, mut both) = (0, 0, 0);
    for _ in 0..trials {
        let j = manager.should_trigger_event_with(&EventPhase::Jump, &standard, &mut rng);
        let a = manager.should_trigger_event_with(&EventPhase::Approach, &standard, &mut rng);
        jump += i32::from(j);
        approach += i32::from(a);
        both += i32::from(j && a);
    }
    let rate = |count: i32| f64::from(count) / f64::from(trials);
    assert!((rate(jump) - 0.10).abs() < 0.005, "jump {}", rate(jump));
    assert!(
        (rate(approach) - 0.12).abs() < 0.005,
        "approach {}",
        rate(approach)
    );
    assert!((rate(both) - 0.012).abs() < 0.002, "both {}", rate(both));
}
