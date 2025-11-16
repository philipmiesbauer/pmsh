use crate::history::HistoryManager;
use crate::parser::Command;

use super::BuiltinResult;

#[allow(clippy::ptr_arg)]
pub fn execute(
    cmd: &Command,
    _history_mgr: &HistoryManager,
    command_history: &mut Vec<String>,
) -> Result<BuiltinResult, String> {
    // Check for --help flag
    if cmd.args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!("history: display command history");
        println!();
        println!("Usage: history");
        println!();
        println!("Display the command history list with line numbers.");
        println!("History is saved to ~/.pmsh_history (max 1000 entries).");
        return Ok(BuiltinResult::HandledContinue);
    }

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
        let cmd = Command {
            name: "history".into(),
            args: vec![],
        };

        let res = execute(&cmd, &mgr, &mut history).unwrap();
        assert!(matches!(res, BuiltinResult::HandledContinue));
    }

    #[test]
    fn test_history_help() {
        let mgr = HistoryManager::new().unwrap_or_else(|_| HistoryManager::default());
        let mut history = Vec::new();

        let cmd = Command {
            name: "history".into(),
            args: vec!["-h".into()],
        };
        let res = execute(&cmd, &mgr, &mut history).unwrap();
        assert!(matches!(res, BuiltinResult::HandledContinue));
    }
}
