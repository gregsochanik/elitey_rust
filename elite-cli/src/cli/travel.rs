use crate::cli::fight::run_fight;
use crate::utils::display::{header, show_error, show_info, show_success, Icon};
use crate::utils::input;
use crate::utils::table::{Table, LEFT, RIGHT};
use colored::*;
use elite_core::game::combat::Combat;
use elite_core::game::events::Event;
use elite_core::game::journey::{Input, Journey, Notice, Step};
use elite_core::game::route::Route;
use elite_core::game::smuggling::InspectionChoice;
use elite_core::game::travel::JumpPlan;
use elite_core::game::GameState;
use std::io::{self, Write};

/// `jump <system>`: confirm, then play the journey through its events, fights and arrival
pub fn jump(game: &mut GameState, args: &[String]) -> bool {
    if args.is_empty() {
        show_error("Usage: jump <system_name>");
        return false;
    }
    let destination = args.join(" ");

    let plan = match game.plan_jump(&destination) {
        Ok(plan) => plan,
        Err(error) => {
            show_error(&error.to_string());
            return false;
        }
    };

    print_confirmation(&plan);
    match input::get_input() {
        Ok(answer) => {
            let answer = answer.trim().to_lowercase();
            if answer != "y" && answer != "yes" {
                show_info("Travel cancelled.");
                return false;
            }
        }
        Err(_) => {
            show_error("Error reading input. Travel cancelled.");
            return false;
        }
    }

    let mut rng = rand::thread_rng();
    let (mut journey, mut step) = game.start_jump(plan, &mut rng);
    loop {
        print_notices(&mut journey);
        let answer = match step {
            Step::Arrived { .. } => return true,
            Step::Event(event) => Input::Choice(ask_event_choice(&event)),
            Step::Fight(combat) => play_fight(combat),
            Step::Inspection {
                bribe_cost,
                can_bribe,
            } => Input::Inspection(ask_inspection_choice(bribe_cost, can_bribe)),
        };
        step = game
            .advance_journey(&mut journey, answer, &mut rng)
            .expect("answers from the terminal are always valid");
    }
}

fn print_notices(journey: &mut Journey) {
    for notice in journey.take_notices() {
        match notice {
            Notice::Blank => println!(),
            Notice::Text(text) => println!("{}", text),
            Notice::Info(text) => show_info(&text),
            Notice::Success(text) => show_success(&text),
            Notice::Error(text) => show_error(&text),
        }
    }
}

fn print_confirmation(plan: &JumpPlan) {
    header(Icon::Ship, "Travel Confirmation");
    println!("From: {}", plan.origin.yellow());
    println!("To: {}", plan.destination.yellow());
    println!("Distance: {:.1} light years", plan.distance);
    println!("{}", "Confirm travel? (y/n): ".cyan());
}

/// Show the event and read a choice; anything unusable defaults to the first option
fn ask_event_choice(event: &Event) -> usize {
    println!();
    println!("{}", "=".repeat(50));
    println!("{}", event.title);
    println!("{}", "=".repeat(50));
    println!("{}", event.description);
    println!();

    for (i, choice) in event.choices.iter().enumerate() {
        println!("{}. {}", i + 1, choice.description);
    }
    println!();
    print!("Choose your action (1-{}): ", event.choices.len());

    match input::get_input() {
        Ok(answer) => match answer.trim().parse::<usize>() {
            Ok(number) if number >= 1 && number <= event.choices.len() => number - 1,
            Ok(_) => {
                show_error("Invalid choice. Defaulting to first option.");
                0
            }
            Err(_) => {
                show_error("Invalid input. Defaulting to first option.");
                0
            }
        },
        Err(_) => {
            show_error("Error reading input. Defaulting to first option.");
            0
        }
    }
}

fn play_fight(mut combat: Combat) -> Input {
    let mut dice = rand::thread_rng();
    let mut roll = || rand::Rng::gen::<f64>(&mut dice);
    match run_fight(
        &mut combat,
        &mut io::stdin().lock(),
        &mut io::stdout(),
        &mut roll,
    ) {
        Ok(_) => Input::FightEnded(combat),
        Err(error) => Input::FightInterrupted(error.to_string()),
    }
}

/// Ask what to do about an inspection; running out of input hands the cargo over
fn ask_inspection_choice(bribe_cost: u32, can_bribe: bool) -> InspectionChoice {
    loop {
        if can_bribe {
            print!(
                "[b]ribe ({}cr), [r]un or [h]and over the cargo? ",
                bribe_cost
            );
        } else {
            print!(
                "[r]un or [h]and over the cargo (you cannot afford the {}cr bribe)? ",
                bribe_cost
            );
        }
        io::stdout().flush().expect("Failed to flush stdout");
        let line = match input::get_input() {
            Ok(line) if !line.is_empty() => line,
            _ => return InspectionChoice::HandOver,
        };
        match line.trim().to_lowercase().as_str() {
            "b" | "bribe" if can_bribe => return InspectionChoice::Bribe,
            "r" | "run" => return InspectionChoice::Run,
            "h" | "hand" | "hand over" => return InspectionChoice::HandOver,
            _ => println!("Please choose one of the options."),
        }
    }
}

/// `route <system>`: the cheapest-fuel way there, hop by hop
pub fn route(game: &GameState, args: &[String]) -> bool {
    if args.is_empty() {
        show_error("Usage: route <system_name>");
        return false;
    }
    let destination = args.join(" ");
    match game.plan_route(&destination) {
        Ok(route) => {
            print_route(&route);
            true
        }
        Err(error) => {
            show_error(&error.to_string());
            false
        }
    }
}

fn print_route(route: &Route) {
    let goal = route.hops.last().map_or("", |hop| hop.to.as_str());
    header(Icon::Ship, &format!("Route to {}", goal));
    let mut table = Table::new(&[
        ("#", RIGHT),
        ("From", LEFT),
        ("To", LEFT),
        ("Distance", RIGHT),
        ("Fuel", RIGHT),
        ("", LEFT),
    ]);
    for (index, hop) in route.hops.iter().enumerate() {
        let note = if route.refuel_at.contains(&hop.from) {
            format!("refuel at {} first", hop.from)
        } else {
            String::new()
        };
        table.row([
            (index + 1).to_string(),
            hop.from.clone(),
            hop.to.clone(),
            format!("{:.1} ly", hop.distance),
            hop.fuel.to_string(),
            note.yellow().to_string(),
        ]);
    }
    table.print();
    println!(
        "{} jumps, {:.1} light years, {} fuel",
        route.hops.len(),
        route.total_distance,
        route.total_fuel
    );
    println!();
}
