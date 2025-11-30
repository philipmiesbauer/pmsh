use conch_parser::ast::{
    self, Command as ConchCommand, CompoundCommand, CompoundCommandKind, DefaultPipeableCommand,
    ListableCommand, PipeableCommand, Redirect, TopLevelCommand, TopLevelWord,
};
use conch_parser::lexer::Lexer;
use conch_parser::parse::DefaultParser;

#[derive(Debug, Clone, PartialEq)]
pub struct SimpleCommand {
    pub name: String,
    pub args: Vec<String>,
    pub assignments: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Simple(SimpleCommand),
    Subshell(Vec<Vec<Command>>),
    FunctionDef(String, Vec<Vec<Command>>),
}

impl SimpleCommand {
    fn word_to_string(word: &TopLevelWord<String>) -> String {
        // This is a simplified extraction that works for literals.
        // For complex words (concatenation, substitution), it might need more work.
        // We can try to format it using Debug and clean up, or traverse.
        // Let's traverse for common cases.

        match &word.0 {
            ast::ComplexWord::Single(w) => Self::inner_word_to_string(w),
            ast::ComplexWord::Concat(ws) => ws.iter().map(Self::inner_word_to_string).collect(),
        }
    }

    fn inner_word_to_string(word: &ast::DefaultWord) -> String {
        match word {
            ast::Word::Simple(s) => Self::simple_word_to_string(s),
            ast::Word::DoubleQuoted(ws) => ws.iter().map(Self::simple_word_to_string).collect(),
            ast::Word::SingleQuoted(s) => s.clone(),
        }
    }

    fn simple_word_to_string(word: &ast::DefaultSimpleWord) -> String {
        match word {
            ast::SimpleWord::Literal(s) => s.clone(),
            ast::SimpleWord::Escaped(s) => s.clone(),
            ast::SimpleWord::Param(p) => {
                // Handle parameters like $var
                match p {
                    ast::Parameter::Var(v) => format!("${}", v),
                    ast::Parameter::Positional(p) => format!("${}", p),
                    ast::Parameter::At => "$@".to_string(),
                    ast::Parameter::Star => "$*".to_string(),
                    ast::Parameter::Pound => "$#".to_string(),
                    ast::Parameter::Question => "$?".to_string(),
                    ast::Parameter::Dash => "$-".to_string(),
                    ast::Parameter::Dollar => "$$".to_string(),
                    ast::Parameter::Bang => "$!".to_string(),
                }
            }
            ast::SimpleWord::Subst(_) => String::new(), // TODO: Handle substitutions (command substitution, arithmetic, etc.)
            ast::SimpleWord::Star => "*".to_string(),
            ast::SimpleWord::Question => "?".to_string(),
            ast::SimpleWord::SquareOpen => "[".to_string(),
            ast::SimpleWord::SquareClose => "]".to_string(),
            ast::SimpleWord::Tilde => "~".to_string(),
            ast::SimpleWord::Colon => ":".to_string(),
        }
    }

    fn simple_command_to_command(
        simple: &ast::SimpleCommand<String, TopLevelWord<String>, Redirect<TopLevelWord<String>>>,
    ) -> Option<SimpleCommand> {
        let mut args = Vec::new();
        let mut assignments = Vec::new();

        // Process redirects_or_env_vars for assignments
        for redirect_or_env in &simple.redirects_or_env_vars {
            if let ast::RedirectOrEnvVar::EnvVar(name, value) = redirect_or_env {
                let val = value.as_ref().map(Self::word_to_string).unwrap_or_default();
                assignments.push((name.to_string(), val));
            }
        }

        // Process redirects_or_cmd_words for command name and args
        for redirect_or_word in &simple.redirects_or_cmd_words {
            if let ast::RedirectOrCmdWord::CmdWord(word) = redirect_or_word {
                args.push(Self::word_to_string(word));
            }
        }

        if args.is_empty() && assignments.is_empty() {
            return None;
        }

        let name = if args.is_empty() {
            String::new()
        } else {
            args.remove(0)
        };

        Some(SimpleCommand {
            name,
            args,
            assignments,
        })
    }
}

impl Command {
    pub fn parse(input: &str) -> Result<Vec<Vec<Command>>, String> {
        let lexer = Lexer::new(input.chars());
        let mut parser = DefaultParser::new(lexer);
        let mut pipelines = Vec::new();

        loop {
            match parser.complete_command() {
                Ok(Some(cmd)) => {
                    let pipeline = Self::process_top_level_command(&cmd);
                    if !pipeline.is_empty() {
                        pipelines.push(pipeline);
                    }
                }
                Ok(None) => break,
                Err(e) => return Err(format!("Parse error: {}", e)),
            }
        }

        Ok(pipelines)
    }

    pub fn parse_pipeline(input: &str) -> Option<Vec<Command>> {
        match Self::parse(input) {
            Ok(mut pipelines) => {
                if pipelines.is_empty() {
                    None
                } else {
                    Some(pipelines.remove(0))
                }
            }
            Err(_) => None,
        }
    }

    pub fn parse_script(input: &str) -> Result<Vec<Vec<Command>>, String> {
        Self::parse(input)
    }

    fn process_top_level_command(cmd_top_level: &TopLevelCommand<String>) -> Vec<Command> {
        let mut commands = Vec::new();
        // cmd_top_level.0 is Command<CommandList<String, TopLevelWord<String>, TopLevelCommand<String>>>
        // CommandList is AndOrList<ListableCommand<DefaultPipeableCommand>>

        let command = &cmd_top_level.0;
        match command {
            ConchCommand::List(list) => {
                Self::process_listable(&list.first, &mut commands);
                for and_or in &list.rest {
                    match and_or {
                        ast::AndOr::And(cmd) => Self::process_listable(cmd, &mut commands),
                        ast::AndOr::Or(cmd) => Self::process_listable(cmd, &mut commands),
                    }
                }
            }
            ConchCommand::Job(job) => {
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

    fn process_listable(
        listable: &ListableCommand<DefaultPipeableCommand>,
        commands: &mut Vec<Command>,
    ) {
        match listable {
            ListableCommand::Pipe(_, cmds) => {
                for cmd in cmds {
                    if let Some(c) = Self::extract_from_pipeable(cmd) {
                        commands.push(c);
                    }
                }
            }
            ListableCommand::Single(cmd) => {
                if let Some(c) = Self::extract_from_pipeable(cmd) {
                    commands.push(c);
                }
            }
        }
    }

    #[allow(clippy::type_complexity)]
    fn process_compound_command(
        compound: &CompoundCommand<
            CompoundCommandKind<String, TopLevelWord<String>, TopLevelCommand<String>>,
            Redirect<TopLevelWord<String>>,
        >,
    ) -> Option<Vec<Vec<Command>>> {
        match &compound.kind {
            CompoundCommandKind::Subshell(cmds) | CompoundCommandKind::Brace(cmds) => {
                let mut pipelines = Vec::new();
                for top_cmd in cmds {
                    let pipeline = Self::process_top_level_command(top_cmd);
                    if !pipeline.is_empty() {
                        pipelines.push(pipeline);
                    }
                }
                Some(pipelines)
            }
            _ => None,
        }
    }

    fn extract_from_pipeable(cmd: &DefaultPipeableCommand) -> Option<Command> {
        match cmd {
            PipeableCommand::Simple(simple_cmd) => {
                SimpleCommand::simple_command_to_command(simple_cmd.as_ref()).map(Command::Simple)
            }
            PipeableCommand::Compound(compound) => match &compound.kind {
                CompoundCommandKind::Subshell(_) => {
                    Self::process_compound_command(compound.as_ref()).map(Command::Subshell)
                }
                _ => None,
            },
            PipeableCommand::FunctionDef(name, body) => {
                Self::process_compound_command(body.as_ref())
                    .map(|cmds| Command::FunctionDef(name.clone(), cmds))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let input = "echo hello";
        let result = Command::parse(input).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].len(), 1);
        if let Command::Simple(cmd) = &result[0][0] {
            assert_eq!(cmd.name, "echo");
            assert_eq!(cmd.args, vec!["hello"]);
        } else {
            panic!("Expected Simple command");
        }
    }

    #[test]
    fn test_parse_pipeline() {
        let input = "echo hello | wc -w";
        let result = Command::parse(input).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].len(), 2);
    }

    #[test]
    fn test_parse_subshell() {
        let input = "(echo hello)";
        let result = Command::parse(input).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].len(), 1);
        if let Command::Subshell(cmds) = &result[0][0] {
            assert_eq!(cmds.len(), 1);
            if let Command::Simple(cmd) = &cmds[0][0] {
                assert_eq!(cmd.name, "echo");
            }
        } else {
            panic!("Expected Subshell command");
        }
    }

    #[test]
    fn test_parse_function() {
        let input = "foo() { echo bar; }";
        let result = Command::parse(input).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].len(), 1);
        if let Command::FunctionDef(name, body) = &result[0][0] {
            assert_eq!(name, "foo");
            assert_eq!(body.len(), 1);
            if let Command::Simple(cmd) = &body[0][0] {
                assert_eq!(cmd.name, "echo");
                assert_eq!(cmd.args, vec!["bar"]);
            }
        } else {
            panic!("Expected FunctionDef command");
        }
    }
}
