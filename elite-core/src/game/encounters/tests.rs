use super::*;
use crate::data::systems::{GovernmentType, StarSystem};
use crate::game::commander::CombatRating;
use rand::SeedableRng;

const GOVERNMENTS: [GovernmentType; 8] = [
    GovernmentType::Anarchy,
    GovernmentType::Feudal,
    GovernmentType::MultiGovernment,
    GovernmentType::Dictatorship,
    GovernmentType::Communist,
    GovernmentType::Confederacy,
    GovernmentType::Democracy,
    GovernmentType::CorporateState,
];

const ECONOMIES: [EconomyType; 8] = [
    EconomyType::RichIndustrial,
    EconomyType::PoorIndustrial,
    EconomyType::MainlyIndustrial,
    EconomyType::AverageIndustrial,
    EconomyType::RichAgricultural,
    EconomyType::PoorAgricultural,
    EconomyType::MainlyAgricultural,
    EconomyType::AverageAgricultural,
];

const STANDINGS: [Standing; 5] = [
    Standing::Criminal,
    Standing::Disreputable,
    Standing::Neutral,
    Standing::Trusted,
    Standing::Respected,
];

const OLD_EVENTS: [EventType; 10] = [
    EventType::Wormhole,
    EventType::HyperspaceAnomaly,
    EventType::NavigationError,
    EventType::SpaceDebris,
    EventType::AlienEncounter,
    EventType::PirateAttack,
    EventType::RescueSignal,
    EventType::GovernmentInspection,
    EventType::MilitaryPatrol,
    EventType::TraderConvoy,
];

fn system(government: GovernmentType, economy: EconomyType) -> StarSystem {
    StarSystem {
        name: "Test".to_string(),
        x: 0,
        y: 0,
        economy,
        government,
        tech_level: 5,
        turnover: 0,
        radius: 0,
        population: String::new(),
        description: String::new(),
        star_class: String::new(),
    }
}

/// The weights as they were written in code before they moved into data
mod before {
    use super::*;

    pub fn weight(event: &EventType, system: &StarSystem, standing: Standing) -> f64 {
        let base = match event {
            EventType::TraderConvoy => 0.6,
            _ => 1.0,
        };
        base * government(event, &system.government)
            * economy(event, &system.economy)
            * standing_weight(event, standing, &system.government)
    }

    fn standing_weight(event: &EventType, standing: Standing, government: &GovernmentType) -> f64 {
        if !matches!(
            event,
            EventType::MilitaryPatrol | EventType::GovernmentInspection
        ) {
            return 1.0;
        }
        let lawful = government.is_lawful();
        match standing {
            Standing::Criminal if lawful => 2.0,
            Standing::Criminal => 0.5,
            Standing::Disreputable if lawful => 1.5,
            Standing::Disreputable => 0.75,
            Standing::Neutral => 1.0,
            Standing::Trusted => 0.75,
            Standing::Respected => 0.5,
        }
    }

    fn government(event: &EventType, government: &GovernmentType) -> f64 {
        match (government, event) {
            (GovernmentType::Anarchy, EventType::PirateAttack) => 2.5,
            (GovernmentType::Anarchy, EventType::GovernmentInspection) => 0.2,
            (GovernmentType::Anarchy, EventType::MilitaryPatrol) => 0.3,
            (GovernmentType::Communist, EventType::GovernmentInspection) => 2.5,
            (GovernmentType::Communist, EventType::MilitaryPatrol) => 1.5,
            (GovernmentType::Communist, EventType::PirateAttack) => 0.6,
            (GovernmentType::Dictatorship, EventType::MilitaryPatrol) => 2.5,
            (GovernmentType::Dictatorship, EventType::GovernmentInspection) => 1.5,
            (GovernmentType::Dictatorship, EventType::PirateAttack) => 0.7,
            (GovernmentType::CorporateState, EventType::GovernmentInspection) => 1.5,
            (GovernmentType::CorporateState, EventType::PirateAttack) => 0.5,
            _ => 1.0,
        }
    }

    fn economy(event: &EventType, economy: &EconomyType) -> f64 {
        match (economy, event) {
            (
                EconomyType::PoorAgricultural | EconomyType::PoorIndustrial,
                EventType::PirateAttack,
            ) => 1.5,
            (
                EconomyType::PoorAgricultural | EconomyType::PoorIndustrial,
                EventType::RescueSignal,
            ) => 1.5,
            (
                EconomyType::RichIndustrial | EconomyType::RichAgricultural,
                EventType::PirateAttack,
            ) => 0.6,
            (EconomyType::MainlyIndustrial, EventType::SpaceDebris) => 1.5,
            (EconomyType::MainlyAgricultural, EventType::RescueSignal) => 1.3,
            _ => 1.0,
        }
    }
}

#[test]
fn the_table_gives_the_weights_the_code_used_to() {
    for government in &GOVERNMENTS {
        for economy in &ECONOMIES {
            let here = system(government.clone(), economy.clone());
            for standing in STANDINGS {
                for event in &OLD_EVENTS {
                    assert_eq!(
                        weight(event, &here, standing),
                        before::weight(event, &here, standing),
                        "{event:?} {government:?} {economy:?} {standing:?}"
                    );
                }
            }
        }
    }
}

#[test]
fn bounty_hunters_only_come_for_the_wanted() {
    let here = system(GovernmentType::Democracy, EconomyType::AverageIndustrial);

    assert_eq!(
        weight(&EventType::BountyHunter, &here, Standing::Criminal),
        2.0
    );
    assert_eq!(
        weight(&EventType::BountyHunter, &here, Standing::Disreputable),
        0.6
    );
    for standing in [Standing::Neutral, Standing::Trusted, Standing::Respected] {
        assert_eq!(weight(&EventType::BountyHunter, &here, standing), 0.0);
    }
}

fn extra_share(event: &EventType, government: GovernmentType, economy: EconomyType) -> (f64, f64) {
    let here = system(government, economy);
    let mut rng = rand::rngs::StdRng::seed_from_u64(5);
    let runs = 20_000;
    let (mut any, mut two) = (0, 0);
    for _ in 0..runs {
        let extra = extra_foes(event, &here, &mut rng);
        any += u32::from(extra >= 1);
        two += u32::from(extra == 2);
    }
    (
        f64::from(any) / f64::from(runs),
        f64::from(two) / f64::from(runs),
    )
}

#[test]
fn pirates_in_anarchy_come_in_groups() {
    let (any, two) = extra_share(
        &EventType::PirateAttack,
        GovernmentType::Anarchy,
        EconomyType::RichIndustrial,
    );

    assert!((0.32..=0.38).contains(&any), "{any}");
    assert!((0.04..=0.08).contains(&two), "{two}");
}

#[test]
fn groups_need_a_reason_and_only_pirates_have_them() {
    let (calm, _) = extra_share(
        &EventType::PirateAttack,
        GovernmentType::Democracy,
        EconomyType::RichIndustrial,
    );
    let (patrols, _) = extra_share(
        &EventType::MilitaryPatrol,
        GovernmentType::Anarchy,
        EconomyType::PoorAgricultural,
    );
    let (poor, _) = extra_share(
        &EventType::PirateAttack,
        GovernmentType::Democracy,
        EconomyType::PoorAgricultural,
    );

    assert_eq!(calm, 0.0);
    assert_eq!(patrols, 0.0);
    assert!((0.12..=0.18).contains(&poor), "{poor}");
}

#[test]
fn only_pirates_toughen_with_the_commanders_rating() {
    assert_eq!(
        difficulty("pirate-raider", CombatRating::Harmless),
        Difficulty::NONE
    );
    let elite = difficulty("pirate-raider", CombatRating::Elite);
    assert!((elite.health - 1.48).abs() < 1e-9);
    assert!((elite.nerve - 0.32).abs() < 1e-9);
    assert_eq!(
        difficulty("patrol-vessel", CombatRating::Elite),
        Difficulty::NONE
    );
    assert_eq!(
        difficulty("bounty-target", CombatRating::Elite),
        Difficulty::NONE
    );
}
