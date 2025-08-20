use std::io::{self, Write};

/// Get user input from stdin
pub fn get_input() -> Result<String, io::Error> {
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input)
}

/// Get user input with a prompt
#[allow(dead_code)]
pub fn get_input_with_prompt(prompt: &str) -> Result<String, io::Error> {
    print!("{}", prompt);
    io::stdout().flush()?;
    get_input()
}

/// Parse command into parts
#[allow(dead_code)]
pub fn parse_command(input: &str) -> Vec<&str> {
    input.split_whitespace().collect()
}

/// Validate command format
#[allow(dead_code)]
pub fn validate_command(parts: &[&str]) -> bool {
    !parts.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_command() {
        let input = "help";
        let parts = parse_command(input);
        assert_eq!(parts, vec!["help"]);
    }

    #[test]
    fn test_parse_command_with_spaces() {
        let input = "  status  ";
        let parts = parse_command(input);
        assert_eq!(parts, vec!["status"]);
    }

    #[test]
    fn test_parse_command_multiple_parts() {
        let input = "buy food 10";
        let parts = parse_command(input);
        assert_eq!(parts, vec!["buy", "food", "10"]);
    }

    #[test]
    fn test_parse_empty_command() {
        let input = "";
        let parts = parse_command(input);
        assert_eq!(parts, Vec::<&str>::new());
    }

    #[test]
    fn test_validate_command() {
        let parts = vec!["help"];
        assert!(validate_command(&parts));
    }

    #[test]
    fn test_validate_empty_command() {
        let parts = Vec::<&str>::new();
        assert!(!validate_command(&parts));
    }
}
