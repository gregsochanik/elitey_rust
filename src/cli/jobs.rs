use crate::game::game_state::JobListing;
use crate::game::jobs::{self, ActiveJob};
use crate::game::GameState;
use crate::utils::display::{show_error, show_info, show_success};
use colored::*;

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
    println!("\n{}", "Active Jobs".cyan().bold());
    println!("{}", "===========".cyan());
    if active.is_empty() {
        println!("No active jobs. Use 'jobs' to see the board.");
    }
    for (index, entry) in active.iter().enumerate() {
        println!(
            "{}. {} - Reward: {}cr - {} left",
            index + 1,
            entry.job.describe().yellow(),
            entry.job.reward.to_string().green(),
            jumps(entry.jumps_left)
        );
    }
    println!();
}

fn print_jobs(location: &str, jobs: &[JobListing]) {
    println!("\n{}", format!("Jobs at {}", location).cyan().bold());
    println!("{}", "=".repeat(location.len() + 8).cyan());
    if jobs.is_empty() {
        println!("No jobs on offer here.");
    }
    for (index, listing) in jobs.iter().enumerate() {
        let job = &listing.job;
        let line = format!(
            "{}. {} - {:.1} light years, {} fuel - Reward: {}cr - Deadline: {}",
            index + 1,
            job.describe(),
            job.distance,
            job.fuel_cost,
            job.reward,
            jumps(job.deadline_jumps)
        );
        match &listing.lock {
            Some(lock) => println!(
                "{} {}",
                line.dimmed(),
                format!("[Locked: {}]", lock.describe()).red()
            ),
            None => println!(
                "{}. {} - {:.1} light years, {} fuel - Reward: {}cr - Deadline: {}",
                index + 1,
                job.describe().yellow(),
                job.distance,
                job.fuel_cost,
                job.reward.to_string().green(),
                jumps(job.deadline_jumps)
            ),
        }
    }
    println!();
}
