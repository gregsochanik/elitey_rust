use super::*;

#[test]
fn test_game_state_creation() {
    // This test would need to be modified to handle user input
    // For now, we'll test the basic structure
    let commander = Commander::new("Test Commander".to_string());
    let ship = Ship::new_cobra_mk3();
    let system_manager = SystemManager::new().expect("Failed to load galaxy data");
    let system_manager_clone = system_manager.clone();
    let game_state = GameState {
        commander,
        ship,
        system_manager,
        trading_handler: TradingHandler::new(),
        upgrade_handler: UpgradeHandler::new(),
        save_load_handler: SaveLoadHandler::new().expect("Failed to create save/load handler"),
        event_manager: EventManager::new(),
        travel_handler: TravelHandler::new(system_manager_clone),
        trade_log: TradeLog::new(),
        job_board: JobBoard::new(),
        active_jobs: Vec::new(),
        world_clock: 0,
    };

    assert_eq!(game_state.commander().name(), "Test Commander");
    assert_eq!(game_state.ship().ship_type_name(), "Cobra Mk III");
    assert!(game_state.is_running());
}

#[test]
fn test_game_state_running() {
    let commander = Commander::new("Test".to_string());
    let ship = Ship::new_cobra_mk3();
    let system_manager = SystemManager::new().expect("Failed to load galaxy data");
    let system_manager_clone = system_manager.clone();
    let game_state = GameState {
        commander,
        ship,
        system_manager,
        trading_handler: TradingHandler::new(),
        upgrade_handler: UpgradeHandler::new(),
        save_load_handler: SaveLoadHandler::new().expect("Failed to create save/load handler"),
        event_manager: EventManager::new(),
        travel_handler: TravelHandler::new(system_manager_clone),
        trade_log: TradeLog::new(),
        job_board: JobBoard::new(),
        active_jobs: Vec::new(),
        world_clock: 0,
    };

    assert!(game_state.is_running());
}

#[test]
fn test_game_state_destroyed() {
    let commander = Commander::new("Test".to_string());
    let mut ship = Ship::new_cobra_mk3();
    ship.take_damage(100); // Destroy the ship
    let system_manager = SystemManager::new().expect("Failed to load galaxy data");
    let system_manager_clone = system_manager.clone();

    let game_state = GameState {
        commander,
        ship,
        system_manager,
        trading_handler: TradingHandler::new(),
        upgrade_handler: UpgradeHandler::new(),
        save_load_handler: SaveLoadHandler::new().expect("Failed to create save/load handler"),
        event_manager: EventManager::new(),
        travel_handler: TravelHandler::new(system_manager_clone),
        trade_log: TradeLog::new(),
        job_board: JobBoard::new(),
        active_jobs: Vec::new(),
        world_clock: 0,
    };

    assert!(!game_state.is_running());
}

#[test]
fn trade_log_missing_from_old_save_loads_empty() {
    let system_manager = SystemManager::new().expect("Failed to load galaxy data");
    let state = GameState {
        commander: Commander::new("Old".to_string()),
        ship: Ship::new_cobra_mk3(),
        system_manager: system_manager.clone(),
        trading_handler: TradingHandler::new(),
        upgrade_handler: UpgradeHandler::new(),
        save_load_handler: SaveLoadHandler::new().expect("Failed to create save/load handler"),
        event_manager: EventManager::new(),
        travel_handler: TravelHandler::new(system_manager),
        trade_log: TradeLog::new(),
        job_board: JobBoard::new(),
        active_jobs: Vec::new(),
        world_clock: 0,
    };
    let mut value = serde_json::to_value(&state).unwrap();
    value.as_object_mut().unwrap().remove("trade_log");

    let loaded: GameState = serde_json::from_value(value).unwrap();
    assert!(loaded.trade_log.transactions().is_empty());
}

#[test]
fn trade_log_survives_serialization() {
    let system_manager = SystemManager::new().expect("Failed to load galaxy data");
    let mut state = GameState {
        commander: Commander::new("New".to_string()),
        ship: Ship::new_cobra_mk3(),
        system_manager: system_manager.clone(),
        trading_handler: TradingHandler::new(),
        upgrade_handler: UpgradeHandler::new(),
        save_load_handler: SaveLoadHandler::new().expect("Failed to create save/load handler"),
        event_manager: EventManager::new(),
        travel_handler: TravelHandler::new(system_manager),
        trade_log: TradeLog::new(),
        job_board: JobBoard::new(),
        active_jobs: Vec::new(),
        world_clock: 0,
    };
    state.trade_log.record_buy(
        crate::data::commodities::CommodityType::Food,
        5,
        10,
        "LAVE",
        0,
    );

    let json = serde_json::to_string(&state).unwrap();
    let loaded: GameState = serde_json::from_str(&json).unwrap();
    assert_eq!(loaded.trade_log.transactions().len(), 1);
}
