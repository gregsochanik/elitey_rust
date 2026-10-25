use crate::utils::display::{header, icon, show_error, show_info, show_success, Icon};
use crate::utils::table::{Table, LEFT, RIGHT};
use colored::*;
use elite_core::game::game_state::JobListing;
use elite_core::game::jobs::{self, ActiveJob};
use elite_core::game::GameState;

/// `jobs`: the board here
pub fn board(game: &mut GameState) {
    match game.jobs_here(&mut rand::thread_rng()) {
        Some(jobs) => print_jobs(&game.commander.location, &jobs),
        None => show_error("Unable to determine current location"),
    }
}

/// `missions`: the jobs being worked on
pub fn missions(game: &GameState) {
    print_missions(&game.active_jobs);
}

/// `accept <number>`
pub fn accept(game: &mut GameState, args: &[String]) {
    let Some(number) = job_number(args, "accept") else {
        return;
    };
    match game.accept_job(number) {
        Ok(job) => show_success(&format!(
            "Accepted: {} for {}cr within {} jumps.",
            job.describe(),
            job.reward,
            job.deadline_jumps
        )),
        Err(e) => show_error(&e),
    }
}

/// `abandon <number>`
pub fn abandon(game: &mut GameState, args: &[String]) {
    let Some(number) = job_number(args, "abandon") else {
        return;
    };
    let standing = game.commander.standing();
    match game.abandon_job(number) {
        Ok(job) => {
            if game.commander.standing() != standing {
                show_info(&format!(
                    "Your standing is now {}.",
                    game.commander.standing().name()
                ));
            }
            println!(
                "{}",
                format!(
                    "Abandoned: {}. Reputation {:+}.",
                    job.describe(),
                    jobs::ABANDON_REPUTATION
                )
                .yellow()
            );
        }
        Err(e) => show_error(&e),
    }
}

fn job_number(args: &[String], command: &str) -> Option<usize> {
    match args.first().map(|arg| arg.parse::<usize>()) {
        Some(Ok(number)) if number > 0 && args.len() == 1 => Some(number),
        _ => {
            show_error(&format!("Usage: {} <number>", command));
            None
        }
    }
}

fn jumps(count: u32) -> String {
    format!("{} jump{}", count, if count == 1 { "" } else { "s" })
}

fn print_missions(active: &[ActiveJob]) {
    header(Icon::Jobs, "Active Jobs");
    if active.is_empty() {
        println!("No active jobs. Use 'jobs' to see the board.");
    }
    if !active.is_empty() {
        let mut table = Table::new(&[
            ("#", RIGHT),
            ("Job", LEFT),
            ("Reward", RIGHT),
            ("Time left", RIGHT),
        ]);
        for (index, entry) in active.iter().enumerate() {
            table.row([
                (index + 1).to_string(),
                entry.job.describe().yellow().to_string(),
                format!("{}cr", entry.job.reward).green().to_string(),
                jumps(entry.jumps_left),
            ]);
        }
        table.print();
    }
    println!();
}

fn print_jobs(location: &str, jobs: &[JobListing]) {
    header(Icon::Jobs, &format!("Jobs at {}", location));
    if jobs.is_empty() {
        println!("No jobs on offer here.");
    }
    if jobs.is_empty() {
        println!();
        return;
    }
    let mut table = Table::new(&[
        ("#", RIGHT),
        ("Job", LEFT),
        ("Distance", RIGHT),
        ("Fuel", RIGHT),
        ("Reward", RIGHT),
        ("Deadline", RIGHT),
        ("", LEFT),
    ]);
    for (index, listing) in jobs.iter().enumerate() {
        let job = &listing.job;
        let row = [
            (index + 1).to_string(),
            job.describe(),
            format!("{:.1} ly", job.distance),
            job.fuel_cost.to_string(),
            format!("{}cr", job.reward),
            jumps(job.deadline_jumps),
        ];
        match &listing.lock {
            Some(lock) => {
                let mut cells: Vec<String> =
                    row.iter().map(|cell| cell.dimmed().to_string()).collect();
                cells.push(
                    format!("[{}Locked: {}]", icon(Icon::Locked), lock.describe())
                        .red()
                        .to_string(),
                );
                table.row(cells);
            }
            None => {
                let mut cells = row.to_vec();
                cells[1] = cells[1].yellow().to_string();
                cells[4] = cells[4].green().to_string();
                cells.push(String::new());
                table.row(cells);
            }
        }
    }
    table.print();
    println!();
}
