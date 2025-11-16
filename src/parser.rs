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

    /// Parse a command line into a pipeline (sequence of commands separated by |).
    /// Returns a Vec of Commands. Single commands are valid pipelines of length 1.
    pub fn parse_pipeline(input: &str) -> Option<Vec<Command>> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return None;
        }

        let segments: Vec<&str> = trimmed.split('|').collect();
        let mut pipeline = Vec::new();

        for segment in segments {
            match Self::parse(segment) {
                Some(cmd) => pipeline.push(cmd),
                None => return None, // if any segment fails to parse, the whole pipeline fails
            }
        }

        if pipeline.is_empty() {
            None
        } else {
            Some(pipeline)
        }
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

    #[test]
    fn test_parse_pipeline_single_command() {
        let pipeline = Command::parse_pipeline("echo hello").unwrap();
        assert_eq!(pipeline.len(), 1);
        assert_eq!(pipeline[0].name, "echo");
        assert_eq!(pipeline[0].args, vec!["hello"]);
    }

    #[test]
    fn test_parse_pipeline_two_commands() {
        let pipeline = Command::parse_pipeline("echo hello | wc -c").unwrap();
        assert_eq!(pipeline.len(), 2);
        assert_eq!(pipeline[0].name, "echo");
        assert_eq!(pipeline[0].args, vec!["hello"]);
        assert_eq!(pipeline[1].name, "wc");
        assert_eq!(pipeline[1].args, vec!["-c"]);
    }

    #[test]
    fn test_parse_pipeline_three_commands() {
        let pipeline = Command::parse_pipeline("cat file.txt | grep pattern | wc -l").unwrap();
        assert_eq!(pipeline.len(), 3);
        assert_eq!(pipeline[0].name, "cat");
        assert_eq!(pipeline[0].args, vec!["file.txt"]);
        assert_eq!(pipeline[1].name, "grep");
        assert_eq!(pipeline[1].args, vec!["pattern"]);
        assert_eq!(pipeline[2].name, "wc");
        assert_eq!(pipeline[2].args, vec!["-l"]);
    }

    #[test]
    fn test_parse_pipeline_empty_string() {
        assert!(Command::parse_pipeline("").is_none());
    }

    #[test]
    fn test_parse_pipeline_whitespace_only() {
        assert!(Command::parse_pipeline("   ").is_none());
    }
}
