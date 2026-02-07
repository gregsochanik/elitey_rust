use crate::data::save_load::{SaveLoadManager, SaveMetadata};
use crate::game::GameState;
use crate::utils::display::{show_error, show_success, show_info};
use crate::utils::input;
use colored::*;
use std::io::Write;

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

    /// Handle save command
    pub fn handle_save(&self, args: &[String], game_state: &GameState) -> bool {
        if args.is_empty() {
            show_error("Usage: save <slot_name>");
            return false;
        }

        let slot_name = &args[0];
        
        // Validate slot name (alphanumeric and underscores only)
        if !slot_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            show_error("Slot name must contain only letters, numbers, and underscores");
            return false;
        }

        match self.save_manager.save_game(game_state, slot_name) {
            Ok(_) => {
                show_success(&format!(
                    "Game saved successfully! Commander {}'s progress has been saved to slot '{}'.",
                    game_state.commander().name(),
                    slot_name
                ));
                true
            }
            Err(e) => {
                show_error(&format!("Failed to save game: {}", e));
                false
            }
        }
    }

    /// Handle load command
    pub fn handle_load(&self, args: &[String]) -> Result<Option<GameState>, String> {
        if args.is_empty() {
            show_error("Usage: load <slot_name>");
            return Ok(None);
        }

        let slot_name = &args[0];
        
        if !self.save_manager.save_exists(slot_name) {
            show_error(&format!("Save file '{}' not found", slot_name));
            return Ok(None);
        }

        // Load metadata first to show confirmation
        let metadata = self.save_manager.load_save_metadata(slot_name)
            .map_err(|e| format!("Failed to load save metadata: {}", e))?;

        // Show save information and ask for confirmation
        self.display_save_info(&metadata);
        
        print!("Would you like to load this save? (y/n): ");
        std::io::stdout().flush().expect("Failed to flush stdout");
        
        match input::get_input() {
            Ok(input) => {
                let response = input.trim().to_lowercase();
                if response == "y" || response == "yes" {
                    match self.save_manager.load_game(slot_name) {
                        Ok(game_state) => {
                            show_success(&format!(
                                "Game loaded successfully! Commander {}, your journey continues!",
                                game_state.commander().name()
                            ));
                            Ok(Some(game_state))
                        }
                        Err(e) => {
                            show_error(&format!("Failed to load game: {}", e));
                            Ok(None)
                        }
                    }
                } else {
                    show_info("Load cancelled.");
                    Ok(None)
                }
            }
            Err(_) => {
                show_error("Error reading input. Load cancelled.");
                Ok(None)
            }
        }
    }

    /// Handle list saves command
    pub fn handle_list_saves(&self) {
        match self.save_manager.list_saves() {
            Ok(saves) => {
                if saves.is_empty() {
                    println!("\n{}", "No save files found.".yellow());
                    println!("Use 'save <slot_name>' to create your first save.");
                } else {
                    println!("\n{}", "Available Save Files".cyan().bold());
                    println!("{}", "===================".cyan());
                    
                    for (i, save) in saves.iter().enumerate() {
                        let slot_name = self.get_slot_name_from_metadata(save);
                        let timestamp = SaveLoadManager::format_timestamp(save.timestamp);
                        
                        println!("{}. {} ({})", i + 1, slot_name.yellow(), timestamp.blue());
                        println!("   Commander: {}", save.commander_name.green());
                        println!("   Location: {}", save.location.cyan());
                        println!("   Credits: {}cr", save.credits.to_string().green());
                        println!("   Ship: {}", save.ship_type.magenta());
                        println!();
                    }
                }
            }
            Err(e) => {
                show_error(&format!("Failed to list saves: {}", e));
            }
        }
    }

    /// Handle delete save command
    pub fn handle_delete_save(&self, args: &[String]) -> bool {
        if args.is_empty() {
            show_error("Usage: delete <slot_name>");
            return false;
        }

        let slot_name = &args[0];
        
        if !self.save_manager.save_exists(slot_name) {
            show_error(&format!("Save file '{}' not found", slot_name));
            return false;
        }

        // Load metadata to show what we're deleting
        if let Ok(metadata) = self.save_manager.load_save_metadata(slot_name) {
            self.display_save_info(&metadata);
            
            print!("Are you sure you want to delete this save? (y/n): ");
            std::io::stdout().flush().expect("Failed to flush stdout");
            
            match input::get_input() {
                Ok(input) => {
                    let response = input.trim().to_lowercase();
                    if response == "y" || response == "yes" {
                        match self.save_manager.delete_save(slot_name) {
                            Ok(_) => {
                                show_success(&format!("Save file '{}' deleted successfully.", slot_name));
                                true
                            }
                            Err(e) => {
                                show_error(&format!("Failed to delete save: {}", e));
                                false
                            }
                        }
                    } else {
                        show_info("Delete cancelled.");
                        false
                    }
                }
                Err(_) => {
                    show_error("Error reading input. Delete cancelled.");
                    false
                }
            }
        } else {
            show_error(&format!("Failed to load save metadata for '{}'", slot_name));
            false
        }
    }

    /// Handle auto-save on quit
    pub fn auto_save(&self, game_state: &GameState) -> bool {
        let slot_name = "autosave";
        
        match self.save_manager.save_game(game_state, slot_name) {
            Ok(_) => {
                println!("{}", "Auto-saving game state...".yellow());
                show_success(&format!(
                    "Game saved successfully! Commander {}'s progress has been saved.",
                    game_state.commander().name()
                ));
                true
            }
            Err(e) => {
                show_error(&format!("Auto-save failed: {}", e));
                false
            }
        }
    }

    /// Check for existing save on game start
    #[allow(dead_code)]
    pub fn check_existing_save(&self, commander_name: &str) -> Result<Option<GameState>, String> {
        // Look for a save with this commander name
        let saves = self.save_manager.list_saves()?;
        
        for save in saves {
            if save.commander_name == commander_name {
                // Found existing save for this commander
                self.display_save_info(&save);
                
                print!("Found existing save for {} at {} with {} credits.\nWould you like to load this save? (y/n): ", 
                       save.commander_name.green(),
                       save.location.cyan(),
                       save.credits.to_string().green());
                std::io::stdout().flush().expect("Failed to flush stdout");
                
                match input::get_input() {
                    Ok(input) => {
                        let response = input.trim().to_lowercase();
                        if response == "y" || response == "yes" {
                            let slot_name = self.get_slot_name_from_metadata(&save);
                            return self.save_manager.load_game(&slot_name).map(Some);
                        } else {
                            return Ok(None);
                        }
                    }
                    Err(_) => {
                        return Ok(None);
                    }
                }
            }
        }
        
        Ok(None)
    }

    /// Display save information
    fn display_save_info(&self, metadata: &SaveMetadata) {
        let timestamp = SaveLoadManager::format_timestamp(metadata.timestamp);
        
        println!("\n{}", "Save Information".cyan().bold());
        println!("{}", "================".cyan());
        println!("Commander: {}", metadata.commander_name.green());
        println!("Location: {}", metadata.location.cyan());
        println!("Credits: {}cr", metadata.credits.to_string().green());
        println!("Ship: {}", metadata.ship_type.magenta());
        println!("Saved: {}", timestamp.blue());
        println!("Version: {}", metadata.version.yellow());
        println!();
    }

    /// Get slot name from metadata (this is a simplified approach)
    fn get_slot_name_from_metadata(&self, metadata: &SaveMetadata) -> String {
        // In a real implementation, we'd need to store the slot name in metadata
        // For now, we'll use the commander name as the slot name
        metadata.commander_name.clone()
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
}
