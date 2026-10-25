mod common;

use common::{args, new_game};
use elitey_rust::data::save_load::SaveLoadManager;
use rand::rngs::StdRng;
use rand::SeedableRng;

fn game_with_board() -> elitey_rust::game::GameState {
    let mut game = new_game(100);
    game.jobs_here(&mut StdRng::seed_from_u64(4)).unwrap();
    game
}

fn courier_index(game: &elitey_rust::game::GameState) -> usize {
    use elitey_rust::game::jobs::JobKind;
    game.job_board_jobs()
        .iter()
        .position(|job| job.kind == JobKind::Courier)
        .expect("a courier job on the board")
        + 1
}

#[test]
fn accepting_jobs_fills_missions_up_to_three() {
    let mut game = game_with_board();
    let first = game.job_board_jobs()[0].clone();

    assert_eq!(game.accept_job(1).unwrap(), first);
    assert_eq!(game.active_jobs.len(), 1);
    assert!(game.accept_job(1).is_ok());
    assert!(game.accept_job(1).is_ok());

    assert!(game.accept_job(1).is_err());
    assert_eq!(game.active_jobs.len(), 3);
}

#[test]
fn abandoning_a_job_costs_reputation_and_renumbers_the_rest() {
    let mut game = game_with_board();
    game.accept_job(1).unwrap();
    game.accept_job(1).unwrap();
    let second = game.active_jobs[1].clone();

    game.abandon_job(1).unwrap();

    assert_eq!(game.commander.reputation, -3);
    assert_eq!(game.active_jobs, vec![second]);
}

#[test]
fn active_jobs_survive_saving_and_loading() {
    let mut game = game_with_board();
    game.accept_job(2).unwrap();
    let expected = game.active_jobs.clone();

    let manager = SaveLoadManager::new().unwrap();
    manager.save_game(&game, "jobs_flow").unwrap();
    let loaded = manager.load_game("jobs_flow").unwrap();

    assert_eq!(loaded.active_jobs, expected);
}

#[test]
fn old_saves_without_active_jobs_load_with_none() {
    let game = new_game(100);
    let mut value = serde_json::to_value(&game).unwrap();
    value.as_object_mut().unwrap().remove("active_jobs");

    let loaded: elitey_rust::game::GameState = serde_json::from_value(value).unwrap();

    assert!(loaded.active_jobs.is_empty());
}

#[test]
fn a_delivered_job_pays_out_through_the_game() {
    let mut game = game_with_board();
    let index = courier_index(&game);
    let job = game.job_board_jobs()[index - 1].clone();
    game.accept_job(index).unwrap();
    let credits = game.commander.credits();

    game.commander.location = job.destination.clone();
    game.settle_jobs();

    assert!(game.active_jobs.is_empty());
    assert_eq!(game.commander.credits(), credits + job.reward);
    assert_eq!(game.commander.reputation, 2);
}

#[test]
fn a_late_job_fails_through_the_game() {
    let mut game = game_with_board();
    game.accept_job(courier_index(&game)).unwrap();
    let destination = game.active_jobs[0].job.destination.clone();
    let deadline = game.active_jobs[0].job.deadline_jumps;
    let elsewhere = ["LAVE", "DISO", "LEESTI", "ZAONCE"]
        .into_iter()
        .find(|name| *name != destination)
        .unwrap();

    for jump in 1..=deadline {
        game.commander.location = elsewhere.to_string();
        game.settle_jobs();
        assert_eq!(game.active_jobs.is_empty(), jump == deadline);
    }

    assert_eq!(game.commander.reputation, -3);
}

fn game_with_freight_on_the_board() -> elitey_rust::game::GameState {
    use elitey_rust::game::jobs::JobKind;

    let mut game = new_game(500);
    game.commander.reputation = 30;
    for seed in 0..200u64 {
        game.job_board.clear();
        game.jobs_here(&mut StdRng::seed_from_u64(seed)).unwrap();
        if game
            .job_board_jobs()
            .iter()
            .any(|job| job.kind == JobKind::Freight)
        {
            return game;
        }
    }
    panic!("no freight offered to a Respected commander at a lawful system");
}

#[test]
fn freight_takes_room_cannot_be_sold_and_pays_on_delivery() {
    use elitey_rust::game::jobs::JobKind;

    let mut game = game_with_freight_on_the_board();
    let index = game
        .job_board_jobs()
        .iter()
        .position(|job| job.kind == JobKind::Freight)
        .unwrap()
        + 1;
    let job = game.job_board_jobs()[index - 1].clone();
    let commodity = job.commodity.clone().unwrap();
    let credits = game.commander.credits();

    game.accept_job(index).unwrap();
    assert_eq!(game.ship.cargo_hold().reserved, job.tons);
    assert_eq!(game.ship.cargo_used(), job.tons);

    // contract cargo is not goods: it cannot be sold at any market
    assert!(!elitey_rust::cli::trading::sell(
        &mut game,
        &args(&format!("{} 1", commodity.id()))
    ));
    assert_eq!(game.ship.cargo_hold().reserved, job.tons);

    game.commander.location = job.destination.clone();
    game.settle_jobs();

    assert_eq!(game.ship.cargo_hold().reserved, 0);
    assert!(game.active_jobs.is_empty());
    assert_eq!(game.commander.credits(), credits + job.reward);
}

#[test]
fn a_full_hold_refuses_freight_and_nothing_changes() {
    use elitey_rust::data::commodities::CommodityType;
    use elitey_rust::game::jobs::JobKind;

    let mut game = game_with_freight_on_the_board();
    let index = game
        .job_board_jobs()
        .iter()
        .position(|job| job.kind == JobKind::Freight)
        .unwrap()
        + 1;
    let tons = game.job_board_jobs()[index - 1].tons;
    let fill = game.ship.cargo_capacity() - tons + 1;
    assert!(game.ship.add_cargo(CommodityType::Food, fill));

    assert!(game.accept_job(index).is_err());
    assert!(game.active_jobs.is_empty());
    assert_eq!(game.ship.cargo_hold().reserved, 0);
}

#[test]
fn contract_cargo_survives_saving_and_loading_and_old_holds_load_empty() {
    let mut game = game_with_freight_on_the_board();
    let index = game
        .job_board_jobs()
        .iter()
        .position(|job| job.kind == elitey_rust::game::jobs::JobKind::Freight)
        .unwrap()
        + 1;
    game.accept_job(index).unwrap();
    let reserved = game.ship.cargo_hold().reserved;

    let manager = SaveLoadManager::new().unwrap();
    manager.save_game(&game, "freight_flow").unwrap();
    assert_eq!(
        manager
            .load_game("freight_flow")
            .unwrap()
            .ship
            .cargo_hold()
            .reserved,
        reserved
    );

    let mut value = serde_json::to_value(new_game(100)).unwrap();
    value["ship"]["cargo_hold"]
        .as_object_mut()
        .unwrap()
        .remove("reserved");
    let old: elitey_rust::game::GameState = serde_json::from_value(value).unwrap();
    assert_eq!(old.ship.cargo_hold().reserved, 0);
}

fn criminal_game_with_smuggling_on_the_board() -> elitey_rust::game::GameState {
    use elitey_rust::game::jobs::JobKind;

    let mut game = new_game(500);
    game.commander.reputation = -40;
    let lawless = game
        .travel_handler
        .get_available_systems()
        .into_iter()
        .find(|system| !system.government.is_lawful())
        .map(|system| system.name.clone())
        .expect("a lawless system");
    game.commander.location = lawless;
    for seed in 0..300u64 {
        game.job_board.clear();
        game.jobs_here(&mut StdRng::seed_from_u64(seed)).unwrap();
        if game
            .job_board_jobs()
            .iter()
            .any(|job| job.kind == JobKind::Smuggling)
        {
            return game;
        }
    }
    panic!("no smuggling offered to a Criminal commander at a lawless system");
}

#[test]
fn smuggling_ends_with_the_job_gone_the_hold_empty_and_either_pay_or_a_fine() {
    use elitey_rust::game::jobs::JobKind;

    let mut game = criminal_game_with_smuggling_on_the_board();
    let index = game
        .job_board_jobs()
        .iter()
        .position(|job| job.kind == JobKind::Smuggling)
        .unwrap()
        + 1;
    let job = game.job_board_jobs()[index - 1].clone();
    game.accept_job(index).unwrap();
    assert_eq!(game.ship.cargo_hold().reserved, job.tons);
    assert_eq!(game.ship.cargo_hold().goods_weight(), 0);
    let credits = game.commander.credits();
    let reputation = game.commander.reputation;

    game.commander.location = job.destination.clone();
    game.settle_jobs();

    assert!(game.active_jobs.is_empty());
    assert_eq!(game.ship.cargo_hold().reserved, 0);
    let paid = game.commander.credits() > credits;
    if paid {
        assert_eq!(game.commander.reputation, reputation - 3);
    } else {
        assert_eq!(game.commander.reputation, reputation - 5);
        assert!(game.commander.credits() <= credits);
    }
}

#[test]
fn a_neutral_commander_is_not_offered_smuggling() {
    use elitey_rust::game::jobs::JobKind;

    let mut game = new_game(500);
    game.commander.location = game
        .travel_handler
        .get_available_systems()
        .into_iter()
        .find(|system| !system.government.is_lawful())
        .map(|system| system.name.clone())
        .unwrap();
    for seed in 0..100u64 {
        game.job_board.clear();
        game.jobs_here(&mut StdRng::seed_from_u64(seed)).unwrap();
        assert!(game
            .job_board_jobs()
            .iter()
            .all(|job| job.kind != JobKind::Smuggling));
    }
}
