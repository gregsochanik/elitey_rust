use std::fs;
use std::path::Path;

const FORBIDDEN: [&str; 8] = [
    "println!",
    "print!",
    "eprintln!",
    "colored",
    "crate::utils",
    "crate::cli",
    "crossterm",
    "stdin",
];

fn rust_files(directory: &Path, found: &mut Vec<std::path::PathBuf>) {
    for entry in fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            rust_files(&path, found);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            found.push(path);
        }
    }
}

#[test]
fn the_game_and_data_layers_do_no_terminal_io() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    rust_files(&root.join("game"), &mut files);
    rust_files(&root.join("data"), &mut files);
    assert!(!files.is_empty());

    let mut offences = Vec::new();
    for file in files {
        let source = fs::read_to_string(&file).unwrap();
        for (number, line) in source.lines().enumerate() {
            let code = line.split("//").next().unwrap_or("");
            for word in FORBIDDEN {
                if code.contains(word) {
                    offences.push(format!("{}:{} uses {}", file.display(), number + 1, word));
                }
            }
        }
    }
    assert!(offences.is_empty(), "{}", offences.join("\n"));
}
