use conch_parser::lexer::Lexer;
use conch_parser::parse::DefaultParser;
use conch_parser::ast;

#[derive(Debug, Clone)]
pub struct Command {
    pub name: String,
    pub args: Vec<String>,
}

impl Command {
    /// Parse a single command line (backward compatibility)
    #[allow(dead_code)]
    pub fn parse(input: &str) -> Option<Self> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return None;
        }

        // Try to parse as a pipeline first
        if let Some(pipeline) = Self::parse_pipeline(trimmed) {
            if pipeline.len() == 1 {
                return Some(pipeline.into_iter().next().unwrap());
            }
        }

        // Fallback to simple split-based parsing for single commands
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }

        let name = parts[0].to_string();
        let args = parts[1..].iter().map(|s| s.to_string()).collect();

        Some(Command { name, args })
    }

    /// Convert a conch SimpleCommand to our Command struct
    fn simple_command_to_command<V, W, R>(simple_cmd: &ast::SimpleCommand<V, W, R>) -> Option<Command>
    where
        V: ToString,
        W: std::fmt::Debug,
        R: std::fmt::Debug,
    {
        // SimpleCommand has redirects_or_cmd_words which is a Vec of either redirects or command words
        // We need to extract the command words (arguments)
        let mut cmd_words = Vec::new();

        for item in &simple_cmd.redirects_or_cmd_words {
            match item {
                ast::RedirectOrCmdWord::CmdWord(word) => {
                    // Use Debug format and extract the actual string value
                    let debug_str = format!("{:?}", word);
                    // Try to extract string from patterns like: TopLevelWord(Single(Simple(Literal("value"))))
                    // Look for the last occurrence of Literal(" and extract until the closing "
                    if let Some(start_idx) = debug_str.rfind("Literal(\"") {
                        let start = start_idx + "Literal(\"".len();
                        if let Some(end_idx) = debug_str[start..].find("\")") {
                            let value = &debug_str[start..start + end_idx];
                            cmd_words.push(value.to_string());
                        } else {
                            // Fallback to full debug string
                            cmd_words.push(debug_str);
                        }
                    } else if let Some(start_idx) = debug_str.rfind("Escaped(\"") {
                        // Handle escaped strings
                        let start = start_idx + "Escaped(\"".len();
                        if let Some(end_idx) = debug_str[start..].find("\")") {
                            let value = &debug_str[start..start + end_idx];
                            cmd_words.push(value.to_string());
                        } else {
                            cmd_words.push(debug_str);
                        }
                    } else {
                        // Fallback to full debug string if we can't parse
                        cmd_words.push(debug_str);
                    }
                }
                _ => {
                    // Ignore redirects for now
                }
            }
        }

        if cmd_words.is_empty() {
            return None;
        }

        let name = cmd_words[0].clone();
        let args = if cmd_words.len() > 1 {
            cmd_words[1..].to_vec()
        } else {
            Vec::new()
        };

        Some(Command { name, args })
    }

    /// Parse a command line into a pipeline (sequence of Commands)
    pub fn parse_pipeline(input: &str) -> Option<Vec<Command>> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return None;
        }

        let lexer = Lexer::new(trimmed.chars());
        let mut parser = DefaultParser::new(lexer);

        // Parse a complete command from the input
        match parser.complete_command() {
            Ok(Some(cmd_top_level)) => {
                let mut commands = Vec::new();

                // The top level command contains a Command directly
                // For simplicity, we'll just process it
                let command = &cmd_top_level.0;
                match command {
                    ast::Command::List(list) => {
                        Self::process_listable(&list.first, &mut commands);
                        for and_or in &list.rest {
                            match and_or {
                                ast::AndOr::And(cmd) => Self::process_listable(cmd, &mut commands),
                                ast::AndOr::Or(cmd) => Self::process_listable(cmd, &mut commands),
                            }
                        }
                    }
                    ast::Command::Job(job) => {
                        Self::process_listable(&job.first, &mut commands);
                        for and_or in &job.rest {
                            match and_or {
                                ast::AndOr::And(cmd) => Self::process_listable(cmd, &mut commands),
                                ast::AndOr::Or(cmd) => Self::process_listable(cmd, &mut commands),
                            }
                        }
                    }
                }

                if commands.is_empty() {
                    None
                } else {
                    Some(commands)
                }
            }
            _ => None,
        }
    }

    fn process_listable<C, F>(
        listable: &ast::ListableCommand<ast::PipeableCommand<String, Box<ast::SimpleCommand<String, ast::TopLevelWord<String>, ast::Redirect<ast::TopLevelWord<String>>>>, C, F>>,
        commands: &mut Vec<Command>
    ) {
         match listable {
            ast::ListableCommand::Pipe(_, cmds) => {
                for cmd in cmds {
                     if let Some(c) = Self::extract_from_pipeable(cmd) {
                         commands.push(c);
                     }
                }
            }
            ast::ListableCommand::Single(cmd) => {
                if let Some(c) = Self::extract_from_pipeable(cmd) {
                    commands.push(c);
                }
            }
         }
    }

    /// Extract a single command from a pipeablecommand enum variant
    fn extract_from_pipeable<PipeRedirect, F>(
        cmd: &ast::PipeableCommand<String, Box<ast::SimpleCommand<String, ast::TopLevelWord<String>, ast::Redirect<ast::TopLevelWord<String>>>>, PipeRedirect, F>,
    ) -> Option<Command> {
        match cmd {
            ast::PipeableCommand::Simple(simple_cmd) => Self::simple_command_to_command(simple_cmd.as_ref()),
            _ => None, // Compound commands not supported for now
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
