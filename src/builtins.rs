use crate::history::HistoryManager;
use crate::parser::Command;
use crate::path_utils::collapse_tilde;

pub enum BuiltinResult {
    HandledContinue,
    HandledExit,
    NotHandled,
}

pub fn handle_builtin(
    cmd: &Command,
    history_mgr: &HistoryManager,
    command_history: &mut Vec<String>,
) -> Result<BuiltinResult, String> {
    match cmd.name.as_str() {
        "exit" => {
            // Save history before exiting
            history_mgr.save(command_history)?;
            println!("Exiting.");
            Ok(BuiltinResult::HandledExit)
        }
        "history" => {
            for (idx, entry) in command_history.iter().enumerate() {
                println!("{}: {}", idx + 1, entry);
            }
            Ok(BuiltinResult::HandledContinue)
        }
        "cd" => {
            let target = if cmd.args.is_empty() {
                std::env::var("HOME").unwrap_or_else(|_| "/".to_string())
            } else {
                let p = collapse_tilde(&cmd.args[0]);
                p.to_string_lossy().to_string()
            };

            match std::env::set_current_dir(&target) {
                Ok(()) => {
                    // persist on success
                    history_mgr.add_entry(&format!("cd {}", target), command_history)?;
                    Ok(BuiltinResult::HandledContinue)
                }
                Err(e) => {
                    eprintln!("cd: {}: {}", target, e);
                    Ok(BuiltinResult::HandledContinue)
                }
            }
        }
        _ => Ok(BuiltinResult::NotHandled),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_history_builtin_prints() {
        // Prepare a fake history
        let mgr = HistoryManager::new().unwrap();
        let mut history: Vec<String> = vec!["a".into(), "b".into()];
        let cmd = Command { name: "history".into(), args: vec![] };

        // Should return HandledContinue and print (we don't capture stdout here)
        let res = handle_builtin(&cmd, &mgr, &mut history).unwrap();
        assert!(matches!(res, BuiltinResult::HandledContinue));
    }

    #[test]
    fn test_cd_builtin_changes_dir() {
        let mgr = HistoryManager::new().unwrap();
        let tmp = TempDir::new().unwrap();
        let tmp_path = tmp.path().to_string_lossy().to_string();

        let orig = std::env::current_dir().unwrap();

        let cmd = Command { name: "cd".into(), args: vec![tmp_path.clone()] };
        let mut history = Vec::new();
        let res = handle_builtin(&cmd, &mgr, &mut history).unwrap();
        assert!(matches!(res, BuiltinResult::HandledContinue));

        let cwd = std::env::current_dir().unwrap();
        assert_eq!(cwd, tmp.path());

        // restore
        std::env::set_current_dir(orig).unwrap();
    }
}
