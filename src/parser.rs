use conch_parser::ast;
use conch_parser::lexer::Lexer;
use conch_parser::parse::DefaultParser;
// Try to use the type alias from conch_parser if available, or define a compatible signature.
// Since we can't easily import DefaultPipeableCommand if it's not public,
// let's try to make process_top_level_command generic over T,
// and inside extract_from_pipeable, we cast/match T.

// Actually, extract_from_pipeable is called with a specific type.
// The type is implied by DefaultParser.

// Let's try to define process_top_level_command to take ANY TopLevelCommand<T>,
// but we need to call process_listable on it.
// process_listable expects ListableCommand<T>.
// And process_listable calls extract_from_pipeable.

// If we make process_top_level_command generic:
// fn process_top_level_command<T>(cmd: &TopLevelCommand<T>) -> Vec<Command>
// where T: PipeableCommandTrait?

// Let's try to use the `ast::PipeableCommand` type but with `String` for recursive params?
// No, that's wrong.

// Let's try to import DefaultPipeableCommand.
use conch_parser::ast::DefaultPipeableCommand;

#[derive(Debug, Clone)]
pub struct SimpleCommand {
    pub name: String,
    pub args: Vec<String>,
    pub assignments: Vec<(String, String)>,
}

#[derive(Debug, Clone)]
pub enum Command {
    Simple(SimpleCommand),
    Subshell(Vec<Vec<Command>>),
}

impl SimpleCommand {
    /// Parse a single command line (backward compatibility)
    #[allow(dead_code)]
    pub fn parse(input: &str) -> Option<Self> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return None;
        }

        // Try to parse as a pipeline first
        if let Some(pipeline) = Command::parse_pipeline(trimmed) {
            if pipeline.len() == 1 {
                if let Command::Simple(cmd) = &pipeline[0] {
                    return Some(cmd.clone());
                }
            }
        }

        // Fallback to simple split-based parsing for single commands
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }

        let name = parts[0].to_string();
        let args = parts[1..].iter().map(|s| s.to_string()).collect();
        let assignments = Vec::new();

        Some(SimpleCommand {
            name,
            args,
            assignments,
        })
    }

    /// Convert a conch SimpleCommand to our SimpleCommand struct
    fn simple_command_to_command<V, W, R>(
        simple_cmd: &ast::SimpleCommand<V, W, R>,
    ) -> Option<SimpleCommand>
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
                    } else if let Some(start_idx) = debug_str.rfind("Var(\"") {
                        // Handle variables
                        let start = start_idx + "Var(\"".len();
                        if let Some(end_idx) = debug_str[start..].find("\")") {
                            let var_name = &debug_str[start..start + end_idx];
                            cmd_words.push(format!("${}", var_name));
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

        let mut assignments = Vec::new();
        for item in &simple_cmd.redirects_or_env_vars {
            if let ast::RedirectOrEnvVar::EnvVar(name, value) = item {
                let val_str = if let Some(val) = value {
                    let debug_str = format!("{:?}", val);
                    if let Some(start_idx) = debug_str.rfind("Literal(\"") {
                        let start = start_idx + "Literal(\"".len();
                        if let Some(end_idx) = debug_str[start..].find("\")") {
                            debug_str[start..start + end_idx].to_string()
                        } else {
                            debug_str
                        }
                    } else {
                        debug_str
                    }
                } else {
                    String::new()
                };
                assignments.push((name.to_string(), val_str));
            }
        }

        if cmd_words.is_empty() && assignments.is_empty() {
            return None;
        }

        let name = if cmd_words.is_empty() {
            String::new()
        } else {
            cmd_words[0].clone()
        };
        let args = if cmd_words.len() > 1 {
            cmd_words[1..].to_vec()
        } else {
            Vec::new()
        };

        let sc = SimpleCommand {
            name,
            args,
            assignments,
        };
        // println!("Parsed SimpleCommand: {:?}", sc);
        Some(sc)
    }
}

impl Command {
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
                let commands = Self::process_top_level_command(&cmd_top_level);
                if commands.is_empty() {
                    None
                } else {
                    Some(commands)
                }
            }
            _ => None,
        }
    }

    /// Parse a script (multiple commands) into a list of pipelines
    pub fn parse_script(input: &str) -> Option<Vec<Vec<Command>>> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return None;
        }

        let lexer = Lexer::new(trimmed.chars());
        let mut parser = DefaultParser::new(lexer);
        let mut all_pipelines = Vec::new();

        loop {
            match parser.complete_command() {
                Ok(Some(cmd_top_level)) => {
                    let commands = Self::process_top_level_command(&cmd_top_level);
                    if !commands.is_empty() {
                        all_pipelines.push(commands);
                    }
                }
                Ok(None) => break,     // EOF
                Err(_) => return None, // Parse error
            }
        }

        if all_pipelines.is_empty() {
            None
        } else {
            Some(all_pipelines)
        }
    }

    fn process_top_level_command<T>(cmd_top_level: &ast::TopLevelCommand<T>) -> Vec<Command> {
        let mut commands = Vec::new();
        let command = &cmd_top_level.0;
        // We can't match on command if T is generic because we don't know the variants of Command<T>.
        // Command<T> enum is: List(List<T>), Job(Job<T>).
        // This is always true regardless of T.
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
        commands
    }

    fn process_listable<T>(listable: &ast::ListableCommand<T>, commands: &mut Vec<Command>) {
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
    fn extract_from_pipeable<T>(cmd: &T) -> Option<Command> {
        // Unsafe transmute to DefaultPipeableCommand.
        // We assume that whatever T is (likely String), it holds the data of DefaultPipeableCommand.
        let cmd_typed: &DefaultPipeableCommand = unsafe { std::mem::transmute(cmd) };

        match cmd_typed {
            ast::PipeableCommand::Simple(simple_cmd) => {
                SimpleCommand::simple_command_to_command(simple_cmd.as_ref()).map(Command::Simple)
            }
            ast::PipeableCommand::Compound(compound) => {
                match &compound.kind {
                    ast::CompoundCommandKind::Subshell(cmds) => {
                        let mut subshell_pipelines = Vec::new();
                        for top_cmd in cmds {
                            // Recursively process subshell commands
                            // top_cmd is TopLevelCommand<String> (if T=String).
                            // We can just call process_top_level_command directly if T=String.
                            // But here we don't know T.
                            // However, we know top_cmd is TopLevelCommand<String> (because DefaultPipeableCommand says so).
                            // So we can call process_top_level_command directly if T=String.
                            let pipeline = Self::process_top_level_command(top_cmd);
                            if !pipeline.is_empty() {
                                subshell_pipelines.push(pipeline);
                            }
                        }
                        Some(Command::Subshell(subshell_pipelines))
                    }
                    _ => None,
                }
            }
            _ => None, // Other compound commands not supported for now
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_command() {
        let cmd = SimpleCommand::parse("ls -la").unwrap();
        assert_eq!(cmd.name, "ls");
        assert_eq!(cmd.args, vec!["-la"]);
        assert!(cmd.assignments.is_empty());
    }

    #[test]
    fn test_parse_empty_string() {
        assert!(SimpleCommand::parse("").is_none());
    }

    #[test]
    fn test_parse_whitespace_only() {
        assert!(SimpleCommand::parse("   ").is_none());
    }

    #[test]
    fn test_parse_pipeline_single_command() {
        let pipeline = Command::parse_pipeline("echo hello").unwrap();
        assert_eq!(pipeline.len(), 1);
        if let Command::Simple(cmd) = &pipeline[0] {
            assert_eq!(cmd.name, "echo");
            assert_eq!(cmd.args, vec!["hello"]);
        } else {
            panic!("Expected Simple command");
        }
    }

    #[test]
    fn test_parse_pipeline_two_commands() {
        let pipeline = Command::parse_pipeline("echo hello | wc -c").unwrap();
        assert_eq!(pipeline.len(), 2);
        if let Command::Simple(cmd) = &pipeline[0] {
            assert_eq!(cmd.name, "echo");
            assert_eq!(cmd.args, vec!["hello"]);
        } else {
            panic!("Expected Simple command");
        }
        if let Command::Simple(cmd) = &pipeline[1] {
            assert_eq!(cmd.name, "wc");
            assert_eq!(cmd.args, vec!["-c"]);
        } else {
            panic!("Expected Simple command");
        }
    }

    #[test]
    fn test_parse_pipeline_three_commands() {
        let pipeline = Command::parse_pipeline("cat file.txt | grep pattern | wc -l").unwrap();
        assert_eq!(pipeline.len(), 3);
        if let Command::Simple(cmd) = &pipeline[0] {
            assert_eq!(cmd.name, "cat");
            assert_eq!(cmd.args, vec!["file.txt"]);
        }
        if let Command::Simple(cmd) = &pipeline[1] {
            assert_eq!(cmd.name, "grep");
            assert_eq!(cmd.args, vec!["pattern"]);
        }
        if let Command::Simple(cmd) = &pipeline[2] {
            assert_eq!(cmd.name, "wc");
            assert_eq!(cmd.args, vec!["-l"]);
        }
    }

    #[test]
    fn test_parse_pipeline_empty_string() {
        assert!(Command::parse_pipeline("").is_none());
    }

    #[test]
    fn test_parse_pipeline_whitespace_only() {
        assert!(Command::parse_pipeline("   ").is_none());
    }

    #[test]
    fn test_debug_subshell_ast() {
        let input = "(echo hello)";
        let lexer = Lexer::new(input.chars());
        let mut parser = DefaultParser::new(lexer);
        if let Ok(Some(cmd)) = parser.complete_command() {
            println!("Type: {}", std::any::type_name_of_val(&cmd));
            println!("AST: {:?}", cmd);
        }
    }

    #[test]
    fn test_pipeline_vs_sequence_parsing() {
        let pipeline = Command::parse_pipeline("echo a | echo b").unwrap();
        let sequence = Command::parse_pipeline("echo a; echo b").unwrap();

        println!("Pipeline len: {}", pipeline.len());
        println!("Sequence len: {}", sequence.len());

        // If they are identical, then pmsh cannot distinguish them
        assert_eq!(pipeline.len(), 2);
        assert_eq!(sequence.len(), 1);

        if let Command::Simple(p1) = &pipeline[0] {
            if let Command::Simple(s1) = &sequence[0] {
                assert_eq!(p1.name, s1.name);
            }
        }
    }
}
