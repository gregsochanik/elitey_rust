use anyhow::Result;
use clap::Parser;
use colored::*;
use std::io::{self, Write};

mod game;
mod utils;
mod data;

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

#[tokio::main]
async fn main() -> Result<()> {
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
        let command_parts: Vec<&str> = input.split_whitespace().collect();

        // Process command
        if command_parts.is_empty() {
            continue;
        }

        match command_parts[0].to_lowercase().as_str() {
            "help" => {
                display::show_help();
            }
            "status" => {
                display::show_status(&game_state);
            }
            "systems" => {
                display::show_systems(&game_state);
            }
            "info" => {
                if command_parts.len() < 2 {
                    println!("{}", "Usage: info <system_name>".red());
                } else {
                    let system_name = command_parts[1];
                    if let Some(system) = game_state.get_system_info(system_name) {
                        display::show_system_info(system, &game_state);
                    } else {
                        println!("{}", format!("Unknown system: {}", system_name).red());
                    }
                }
            }
            "market" => {
                game_state.handle_market_command();
            }
            "buy" => {
                let args: Vec<String> = command_parts.iter().map(|s| s.to_string()).collect();
                game_state.handle_buy_command(&args);
            }
            "sell" => {
                let args: Vec<String> = command_parts.iter().map(|s| s.to_string()).collect();
                game_state.handle_sell_command(&args);
            }
            "cargo" => {
                game_state.handle_cargo_command();
            }
            "upgrade" => {
                if command_parts.len() == 1 {
                    game_state.handle_upgrade_command();
                } else {
                    let args: Vec<String> = command_parts.iter().map(|s| s.to_string()).collect();
                    game_state.handle_upgrade_purchase_command(&args);
                }
            }
            "ships" => {
                game_state.handle_ships_command();
            }
            "save" => {
                let args: Vec<String> = command_parts.iter().map(|s| s.to_string()).collect();
                game_state.handle_save_command(&args);
            }
            "load" => {
                let args: Vec<String> = command_parts.iter().map(|s| s.to_string()).collect();
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
                let args: Vec<String> = command_parts.iter().map(|s| s.to_string()).collect();
                game_state.handle_delete_save_command(&args);
            }
            "jump" => {
                if command_parts.len() < 2 {
                    println!("{}", "Usage: jump <system_name>".red());
                } else {
                    let destination = command_parts[1];
                    
                    // Check if we can travel there
                    if let Some(fuel_cost) = game_state.calculate_travel_cost(destination) {
                        display::show_travel_confirmation(&game_state.commander.location, destination, fuel_cost);
                        
                        // Get confirmation
                        print!("Confirm travel? (y/n): ");
                        io::stdout().flush()?;
                        
                        if let Ok(input) = input::get_input() {
                            if input.trim().to_lowercase() == "y" || input.trim().to_lowercase() == "yes" {
                                match game_state.travel_to_system(destination) {
                                    Ok(_) => {
                                        println!("{}", format!("Successfully jumped to {}!", destination).green());
                                        println!("{}", "Fuel consumed during travel.".yellow());
                                    }
                                    Err(e) => {
                                        println!("{}", format!("Travel failed: {}", e).red());
                                    }
                                }
                            } else {
                                println!("{}", "Travel cancelled.".yellow());
                            }
                        }
                    } else {
                        println!("{}", format!("Unknown system: {}", destination).red());
                    }
                }
            }
            "quit" | "exit" => {
                // Auto-save before quitting
                game_state.handle_auto_save();
                display::print_goodbye(&game_state.commander.name);
                break;
            }
            _ => {
                println!("{}", "Unknown command. Type 'help' for available commands.".red());
            }
        }
    }

    Ok(())
}
