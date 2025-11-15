#[derive(Debug, Clone)]
pub struct Command {
    pub name: String,
    pub args: Vec<String>,
}

impl Command {
    pub fn parse(input: &str) -> Option<Self> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return None;
        }

        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }

        let name = parts[0].to_string();
        let args = parts[1..].iter().map(|s| s.to_string()).collect();

        Some(Command { name, args })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_command() {
        let cmd = Command::parse("ls -la").unwrap();
        assert_eq!(cmd.name, "ls");
        assert_eq!(cmd.args, vec!["-la"]);
    }

    #[test]
    fn test_parse_empty_string() {
        assert!(Command::parse("").is_none());
    }

    #[test]
    fn test_parse_whitespace_only() {
        assert!(Command::parse("   ").is_none());
    }
}
