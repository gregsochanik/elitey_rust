use crate::data::save_load::{SaveLoadManager, SaveMetadata};
use crate::game::GameState;
use thiserror::Error;

/// Why a save operation failed
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SaveError {
    #[error("Slot name must contain only letters, numbers, and underscores")]
    InvalidSlotName,
    #[error("Save file '{0}' not found")]
    NotFound(String),
    #[error("Failed to save game: {0}")]
    Save(String),
    #[error("Failed to load save metadata: {0}")]
    Metadata(String),
    #[error("Failed to load game: {0}")]
    Load(String),
    #[error("Failed to list saves: {0}")]
    List(String),
    #[error("Failed to delete save: {0}")]
    Delete(String),
    #[error("Auto-save failed: {0}")]
    AutoSave(String),
}

/// Save/load command handler
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SaveLoadHandler {
    save_manager: SaveLoadManager,
}

impl SaveLoadHandler {
    /// Create a new save/load handler
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            save_manager: SaveLoadManager::new()?,
        })
    }

    /// Create a handler that keeps saves in `directory`
    pub fn with_directory(directory: std::path::PathBuf) -> Result<Self, SaveError> {
        Ok(Self {
            save_manager: SaveLoadManager::in_directory(directory).map_err(SaveError::Save)?,
        })
    }

    /// Save to a named slot (letters, numbers and underscores only)
    pub fn save(&self, game_state: &GameState, slot_name: &str) -> Result<(), SaveError> {
        if !slot_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(SaveError::InvalidSlotName);
        }
        self.save_manager
            .save_game(game_state, slot_name)
            .map_err(SaveError::Save)
    }

    /// What is stored in a slot, without loading it
    pub fn preview(&self, slot_name: &str) -> Result<SaveMetadata, SaveError> {
        if !self.save_manager.save_exists(slot_name) {
            return Err(SaveError::NotFound(slot_name.to_string()));
        }
        self.save_manager
            .load_save_metadata(slot_name)
            .map_err(SaveError::Metadata)
    }

    pub fn load(&self, slot_name: &str) -> Result<GameState, SaveError> {
        self.save_manager
            .load_game(slot_name)
            .map_err(SaveError::Load)
    }

    pub fn list(&self) -> Result<Vec<SaveMetadata>, SaveError> {
        self.save_manager.list_saves().map_err(SaveError::List)
    }

    /// Every save with the slot name it loads from, newest first
    pub fn list_slots(&self) -> Result<Vec<(String, SaveMetadata)>, SaveError> {
        self.save_manager.list_slots().map_err(SaveError::List)
    }

    pub fn delete(&self, slot_name: &str) -> Result<(), SaveError> {
        if !self.save_manager.save_exists(slot_name) {
            return Err(SaveError::NotFound(slot_name.to_string()));
        }
        self.save_manager
            .delete_save(slot_name)
            .map_err(SaveError::Delete)
    }

    /// Write the commander's autosave
    pub fn auto_save(&self, game_state: &GameState) -> Result<(), SaveError> {
        let slot_name = Self::autosave_slot(game_state.commander().name());
        self.save_manager
            .save_game(game_state, &slot_name)
            .map_err(SaveError::AutoSave)
    }

    /// Slot the autosave for a commander is written to
    pub fn autosave_slot(commander_name: &str) -> String {
        let name = commander_name.to_ascii_lowercase();
        let slug: String = name
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
            .take(40)
            .collect();
        let hash = name.bytes().fold(0x811c_9dc5_u32, |hash, byte| {
            (hash ^ u32::from(byte)).wrapping_mul(0x0100_0193)
        });
        format!("autosave-{}-{:08x}", slug, hash)
    }

    /// The autosave for this commander. `Ok(None)` means there is none;
    /// an unreadable save is an error.
    pub fn find_autosave(&self, commander_name: &str) -> Result<Option<GameState>, String> {
        let slot = Self::autosave_slot(commander_name);
        if self.save_manager.save_exists(&slot) {
            let meta = self.save_manager.load_save_metadata(&slot)?;
            if !meta.commander_name.eq_ignore_ascii_case(commander_name) {
                return Err(format!(
                    "slot '{}' belongs to commander {}",
                    slot, meta.commander_name
                ));
            }
            return self.save_manager.load_game(&slot).map(Some);
        }

        let legacy_matches = self
            .save_manager
            .load_save_metadata("autosave")
            .map(|meta| meta.commander_name.eq_ignore_ascii_case(commander_name))
            .unwrap_or(false);
        if legacy_matches {
            self.save_manager.load_game("autosave").map(Some)
        } else {
            Ok(None)
        }
    }
}

impl Default for SaveLoadHandler {
    fn default() -> Self {
        Self::new().expect("Failed to create SaveLoadHandler")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_load_handler_creation() {
        let handler = SaveLoadHandler::new();
        assert!(handler.is_ok());
    }

    fn temp_handler(label: &str) -> SaveLoadHandler {
        let directory =
            std::env::temp_dir().join(format!("elitey-save-test-{}-{}", label, std::process::id()));
        let _ = std::fs::remove_dir_all(&directory);
        SaveLoadHandler::with_directory(directory).unwrap()
    }

    #[test]
    fn slot_names_are_validated_before_saving() {
        let handler = temp_handler("slot-names");
        let game = GameState::new("Test".to_string());

        assert_eq!(
            handler.save(&game, "bad name!"),
            Err(SaveError::InvalidSlotName)
        );
        assert!(handler.save(&game, "good_name_1").is_ok());
    }

    #[test]
    fn a_save_at_an_unknown_system_loads_at_sol() {
        let handler = temp_handler("lost");
        let mut game = GameState::new("Test".to_string());
        game.commander.location = "USCELA".to_string();
        handler.save(&game, "lost").unwrap();

        let loaded = handler.load("lost").unwrap();

        assert_eq!(loaded.commander.location, "Sol");
    }

    #[test]
    fn a_save_made_before_visits_were_kept_counts_where_the_commander_is() {
        let handler = temp_handler("visits");
        let mut game = GameState::new("Test".to_string());
        game.commander.visited.clear();
        handler.save(&game, "old").unwrap();

        let loaded = handler.load("old").unwrap();

        assert!(loaded.commander.has_visited("Sol"));
        assert_eq!(loaded.exploration().visited, 1);
    }

    #[test]
    fn saves_round_trip_through_a_chosen_directory() {
        let handler = temp_handler("round-trip");
        let mut game = GameState::new("Test".to_string());
        game.commander.credits = 777;

        handler.save(&game, "slot1").unwrap();

        assert_eq!(handler.preview("slot1").unwrap().credits, 777);
        assert_eq!(handler.list().unwrap().len(), 1);
        let slots = handler.list_slots().unwrap();
        assert_eq!(slots.len(), 1);
        assert_eq!(slots[0].0, "slot1");
        assert_eq!(handler.load("slot1").unwrap().commander.credits(), 777);
        assert!(handler.delete("slot1").is_ok());
        assert!(handler.list().unwrap().is_empty());
    }

    #[test]
    fn missing_slots_are_reported_by_name() {
        let handler = temp_handler("missing");

        assert_eq!(
            handler.preview("nope").unwrap_err(),
            SaveError::NotFound("nope".to_string())
        );
        assert_eq!(
            handler.delete("nope").unwrap_err().to_string(),
            "Save file 'nope' not found"
        );
    }
}
