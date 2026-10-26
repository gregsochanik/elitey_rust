#![allow(dead_code)]

use elite_core::data::systems::SystemManager;
use elite_core::game::{
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
        world_clock: 0,
    }
}

pub fn args(text: &str) -> Vec<String> {
    text.split_whitespace().map(String::from).collect()
}

/// Move the commander to the system with the highest tech level, where every item is on sale
pub fn go_to_high_tech(game: &mut GameState) {
    let best = game
        .system_manager
        .get_all_systems()
        .into_iter()
        .max_by_key(|system| (system.tech_level, system.name.clone()))
        .expect("a system")
        .name
        .clone();
    game.commander.location = best;
}
