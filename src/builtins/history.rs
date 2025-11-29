use crate::builtins::common::SHELL_HELP_TEMPLATE;
use crate::history::HistoryManager;
use crate::parser::SimpleCommand;
use clap::Parser;

use super::BuiltinResult;

/// Display command history
#[derive(Parser, Debug)]
#[command(name = "history")]
#[command(about = "Display the command history list with line numbers.", long_about = None)]
#[command(help_template = SHELL_HELP_TEMPLATE)]
#[command(after_help = "History is saved to ~/.pmsh_history (max 1000 entries).")]
struct HistoryArgs {}

#[allow(clippy::ptr_arg)]
pub fn execute(
    cmd: &SimpleCommand,
    _history_mgr: &HistoryManager,
    command_history: &mut Vec<String>,
) -> Result<BuiltinResult, String> {
    // Parse arguments using clap
    let args_iter = std::iter::once("history".to_string())
        .chain(cmd.args.iter().cloned())
        .collect::<Vec<_>>();

    let _parsed_args = match HistoryArgs::try_parse_from(&args_iter) {
        Ok(args) => args,
        Err(e) => {
            // Clap handles --help and errors; just print and return
            print!("{}", e);
            return Ok(BuiltinResult::HandledContinue);
        }
    };

    for (idx, entry) in command_history.iter().enumerate() {
        println!("{}: {}", idx + 1, entry);
    }
    Ok(BuiltinResult::HandledContinue)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::history::HistoryManager;

    #[test]
    fn test_history_builtin_prints() {
        let mgr = HistoryManager::new().unwrap();
        let mut history: Vec<String> = vec!["a".into(), "b".into()];
        let cmd = SimpleCommand {
            name: "history".to_string(),
            args: vec![],
            assignments: vec![],
        };

        let res = execute(&cmd, &mgr, &mut history).unwrap();
        assert!(matches!(res, BuiltinResult::HandledContinue));
    }

    #[test]
    fn test_history_help() {
        let mgr = HistoryManager::new().unwrap_or_else(|_| HistoryManager::default());
        let mut history = Vec::new();

        let cmd = SimpleCommand {
            name: "history".into(),
            args: vec!["-h".into()],
            assignments: vec![],
        };
        let res = execute(&cmd, &mgr, &mut history).unwrap();
        assert!(matches!(res, BuiltinResult::HandledContinue));
    }
}
