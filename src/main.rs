use anyhow::Result;
use clap::Parser;
use colored::*;
use std::io::{self, Write};

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

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    // Initialize game state
    let game_state = if args.new {
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
        let command = input.trim().to_lowercase();

        // Process command
        match command.as_str() {
            "help" => {
                display::show_help();
            }
            "status" => {
                display::show_status(&game_state);
            }
            "quit" | "exit" => {
                display::print_goodbye(&game_state.commander.name);
                break;
            }
            "" => {
                // Empty input, continue
            }
            _ => {
                println!("{}", "Unknown command. Type 'help' for available commands.".red());
            }
        }
    }

    Ok(())
}
