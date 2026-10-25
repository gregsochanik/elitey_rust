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
    let home =
        std::env::temp_dir().join(format!("elitey_rust_cli_{}_{}", label, std::process::id()));
    let _ = std::fs::remove_dir_all(&home);
    std::fs::create_dir_all(&home).unwrap();

    let mut child = Command::new(env!("CARGO_BIN_EXE_elitey_rust"))
        .arg("--new")
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
    assert!(output.contains("Paid: "));
}

#[test]
fn the_market_shows_what_held_cargo_cost() {
    let output = play("hold", "buy food 2\nmarket");

    assert!(output.contains("Hold: 2t @ "));
}

#[test]
fn the_market_and_status_are_laid_out_as_before() {
    let output = play("layout", "market\nstatus");

    assert!(output.contains("LAVE Market"));
    assert!(output.contains("Food:"));
    assert!(output.contains("Buy: "));
    assert!(output.contains("vs base"));
    assert!(output.contains("Commander Status"));
    assert!(output.contains("Fuel: 7/7 light years"));
    assert!(output.contains("Hull: 100/100"));
}

#[test]
fn a_jump_asks_for_confirmation_and_can_be_cancelled() {
    let output = play("jump", "jump diso\nn\njump nowhere\njump");

    assert!(output.contains("Travel Confirmation"));
    assert!(output.contains("To: DISO"));
    assert!(output.contains("Info: Travel cancelled."));
    assert!(output.contains("Error: System 'nowhere' not found"));
    assert!(output.contains("Error: Usage: jump <system_name>"));
}

#[test]
fn upkeep_and_equipment_commands_report_what_they_did() {
    let output = play(
        "upkeep",
        "repair\nrefuel\nrefuel x\nequipment buy nothing\nupgrade python",
    );

    assert!(output.contains("Info: Your hull is already at full strength."));
    assert!(output.contains("Info: Your tank is already full."));
    assert!(output.contains("Error: Usage: refuel [units]"));
    assert!(output.contains("Error: Unknown equipment: nothing"));
    assert!(output.contains("Error: Insufficient credits. Need 2000cr, have 100cr"));
}

#[test]
fn jobs_can_be_listed_accepted_and_abandoned() {
    let output = play("jobs", "jobs\naccept 1\nmissions\nabandon 1\nabandon 9");

    assert!(output.contains("Jobs at LAVE"));
    assert!(output.contains("Success: Accepted: "));
    assert!(output.contains("Active Jobs"));
    assert!(output.contains("Abandoned: "));
    assert!(output.contains("Error: You have no job 9."));
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
