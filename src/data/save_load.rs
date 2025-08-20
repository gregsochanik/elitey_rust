use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::game::GameState;

/// Represents save file metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveMetadata {
    /// Commander name
    pub commander_name: String,
    /// Current location
    pub location: String,
    /// Current credits
    pub credits: u32,
    /// Ship type
    pub ship_type: String,
    /// Save timestamp (Unix timestamp)
    pub timestamp: u64,
    /// Game version for compatibility
    pub version: String,
}

/// Represents a complete save file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveFile {
    /// Save metadata
    pub metadata: SaveMetadata,
    /// Complete game state
    pub game_state: GameState,
}

/// Manages save/load operations
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SaveLoadManager {
    /// Directory where save files are stored
    save_directory: PathBuf,
    /// Current game version
    version: String,
}

impl SaveLoadManager {
    /// Create a new save/load manager
    pub fn new() -> Result<Self, String> {
        let save_dir = Self::get_save_directory()?;
        
        // Create save directory if it doesn't exist
        if !save_dir.exists() {
            fs::create_dir_all(&save_dir)
                .map_err(|e| format!("Failed to create save directory: {}", e))?;
        }

        Ok(Self {
            save_directory: save_dir,
            version: "1.0.0".to_string(),
        })
    }

    /// Get the save directory path
    fn get_save_directory() -> Result<PathBuf, String> {
        let mut save_dir = dirs::home_dir()
            .ok_or("Could not determine home directory")?;
        save_dir.push(".elitey_rust");
        save_dir.push("saves");
        Ok(save_dir)
    }

    /// Save game state to a file
    pub fn save_game(&self, game_state: &GameState, slot_name: &str) -> Result<(), String> {
        let save_file = SaveFile {
            metadata: SaveMetadata {
                commander_name: game_state.commander().name().to_string(),
                location: game_state.commander().location().to_string(),
                credits: game_state.commander().credits(),
                ship_type: game_state.ship().ship_type_name().to_string(),
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|e| format!("Failed to get timestamp: {}", e))?
                    .as_secs(),
                version: self.version.clone(),
            },
            game_state: game_state.clone(),
        };

        let file_path = self.get_save_file_path(slot_name);
        
        // Create backup of existing save if it exists
        if file_path.exists() {
            let backup_path = self.get_backup_file_path(slot_name);
            fs::copy(&file_path, &backup_path)
                .map_err(|e| format!("Failed to create backup: {}", e))?;
        }

        // Write save file
        let json = serde_json::to_string_pretty(&save_file)
            .map_err(|e| format!("Failed to serialize save file: {}", e))?;
        
        fs::write(&file_path, json)
            .map_err(|e| format!("Failed to write save file: {}", e))?;

        Ok(())
    }

    /// Load game state from a file
    pub fn load_game(&self, slot_name: &str) -> Result<GameState, String> {
        let file_path = self.get_save_file_path(slot_name);
        
        if !file_path.exists() {
            return Err(format!("Save file '{}' not found", slot_name));
        }

        // Read save file
        let json = fs::read_to_string(&file_path)
            .map_err(|e| format!("Failed to read save file: {}", e))?;
        
        let save_file: SaveFile = serde_json::from_str(&json)
            .map_err(|e| format!("Failed to parse save file: {}", e))?;

        // Validate version compatibility
        if save_file.metadata.version != self.version {
            return Err(format!(
                "Save file version {} is incompatible with current version {}",
                save_file.metadata.version, self.version
            ));
        }

        Ok(save_file.game_state)
    }

    /// List all available save slots
    pub fn list_saves(&self) -> Result<Vec<SaveMetadata>, String> {
        let mut saves = Vec::new();
        
        if !self.save_directory.exists() {
            return Ok(saves);
        }

        for entry in fs::read_dir(&self.save_directory)
            .map_err(|e| format!("Failed to read save directory: {}", e))? {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    if let Ok(save_file) = self.load_save_metadata(stem) {
                        saves.push(save_file);
                    }
                }
            }
        }

        // Sort by timestamp (newest first)
        saves.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(saves)
    }

    /// Load just the metadata from a save file
    pub fn load_save_metadata(&self, slot_name: &str) -> Result<SaveMetadata, String> {
        let file_path = self.get_save_file_path(slot_name);
        
        if !file_path.exists() {
            return Err(format!("Save file '{}' not found", slot_name));
        }

        let json = fs::read_to_string(&file_path)
            .map_err(|e| format!("Failed to read save file: {}", e))?;
        
        let save_file: SaveFile = serde_json::from_str(&json)
            .map_err(|e| format!("Failed to parse save file: {}", e))?;

        Ok(save_file.metadata)
    }

    /// Delete a save file
    pub fn delete_save(&self, slot_name: &str) -> Result<(), String> {
        let file_path = self.get_save_file_path(slot_name);
        
        if !file_path.exists() {
            return Err(format!("Save file '{}' not found", slot_name));
        }

        fs::remove_file(&file_path)
            .map_err(|e| format!("Failed to delete save file: {}", e))?;

        // Also delete backup if it exists
        let backup_path = self.get_backup_file_path(slot_name);
        if backup_path.exists() {
            let _ = fs::remove_file(&backup_path);
        }

        Ok(())
    }

    /// Check if a save file exists
    pub fn save_exists(&self, slot_name: &str) -> bool {
        self.get_save_file_path(slot_name).exists()
    }

    /// Get save file path for a slot
    fn get_save_file_path(&self, slot_name: &str) -> PathBuf {
        let mut path = self.save_directory.clone();
        path.push(format!("{}.json", slot_name));
        path
    }

    /// Get backup file path for a slot
    fn get_backup_file_path(&self, slot_name: &str) -> PathBuf {
        let mut path = self.save_directory.clone();
        path.push(format!("{}.backup", slot_name));
        path
    }

    /// Format timestamp for display
    pub fn format_timestamp(timestamp: u64) -> String {
        use chrono::{DateTime, Utc};
        let dt = DateTime::from_timestamp(timestamp as i64, 0)
            .unwrap_or_else(Utc::now);
        dt.format("%Y-%m-%d %H:%M:%S").to_string()
    }
}

impl Default for SaveLoadManager {
    fn default() -> Self {
        Self::new().expect("Failed to create SaveLoadManager")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_load_manager_creation() {
        let manager = SaveLoadManager::new();
        assert!(manager.is_ok());
    }

    #[test]
    fn test_save_file_path() {
        let manager = SaveLoadManager::new().unwrap();
        let path = manager.get_save_file_path("test");
        assert!(path.to_string_lossy().ends_with("test.json"));
    }

    #[test]
    fn test_format_timestamp() {
        let timestamp = 1640995200; // 2022-01-01 00:00:00 UTC
        let formatted = SaveLoadManager::format_timestamp(timestamp);
        assert!(formatted.contains("2022-01-01"));
    }
}
