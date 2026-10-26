use crate::cli::{equipment, info, jobs, menu, saves, trading, travel, upgrades, upkeep};
use crate::utils::display;
use colored::*;
use elite_core::game::GameState;

/// What the game loop does after a command
pub enum Control {
    Continue,
    /// A saved game was loaded to replace the current one
    Replace(Box<GameState>),
    Quit,
}

/// Run one command line's command with its arguments
pub fn execute(game: &mut GameState, command: &str, args: &[String]) -> Control {
    let command = menu::resolve(command);
    match command.as_str() {
        "help" => menu::show_help(),
        "status" => info::status(game),
        "market" => trading::market(game),
        "buy" => {
            trading::buy(game, args);
        }
        "sell" => {
            trading::sell(game, args);
        }
        "cargo" => trading::cargo(game),
        "equipment" => {
            equipment::command(game, args);
        }
        "jobs" => jobs::board(game),
        "accept" => jobs::accept(game, args),
        "missions" => jobs::missions(game),
        "abandon" => jobs::abandon(game, args),
        "refuel" => {
            upkeep::refuel(game, args);
        }
        "repair" => {
            upkeep::repair(game, args);
        }
        "log" => info::log(game, args),
        "upgrade" => {
            if args.is_empty() {
                upgrades::show_upgrades(game);
            } else {
                upgrades::buy(game, args);
            }
        }
        "ships" => upgrades::show_ships(game),
        "save" => {
            saves::save(game, args);
        }
        "load" => match saves::load(game, args) {
            Ok(Some(loaded)) => return Control::Replace(Box::new(loaded)),
            Ok(None) => {}
            Err(e) => println!("{}", format!("Load error: {}", e).red()),
        },
        "saves" => saves::list(game),
        "delete" => {
            saves::delete(game, args);
        }
        "jump" => {
            travel::jump(game, args);
        }
        "route" => {
            travel::route(game, args);
        }
        "discoveries" => info::discoveries(game),
        "systems" => info::systems(game),
        "info" => {
            info::system_info(game, args);
        }
        "quit" | "exit" => {
            saves::auto_save(game);
            display::print_goodbye(&game.commander.name);
            return Control::Quit;
        }
        _ => println!(
            "{}",
            "Unknown command. Type 'help' for available commands.".red()
        ),
    }
    Control::Continue
}
