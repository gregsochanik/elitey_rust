use crate::game::{GameState, SaveLoadHandler};
use crate::utils::display::show_error;
use crate::utils::input;
use colored::*;
use std::io::Write;

/// Start a new game, asking before replacing an existing autosave
pub fn new_game() -> GameState {
    start(true)
}

/// Continue the commander's autosave if there is one, else start a new game
pub fn load_or_new() -> GameState {
    start(false)
}

fn start(force_new: bool) -> GameState {
    println!("{}", "Welcome to Elite Rust!".green().bold());
    println!("{}", "=====================".green());

    let name = commander_name();
    let handler = SaveLoadHandler::new().expect("Failed to create save/load handler");
    match handler.find_autosave(&name) {
        Ok(None) => GameState::new(name),
        Ok(Some(saved)) => {
            let summary = format!(
                "Found a save for {} at {} with {} credits.",
                saved.commander.name().green(),
                saved.commander.location().cyan(),
                saved.commander.credits().to_string().green()
            );
            let keep = if force_new {
                !ask_yes_no(&format!("{} Start over and replace it? (y/n): ", summary))
                    .unwrap_or(false)
            } else {
                ask_yes_no(&format!("{} Continue it? (y/n): ", summary)).unwrap_or(true)
            };
            if keep {
                saved
            } else {
                GameState::new(name)
            }
        }
        Err(e) => {
            show_error(&format!("Could not read the save for {}: {}", name, e));
            let replace = ask_yes_no("Start a new game and replace it? (y/n): ").unwrap_or(false);
            if !replace {
                std::process::exit(1);
            }
            GameState::new(name)
        }
    }
}

/// Ask until the answer is yes or no; `None` when input has ended
fn ask_yes_no(prompt: &str) -> Option<bool> {
    loop {
        print!("{}", prompt);
        std::io::stdout().flush().expect("Failed to flush stdout");
        let answer = input::get_input().ok().filter(|line| !line.is_empty())?;
        match answer.trim().to_lowercase().as_str() {
            "y" | "yes" => return Some(true),
            "n" | "no" => return Some(false),
            _ => println!("Please answer y or n."),
        }
    }
}

fn commander_name() -> String {
    loop {
        print!("Enter your commander name: ");
        std::io::stdout().flush().expect("Failed to flush stdout");

        match input::get_input() {
            Ok(input) => {
                let name = input.trim();
                if !name.is_empty() {
                    return name.to_string();
                } else {
                    println!("{}", "Name cannot be empty. Please try again.".red());
                }
            }
            Err(_) => {
                println!("{}", "Error reading input. Please try again.".red());
            }
        }
    }
}
