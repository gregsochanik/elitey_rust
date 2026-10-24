#![allow(dead_code)]

use elitey_rust::data::systems::SystemManager;
use elitey_rust::game::{
    Commander, EventManager, GameState, JobBoard, SaveLoadHandler, Ship, TradeLog, TradingHandler,
    TravelHandler, UpgradeHandler,
};
use std::sync::Once;

static ISOLATE_HOME: Once = Once::new();

/// Point HOME at a temp directory so tests never touch real save files
pub fn isolate_home() {
    ISOLATE_HOME.call_once(|| {
        let dir = std::env::temp_dir().join(format!("elitey_rust_tests_{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("create temp home");
        std::env::set_var("HOME", dir);
    });
}

pub fn new_game(credits: u32) -> GameState {
    isolate_home();
    let system_manager = SystemManager::new().expect("galaxy data");
    let mut commander = Commander::new("Tester".to_string());
    commander.credits = credits;
    GameState {
        commander,
        ship: Ship::new_cobra_mk3(),
        system_manager: system_manager.clone(),
        trading_handler: TradingHandler::new(),
        upgrade_handler: UpgradeHandler::new(),
        save_load_handler: SaveLoadHandler::new().expect("save handler"),
        event_manager: EventManager::new(),
        travel_handler: TravelHandler::new(system_manager),
        trade_log: TradeLog::new(),
        job_board: JobBoard::new(),
        active_jobs: Vec::new(),
    }
}

pub fn args(text: &str) -> Vec<String> {
    text.split_whitespace().map(String::from).collect()
}
