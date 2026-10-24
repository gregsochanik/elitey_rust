mod common;

use common::new_game;
use elitey_rust::data::save_load::SaveLoadManager;
use rand::rngs::StdRng;
use rand::SeedableRng;

fn game_with_board() -> elitey_rust::game::GameState {
    let mut game = new_game(100);
    game.jobs_here(&mut StdRng::seed_from_u64(4)).unwrap();
    game
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
    let job = game.job_board_jobs()[0].clone();
    game.accept_job(1).unwrap();
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
    game.accept_job(1).unwrap();
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
