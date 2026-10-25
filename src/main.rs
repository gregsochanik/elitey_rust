use anyhow::Result;
use clap::Parser;
use std::io::{self, Write};

use colored::*;
use elitey_rust::cli::commands::{self, Control};
use elitey_rust::utils::display;
use elitey_rust::utils::input;

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
        elitey_rust::cli::startup::new_game()
    } else {
        elitey_rust::cli::startup::load_or_new()
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

        let args: Vec<String> = command_parts
            .iter()
            .skip(1)
            .map(|s| s.to_string())
            .collect();
        match commands::execute(&mut game_state, &command, &args) {
            Control::Continue => {}
            Control::Replace(loaded) => game_state = *loaded,
            Control::Quit => break,
        }
    }

    Ok(())
}
