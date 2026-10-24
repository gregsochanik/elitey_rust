use anyhow::Result;
use clap::Parser;
use colored::*;
use std::io::{self, Write};

mod data;
mod game;
mod utils;

use game::GameState;
use utils::display;
use utils::input;

#[derive(Parser)]
#[command(name = "Elite Rust")]
#[command(about = "Elite-style terminal trading game")]
#[command(version)]
struct Args {
    /// Start a new game
    #[arg(short, long)]
    new: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize game state
    let mut game_state = if args.new {
        GameState::new_game()
    } else {
        GameState::load_or_new()
    };

    // Display welcome message
    display::print_welcome(&game_state.commander.name);

    // Main game loop
    loop {
        // Display prompt
        print!("{} ", ">".green());
        io::stdout().flush()?;

        // Get user input
        let input = input::get_input()?;
        let end_of_input = input.is_empty();
        let command_parts: Vec<&str> = input.split_whitespace().collect();

        // Process command
        if command_parts.is_empty() && !end_of_input {
            continue;
        }

        let command = if end_of_input {
            "quit".to_string()
        } else {
            command_parts[0].to_lowercase()
        };

        match command.as_str() {
            "help" => {
                display::show_help();
            }
            "status" => {
                display::show_status(&game_state);
            }

            "market" => {
                game_state.handle_market_command();
            }
            "buy" => {
                let args: Vec<String> = command_parts
                    .iter()
                    .skip(1)
                    .map(|s| s.to_string())
                    .collect();
                game_state.handle_buy_command(&args);
            }
            "sell" => {
                let args: Vec<String> = command_parts
                    .iter()
                    .skip(1)
                    .map(|s| s.to_string())
                    .collect();
                game_state.handle_sell_command(&args);
            }
            "cargo" => {
                game_state.handle_cargo_command();
            }
            "equipment" => {
                let args: Vec<String> = command_parts
                    .iter()
                    .skip(1)
                    .map(|s| s.to_string())
                    .collect();
                game_state.handle_equipment_command(&args);
            }
            "jobs" => {
                game_state.handle_jobs_command();
            }
            "accept" => {
                let args: Vec<String> = command_parts
                    .iter()
                    .skip(1)
                    .map(|s| s.to_string())
                    .collect();
                game_state.handle_accept_command(&args);
            }
            "missions" => {
                game_state.handle_missions_command();
            }
            "abandon" => {
                let args: Vec<String> = command_parts
                    .iter()
                    .skip(1)
                    .map(|s| s.to_string())
                    .collect();
                game_state.handle_abandon_command(&args);
            }
            "refuel" => {
                let args: Vec<String> = command_parts
                    .iter()
                    .skip(1)
                    .map(|s| s.to_string())
                    .collect();
                game_state.handle_refuel_command(&args);
            }
            "repair" => {
                let args: Vec<String> = command_parts
                    .iter()
                    .skip(1)
                    .map(|s| s.to_string())
                    .collect();
                game_state.handle_repair_command(&args);
            }
            "log" => {
                let args: Vec<String> = command_parts
                    .iter()
                    .skip(1)
                    .map(|s| s.to_string())
                    .collect();
                game_state.handle_log_command(&args);
            }
            "upgrade" => {
                if command_parts.len() == 1 {
                    game_state.handle_upgrade_command();
                } else {
                    let args: Vec<String> = command_parts
                        .iter()
                        .skip(1)
                        .map(|s| s.to_string())
                        .collect();
                    game_state.handle_upgrade_purchase_command(&args);
                }
            }
            "ships" => {
                game_state.handle_ships_command();
            }
            "save" => {
                let args: Vec<String> = command_parts
                    .iter()
                    .skip(1)
                    .map(|s| s.to_string())
                    .collect();
                game_state.handle_save_command(&args);
            }
            "load" => {
                let args: Vec<String> = command_parts
                    .iter()
                    .skip(1)
                    .map(|s| s.to_string())
                    .collect();
                match game_state.handle_load_command(&args) {
                    Ok(Some(new_game_state)) => {
                        game_state = new_game_state;
                    }
                    Ok(None) => {
                        // Load was cancelled or failed, continue with current game
                    }
                    Err(e) => {
                        println!("{}", format!("Load error: {}", e).red());
                    }
                }
            }
            "saves" => {
                game_state.handle_list_saves_command();
            }
            "delete" => {
                let args: Vec<String> = command_parts
                    .iter()
                    .skip(1)
                    .map(|s| s.to_string())
                    .collect();
                game_state.handle_delete_save_command(&args);
            }
            "jump" => {
                let args: Vec<String> = command_parts
                    .iter()
                    .skip(1)
                    .map(|s| s.to_string())
                    .collect();
                game_state.handle_jump_command(&args);
            }
            "systems" => {
                game_state.handle_systems_command();
            }
            "info" => {
                let args: Vec<String> = command_parts
                    .iter()
                    .skip(1)
                    .map(|s| s.to_string())
                    .collect();
                game_state.handle_info_command(&args);
            }
            "quit" | "exit" => {
                // Auto-save before quitting
                game_state.handle_auto_save();
                display::print_goodbye(&game_state.commander.name);
                break;
            }
            _ => {
                println!(
                    "{}",
                    "Unknown command. Type 'help' for available commands.".red()
                );
            }
        }
    }

    Ok(())
}
