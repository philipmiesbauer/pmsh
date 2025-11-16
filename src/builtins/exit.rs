use crate::builtins::common::SHELL_HELP_TEMPLATE;
use crate::history::HistoryManager;
use crate::parser::Command;
use clap::Parser;

use super::BuiltinResult;

/// Exit the shell
#[derive(Parser, Debug)]
#[command(name = "exit")]
#[command(about = "Exit the shell", long_about = None)]
#[command(help_template = SHELL_HELP_TEMPLATE)]
struct ExitArgs {
    /// Return value to exit with
    #[arg(value_name = "n")]
    return_value: Option<i32>,
}

#[allow(clippy::ptr_arg)]
pub fn execute(
    cmd: &Command,
    history_mgr: &HistoryManager,
    command_history: &mut Vec<String>,
) -> Result<BuiltinResult, String> {
    // Parse arguments using clap
    let args_iter = std::iter::once("exit".to_string())
        .chain(cmd.args.iter().cloned())
        .collect::<Vec<_>>();

    let parsed_args = match ExitArgs::try_parse_from(&args_iter) {
        Ok(args) => args,
        Err(e) => {
            // Clap handles --help and errors; just print and return
            print!("{}", e);
            return Ok(BuiltinResult::HandledContinue);
        }
    };

    let return_code: i32 = parsed_args.return_value.unwrap_or(0);

    // Save history before exiting
    history_mgr.save(command_history)?;
    println!("Exiting.");
    Ok(BuiltinResult::HandledExit(return_code))
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
    fn test_exit_builtin_saves_history() {
        let home_tmp = TempDir::new().unwrap();
        let home_guard = EnvVarGuard::new("HOME");
        home_guard.set(home_tmp.path().to_string_lossy().as_ref());

        let mgr = HistoryManager::new().unwrap();
        let mut history: Vec<String> = vec!["one".into(), "two".into()];
        let cmd = Command {
            name: "exit".into(),
            args: vec![],
        };

        let res = execute(&cmd, &mgr, &mut history).unwrap();
        assert!(matches!(res, BuiltinResult::HandledExit(0)));
        drop(home_guard);
    }

    #[test]
    #[serial_test::serial]
    fn test_exit_builtin_save_failure() {
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

        let res = execute(&cmd, &mgr, &mut history);
        assert!(res.is_err());
        drop(home_guard);
    }

    #[test]
    #[serial_test::serial]
    fn test_exit_help() {
        let home_tmp = TempDir::new().unwrap();
        let home_guard = EnvVarGuard::new("HOME");
        home_guard.set(home_tmp.path().to_string_lossy().as_ref());

        let mgr = HistoryManager::new().unwrap();
        let mut history = Vec::new();

        let cmd = Command {
            name: "exit".into(),
            args: vec!["--help".into()],
        };
        let res = execute(&cmd, &mgr, &mut history).unwrap();
        assert!(matches!(res, BuiltinResult::HandledContinue));

        drop(home_guard);
    }
}
