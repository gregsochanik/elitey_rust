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
