use crate::utils::display::{header, show_error, show_info, show_success, Icon};
use crate::utils::input;
use crate::utils::table::{Table, LEFT, RIGHT};
use colored::*;
use elite_core::data::save_load::{SaveLoadManager, SaveMetadata};
use elite_core::game::save_load::SaveError;
use elite_core::game::GameState;
use std::io::Write;

/// `save <slot>`
pub fn save(game: &GameState, args: &[String]) -> bool {
    let Some(slot) = args.first() else {
        show_error("Usage: save <slot_name>");
        return false;
    };
    match game.save_load_handler.save(game, slot) {
        Ok(()) => {
            show_success(&format!(
                "Game saved successfully! Commander {}'s progress has been saved to slot '{}'.",
                game.commander().name(),
                slot
            ));
            true
        }
        Err(error) => {
            show_error(&error.to_string());
            false
        }
    }
}

/// `load <slot>`: a loaded game replaces the current one
pub fn load(game: &GameState, args: &[String]) -> Result<Option<GameState>, String> {
    let Some(slot) = args.first() else {
        show_error("Usage: load <slot_name>");
        return Ok(None);
    };
    let handler = &game.save_load_handler;

    let metadata = match handler.preview(slot) {
        Ok(metadata) => metadata,
        Err(error @ SaveError::NotFound(_)) => {
            show_error(&error.to_string());
            return Ok(None);
        }
        Err(error) => return Err(error.to_string()),
    };
    print_save_info(&metadata);

    if !confirm("Would you like to load this save? (y/n): ") {
        return Ok(None);
    }
    match handler.load(slot) {
        Ok(loaded) => {
            show_success(&format!(
                "Game loaded successfully! Commander {}, your journey continues!",
                loaded.commander().name()
            ));
            Ok(Some(loaded))
        }
        Err(error) => {
            show_error(&error.to_string());
            Ok(None)
        }
    }
}

/// `saves`
pub fn list(game: &GameState) {
    match game.save_load_handler.list_slots() {
        Ok(saves) if saves.is_empty() => {
            println!("\n{}", "No save files found.".yellow());
            println!("Use 'save <slot_name>' to create your first save.");
        }
        Ok(saves) => {
            header(Icon::Save, "Available Save Files");

            let mut table = Table::new(&[
                ("#", RIGHT),
                ("Slot", LEFT),
                ("Commander", LEFT),
                ("Saved", LEFT),
                ("Location", LEFT),
                ("Credits", RIGHT),
                ("Ship", LEFT),
            ]);
            for (i, (slot, save)) in saves.iter().enumerate() {
                table.row([
                    (i + 1).to_string(),
                    slot.clone(),
                    save.commander_name.yellow().to_string(),
                    SaveLoadManager::format_timestamp(save.timestamp)
                        .blue()
                        .to_string(),
                    save.location.cyan().to_string(),
                    format!("{}cr", save.credits).green().to_string(),
                    save.ship_type.magenta().to_string(),
                ]);
            }
            table.print();
            println!();
        }
        Err(error) => show_error(&error.to_string()),
    }
}

/// `delete <slot>`
pub fn delete(game: &GameState, args: &[String]) -> bool {
    let Some(slot) = args.first() else {
        show_error("Usage: delete <slot_name>");
        return false;
    };
    let handler = &game.save_load_handler;

    match handler.preview(slot) {
        Ok(metadata) => print_save_info(&metadata),
        Err(error @ SaveError::NotFound(_)) => {
            show_error(&error.to_string());
            return false;
        }
        Err(_) => {
            show_error(&format!("Failed to load save metadata for '{}'", slot));
            return false;
        }
    }

    if !confirm_delete() {
        return false;
    }
    match handler.delete(slot) {
        Ok(()) => {
            show_success(&format!("Save file '{}' deleted successfully.", slot));
            true
        }
        Err(error) => {
            show_error(&error.to_string());
            false
        }
    }
}

/// Write the autosave on quit
pub fn auto_save(game: &GameState) -> bool {
    match game.save_load_handler.auto_save(game) {
        Ok(()) => {
            println!("{}", "Auto-saving game state...".yellow());
            show_success(&format!(
                "Game saved successfully! Commander {}'s progress has been saved.",
                game.commander().name()
            ));
            true
        }
        Err(error) => {
            show_error(&error.to_string());
            false
        }
    }
}

fn confirm(prompt: &str) -> bool {
    print!("{}", prompt);
    std::io::stdout().flush().expect("Failed to flush stdout");
    match input::get_input() {
        Ok(answer) => {
            let answer = answer.trim().to_lowercase();
            if answer == "y" || answer == "yes" {
                true
            } else {
                show_info("Load cancelled.");
                false
            }
        }
        Err(_) => {
            show_error("Error reading input. Load cancelled.");
            false
        }
    }
}

fn confirm_delete() -> bool {
    print!("Are you sure you want to delete this save? (y/n): ");
    std::io::stdout().flush().expect("Failed to flush stdout");
    match input::get_input() {
        Ok(answer) => {
            let answer = answer.trim().to_lowercase();
            if answer == "y" || answer == "yes" {
                true
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
}

fn print_save_info(metadata: &SaveMetadata) {
    let timestamp = SaveLoadManager::format_timestamp(metadata.timestamp);

    header(Icon::Save, "Save Information");
    println!("Commander: {}", metadata.commander_name.green());
    println!("Location: {}", metadata.location.cyan());
    println!("Credits: {}cr", metadata.credits.to_string().green());
    println!("Ship: {}", metadata.ship_type.magenta());
    println!("Saved: {}", timestamp.blue());
    println!("Version: {}", metadata.version.yellow());
    println!();
}
