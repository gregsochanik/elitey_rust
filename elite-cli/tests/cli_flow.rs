use std::io::Write;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

#[test]
fn the_game_saves_and_exits_when_input_ends() {
    let home = std::env::temp_dir().join(format!("elitey_rust_cli_{}", std::process::id()));
    std::fs::create_dir_all(&home).unwrap();

    let mut child = Command::new(env!("CARGO_BIN_EXE_elitey_rust"))
        .env("HOME", &home)
        .env("NO_COLOR", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"Tester\nstatus\n")
        .unwrap();

    let deadline = Instant::now() + Duration::from_secs(10);
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        if Instant::now() > deadline {
            child.kill().unwrap();
            panic!("game did not exit when input ended");
        }
        std::thread::sleep(Duration::from_millis(50));
    };

    let output = child.wait_with_output().unwrap();
    assert!(status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("Safe travels"));
}

fn play(label: &str, commands: &str) -> String {
    play_with(label, commands, &[("NO_COLOR", "1")])
}

fn play_with(label: &str, commands: &str, env: &[(&str, &str)]) -> String {
    let home =
        std::env::temp_dir().join(format!("elitey_rust_cli_{}_{}", label, std::process::id()));
    let _ = std::fs::remove_dir_all(&home);
    std::fs::create_dir_all(&home).unwrap();

    let mut child = Command::new(env!("CARGO_BIN_EXE_elitey_rust"))
        .arg("--new")
        .env("HOME", &home)
        .envs(env.iter().copied())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(format!("Tester\n{}\nquit\n", commands).as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn trading_commands_print_receipts_and_errors() {
    let output = play(
        "trading",
        "buy food 2\nbuy gold 1\nbuy food x\nbuy food 999\nsell food 5\ncargo",
    );

    assert!(output.contains("Success: Purchased 2 tons of Food for"));
    assert!(output.contains("Error: Unknown commodity: gold"));
    assert!(output.contains("Error: Quantity must be a positive number"));
    assert!(output.contains("Error: Only "));
    assert!(output.contains("Error: Insufficient cargo. Have 2 tons, trying to sell 5"));
    assert!(output.contains("Cargo Hold (2/20)"));
    assert!(output.contains("Paid/ton"));
}

#[test]
fn the_market_shows_what_held_cargo_cost() {
    let output = play("hold", "buy food 2\nmarket");

    assert!(output.contains("2t @ "));
}

#[test]
fn the_market_and_status_are_laid_out_as_before() {
    let output = play("layout", "market\nstatus");

    assert!(output.contains("Sol Market"));
    assert!(output.contains("Food"));
    assert!(output.contains("Commodity") && output.contains("Buy"));
    assert!(output.contains("vs Base"));
    assert!(output.contains("Commander Status"));
    assert!(output.contains("Combat rating: Harmless (0 kills, 3 more for Mostly Harmless)"));
    assert!(output.contains("Explored: 0% — Local (1 of 256 stars)"));
    assert!(output.contains("Fuel: 7/7 light years [##########]"));
    assert!(output.contains("Hull: 100/100 [##########]"));
    assert!(output.contains("Cargo: 0/20 tons [----------]"));
    assert!(output.contains("Power: 9/20"));
}

#[test]
fn power_is_shown_in_the_shop_and_the_installed_list_and_can_be_upgraded() {
    let output = play(
        "power",
        "equipment\nequipment buy improved-reactor\nequipment installed\nstatus\nequipment buy elite-reactor",
    );

    assert!(output.contains("Power: 9/20 used, 11 free"));
    assert!(output.contains("+28"));
    assert!(output.contains("Success: Installed Improved Reactor for 60cr"));
    assert!(output.contains("Power: 9/28 used, 19 free"));
    assert!(output.contains("Power: 9/28 [###-------]"));
    assert!(output.contains("Error: Elite Reactor needs tech level 11 (this system is 5)"));
}

#[test]
fn a_jump_asks_for_confirmation_and_can_be_cancelled() {
    let output = play("jump", "jump proxima centauri\nn\njump nowhere\njump");

    assert!(output.contains("Travel Confirmation"));
    assert!(output.contains("To: Proxima Centauri"));
    assert!(output.contains("Info: Travel cancelled."));
    assert!(output.contains("Error: System 'nowhere' not found"));
    assert!(output.contains("Error: Usage: jump <system_name>"));
}

#[test]
fn upkeep_and_equipment_commands_report_what_they_did() {
    let output = play(
        "upkeep",
        "repair\nrefuel\nrefuel x\nequipment buy nothing\nupgrade python\nupgrade cobra4",
    );

    assert!(output.contains("Info: Your hull is already at full strength."));
    assert!(output.contains("Info: Your tank is already full."));
    assert!(output.contains("Error: Usage: refuel [units]"));
    assert!(output.contains("Error: Unknown equipment: nothing"));
    assert!(output.contains("Error: The Python needs tech level 7 (this system is 5)"));
    assert!(output.contains("Error: Insufficient credits. Need 500cr, have 100cr"));
}

#[test]
fn jobs_can_be_listed_accepted_and_abandoned() {
    let output = play("jobs", "jobs\naccept 1\nmissions\nabandon 1\nabandon 9");

    assert!(output.contains("Jobs at Sol"));
    assert!(output.contains("Active Jobs"));
    assert!(output.contains("Error: You have no job 9."));
    if output.contains("Success: Accepted: ") {
        assert!(output.contains("Abandoned: "));
    } else {
        assert!(output.contains("Error: Locked: needs "));
    }
}

#[test]
fn saves_can_be_made_listed_and_refused() {
    let output = play("saves", "save slot_one\nsave bad!\nsaves\nload nowhere");

    assert!(output.contains("Success: Game saved successfully!"));
    assert!(output.contains("Error: Slot name must contain only letters, numbers, and underscores"));
    assert!(output.contains("Available Save Files"));
    assert!(output.contains("Error: Save file 'nowhere' not found"));
}

#[test]
fn unknown_commands_are_reported() {
    let output = play("unknown", "dance");

    assert!(output.contains("Unknown command. Type 'help' for available commands."));
}

#[test]
fn messages_carry_icons_with_colour_and_none_without() {
    let plain = play("plain_icons", "buy gold 1");
    assert!(plain.contains("Error: Unknown commodity: gold"));
    assert!(!plain.contains('❌'));

    let coloured = play_with("coloured_icons", "buy gold 1", &[("CLICOLOR_FORCE", "1")]);
    assert!(coloured.contains("❌"));
}

#[test]
fn screens_open_with_an_icon_header_when_colour_is_on() {
    let plain = play("plain_headers", "status\nmarket\njobs\nhelp");
    assert!(plain.contains("Commander Status\n================"));
    assert!(!plain.contains('🚀'));

    let coloured = play_with(
        "coloured_headers",
        "status\nmarket\njobs\ncargo\nhelp",
        &[("CLICOLOR_FORCE", "1")],
    );
    for icon in ["🚀", "📊", "💼", "📦", "❓", "💰", "⛽", "⚡"] {
        assert!(coloured.contains(icon), "missing {}", icon);
    }
}

#[test]
fn systems_and_info_show_economy_and_government() {
    let plain = play("plain_systems", "systems\ninfo proxima centauri");
    assert!(plain.contains("Proxima Centauri") && plain.contains("2.2 ly"));
    assert!(plain.contains("Economy: "));
    assert!(
        !plain.contains("Economy: Rich Ind"),
        "uses the display name"
    );
    assert!(!plain.contains('🏭') && !plain.contains('🌾'));

    let coloured = play_with(
        "coloured_systems",
        "systems\ninfo proxima centauri",
        &[("CLICOLOR_FORCE", "1")],
    );
    assert!(coloured.contains('🏭') || coloured.contains('🌾'));
    assert!(coloured.contains("Government: "));
}

#[test]
fn status_bars_use_blocks_with_colour_and_hashes_without() {
    let coloured = play_with("coloured_bars", "status", &[("CLICOLOR_FORCE", "1")]);
    assert!(coloured.contains("██████████"));
    assert!(coloured.contains("░░░░░░░░░░"));
}

#[test]
fn help_is_numbered_and_numbers_and_shortcuts_run_commands() {
    let output = play("menu", "help\n2\ns\n7\nm\n99");

    assert!(output.contains(" 1. help - Show this help menu"));
    assert!(output.contains(" 2. status - Display ship and commander status"));
    assert!(output.contains("25. quit - Exit the game"));
    assert_eq!(output.matches("Commander Status").count(), 2);
    assert_eq!(output.matches("Sol Market").count(), 2);
    assert!(output.contains("Unknown command."));
}

#[test]
fn lists_are_tables_with_a_header_rule() {
    let output = play("tables", "market\ncargo\nsystems\njobs");

    for header in ["Commodity", "Available", "Distance", "Deadline"] {
        assert!(output.contains(header), "missing {}", header);
    }
    assert!(output
        .lines()
        .any(|line| line.trim().chars().all(|c| c == '-') && line.len() > 20));
}

#[test]
fn route_plans_the_cheapest_fuel_path_and_marks_refuel_stops() {
    let output = play("route", "route gj 3512\nroute sol\nroute nowhere\nroute");

    assert!(output.contains("Route to GJ 3512"));
    assert!(output.contains("refuel at"));
    assert!(output.contains("jumps,"));
    assert!(output.contains("Error: You are already at Sol"));
    assert!(output.contains("Error: System 'nowhere' not found"));
    assert!(output.contains("Error: Usage: route <system_name>"));
}

#[test]
fn info_marks_the_frontier() {
    let manager = elite_core::data::systems::SystemManager::new().unwrap();
    let systems = manager.get_all_systems();
    let frontier = elite_core::game::frontier::frontier_systems(&systems);
    let far = frontier.iter().min().unwrap().to_lowercase();

    let output = play("frontier", &format!("info {}\ninfo sol", far));

    assert_eq!(
        output
            .matches("Frontier: yes, one of the outer stars")
            .count(),
        1
    );
}
