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
