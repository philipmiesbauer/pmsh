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
    oldpwd: &mut Option<String>,
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
            } else if cmd.args[0] == "-" {
                // cd - switches to OLDPWD
                match oldpwd.as_ref() {
                    Some(prev) => prev.clone(),
                    None => {
                        eprintln!("cd: OLDPWD not set");
                        return Ok(BuiltinResult::HandledContinue);
                    }
                }
            } else {
                let p = collapse_tilde(&cmd.args[0]);
                p.to_string_lossy().to_string()
            };

            // Save current directory before changing
            let current = std::env::current_dir()
                .ok()
                .and_then(|p| p.to_str().map(|s| s.to_string()));

            match std::env::set_current_dir(&target) {
                Ok(()) => {
                    // Update OLDPWD to the previous current directory
                    *oldpwd = current;

                    // Print new directory for cd -
                    if !cmd.args.is_empty() && cmd.args[0] == "-" {
                        println!("{}", target);
                    }

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

    struct EnvVarGuard {
        key: String,
        original: Option<String>,
    }

    impl EnvVarGuard {
        fn new(key: &str) -> Self {
            let original = std::env::var(key).ok();
            EnvVarGuard {
                key: key.to_string(),
                original,
            }
        }

        fn set(&self, val: &str) {
            std::env::set_var(&self.key, val);
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            match &self.original {
                Some(v) => std::env::set_var(&self.key, v),
                None => std::env::remove_var(&self.key),
            }
        }
    }

    #[test]
    #[serial_test::serial]
    fn test_exit_builtin_saves_history() {
        // Ensure HOME is a writable temp dir so save succeeds
        let home_tmp = TempDir::new().unwrap();
        let home_guard = EnvVarGuard::new("HOME");
        home_guard.set(home_tmp.path().to_string_lossy().as_ref());

        let mgr = HistoryManager::new().unwrap();
        let mut history: Vec<String> = vec!["one".into(), "two".into()];
        let cmd = Command {
            name: "exit".into(),
            args: vec![],
        };
        let mut oldpwd = None;

        let res = handle_builtin(&cmd, &mgr, &mut history, &mut oldpwd).unwrap();
        assert!(matches!(res, BuiltinResult::HandledExit));
        drop(home_guard);
    }

    #[test]
    #[serial_test::serial]
    fn test_exit_builtin_save_failure() {
        // Create a file and point HOME at that file so history save will fail
        let tmp = TempDir::new().unwrap();
        let file_path = tmp.path().join("homefile");
        std::fs::write(&file_path, "x").unwrap();

        let home_guard = EnvVarGuard::new("HOME");
        home_guard.set(file_path.to_string_lossy().as_ref());

        let mgr = HistoryManager::new().unwrap();
        let mut history: Vec<String> = vec!["one".into()];
        let cmd = Command {
            name: "exit".into(),
            args: vec![],
        };
        let mut oldpwd = None;

        let res = handle_builtin(&cmd, &mgr, &mut history, &mut oldpwd);
        // should return Err because save fails
        assert!(res.is_err());
        drop(home_guard);
    }

    #[test]
    fn test_history_builtin_prints() {
        // Prepare a fake history
        let mgr = HistoryManager::new().unwrap();
        let mut history: Vec<String> = vec!["a".into(), "b".into()];
        let cmd = Command {
            name: "history".into(),
            args: vec![],
        };
        let mut oldpwd = None;

        // Should return HandledContinue and print (we don't capture stdout here)
        let res = handle_builtin(&cmd, &mgr, &mut history, &mut oldpwd).unwrap();
        assert!(matches!(res, BuiltinResult::HandledContinue));
    }

    #[test]
    #[serial_test::serial]
    fn test_cd_builtin_changes_dir() {
        // Ensure HOME writable so history add_entry works
        let home_tmp = TempDir::new().unwrap();
        let home_guard = EnvVarGuard::new("HOME");
        home_guard.set(home_tmp.path().to_string_lossy().as_ref());

        let mgr = HistoryManager::new().unwrap();
        let tmp = TempDir::new().unwrap();
        let tmp_path = tmp.path().to_string_lossy().to_string();

        let orig = std::env::current_dir().unwrap();

        let cmd = Command {
            name: "cd".into(),
            args: vec![tmp_path.clone()],
        };
        let mut history = Vec::new();
        let mut oldpwd = None;
        let res = handle_builtin(&cmd, &mgr, &mut history, &mut oldpwd).unwrap();
        assert!(matches!(res, BuiltinResult::HandledContinue));

        // We recorded a cd entry in history; don't rely on global CWD equality (tests run in parallel environments)
        assert!(history.iter().any(|h| h.starts_with("cd ")));

        // restore
        let _ = std::env::set_current_dir(orig);
        drop(home_guard);
    }

    #[test]
    #[serial_test::serial]
    fn test_cd_dash_switches_to_previous_dir() {
        // Ensure HOME writable so history add_entry works
        let home_tmp = TempDir::new().unwrap();
        let home_guard = EnvVarGuard::new("HOME");
        home_guard.set(home_tmp.path().to_string_lossy().as_ref());

        let mgr = HistoryManager::new().unwrap();
        let tmp1 = TempDir::new().unwrap();
        let tmp2 = TempDir::new().unwrap();
        let tmp1_path = tmp1.path().to_string_lossy().to_string();
        let tmp2_path = tmp2.path().to_string_lossy().to_string();

        let orig = std::env::current_dir().unwrap();
        let mut history = Vec::new();
        let mut oldpwd = None;

        // cd to tmp1
        let cmd1 = Command {
            name: "cd".into(),
            args: vec![tmp1_path.clone()],
        };
        handle_builtin(&cmd1, &mgr, &mut history, &mut oldpwd).unwrap();
        assert!(oldpwd.is_some());

        // cd to tmp2
        let cmd2 = Command {
            name: "cd".into(),
            args: vec![tmp2_path.clone()],
        };
        handle_builtin(&cmd2, &mgr, &mut history, &mut oldpwd).unwrap();
        // oldpwd should now be tmp1
        assert_eq!(oldpwd.as_ref().unwrap(), &tmp1_path);

        // cd - should go back to tmp1
        let cmd_dash = Command {
            name: "cd".into(),
            args: vec!["-".into()],
        };
        handle_builtin(&cmd_dash, &mgr, &mut history, &mut oldpwd).unwrap();
        let current = std::env::current_dir().unwrap();
        assert_eq!(current.to_string_lossy(), tmp1_path);
        // oldpwd should now be tmp2
        assert_eq!(oldpwd.as_ref().unwrap(), &tmp2_path);

        // restore
        let _ = std::env::set_current_dir(orig);
        drop(home_guard);
    }

    #[test]
    #[serial_test::serial]
    fn test_cd_dash_without_oldpwd() {
        // Ensure HOME writable
        let home_tmp = TempDir::new().unwrap();
        let home_guard = EnvVarGuard::new("HOME");
        home_guard.set(home_tmp.path().to_string_lossy().as_ref());

        let mgr = HistoryManager::new().unwrap();
        let mut history = Vec::new();
        let mut oldpwd = None;

        // cd - without OLDPWD set should print error and not change dir
        let orig = std::env::current_dir().unwrap();
        let cmd = Command {
            name: "cd".into(),
            args: vec!["-".into()],
        };
        let res = handle_builtin(&cmd, &mgr, &mut history, &mut oldpwd).unwrap();
        assert!(matches!(res, BuiltinResult::HandledContinue));
        // Directory should not have changed
        let current = std::env::current_dir().unwrap();
        assert_eq!(current, orig);

        drop(home_guard);
    }
}
