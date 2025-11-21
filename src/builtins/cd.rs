use crate::builtins::common::SHELL_HELP_TEMPLATE;
use crate::history::HistoryManager;
use crate::parser::Command;
use crate::path_utils::collapse_tilde;
use clap::Parser;

use super::BuiltinResult;

/// Change the shell working directory
#[derive(Parser, Debug)]
#[command(name = "cd")]
#[command(about = "Change the shell working directory", long_about = None)]
#[command(help_template = SHELL_HELP_TEMPLATE)]
struct CdArgs {
    /// The target directory to change to. Use '-' for previous directory, '~' for HOME
    #[arg(value_name = "DIR")]
    dir: Option<String>,
}

pub fn execute(
    cmd: &Command,
    history_mgr: &HistoryManager,
    command_history: &mut Vec<String>,
    oldpwd: &mut Option<String>,
) -> Result<BuiltinResult, String> {
    // Parse arguments using clap
    let args_iter = std::iter::once("cd".to_string())
        .chain(cmd.args.iter().cloned())
        .collect::<Vec<_>>();

    let parsed_args = match CdArgs::try_parse_from(&args_iter) {
        Ok(args) => args,
        Err(e) => {
            // Clap handles --help and errors; just print and return
            print!("{}", e);
            return Ok(BuiltinResult::HandledContinue);
        }
    };

    let target = if let Some(ref dir) = parsed_args.dir {
        if dir == "-" {
            // cd - switches to OLDPWD
            match oldpwd.as_ref() {
                Some(prev) => prev.clone(),
                None => {
                    eprintln!("cd: OLDPWD not set");
                    return Ok(BuiltinResult::HandledContinue);
                }
            }
        } else {
            let p = collapse_tilde(dir);
            p.to_string_lossy().to_string()
        }
    } else {
        // No argument provided, use HOME
        std::env::var("HOME").unwrap_or_else(|_| "/".to_string())
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
            if parsed_args.dir.as_deref() == Some("-") {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::history::HistoryManager;
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
    fn test_cd_builtin_changes_dir() {
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
        let res = execute(&cmd, &mgr, &mut history, &mut oldpwd).unwrap();
        assert!(matches!(res, BuiltinResult::HandledContinue));

        assert!(history.iter().any(|h| h.starts_with("cd ")));

        let _ = std::env::set_current_dir(orig);
        drop(home_guard);
    }

    #[test]
    #[serial_test::serial]
    fn test_cd_dash_switches_to_previous_dir() {
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

        let cmd1 = Command {
            name: "cd".into(),
            args: vec![tmp1_path.clone()],
        };
        execute(&cmd1, &mgr, &mut history, &mut oldpwd).unwrap();
        assert!(oldpwd.is_some());

        let cmd2 = Command {
            name: "cd".into(),
            args: vec![tmp2_path.clone()],
        };
        execute(&cmd2, &mgr, &mut history, &mut oldpwd).unwrap();
        assert_eq!(oldpwd.as_ref().unwrap(), &tmp1_path);

        let cmd_dash = Command {
            name: "cd".into(),
            args: vec!["-".into()],
        };
        execute(&cmd_dash, &mgr, &mut history, &mut oldpwd).unwrap();
        let current = std::env::current_dir().unwrap();
        assert_eq!(current.to_string_lossy(), tmp1_path);
        assert_eq!(oldpwd.as_ref().unwrap(), &tmp2_path);

        let _ = std::env::set_current_dir(orig);
        drop(home_guard);
    }

    #[test]
    #[serial_test::serial]
    fn test_cd_dash_without_oldpwd() {
        let home_tmp = TempDir::new().unwrap();
        let home_guard = EnvVarGuard::new("HOME");
        home_guard.set(home_tmp.path().to_string_lossy().as_ref());

        let mgr = HistoryManager::new().unwrap();
        let mut history = Vec::new();
        let mut oldpwd = None;

        let orig = std::env::current_dir().unwrap();
        let cmd = Command {
            name: "cd".into(),
            args: vec!["-".into()],
        };
        let res = execute(&cmd, &mgr, &mut history, &mut oldpwd).unwrap();
        assert!(matches!(res, BuiltinResult::HandledContinue));
        let current = std::env::current_dir().unwrap();
        assert_eq!(current, orig);

        drop(home_guard);
    }

    #[test]
    fn test_cd_help() {
        let mgr = HistoryManager::new().unwrap_or_else(|_| HistoryManager::default());
        let mut history = Vec::new();
        let mut oldpwd = None;

        let cmd = Command {
            name: "cd".into(),
            args: vec!["--help".into()],
        };
        let res = execute(&cmd, &mgr, &mut history, &mut oldpwd).unwrap();
        assert!(matches!(res, BuiltinResult::HandledContinue));
    }
}
