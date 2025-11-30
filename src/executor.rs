use crate::builtins::{handle_builtin, BuiltinResult};
use crate::parser::{Command, SimpleCommand};
use crate::variables::Variables;
use std::process::{Command as StdCommand, Stdio};

pub struct Executor;

impl Executor {
    pub fn execute(
        cmd: &Command,
        vars: &mut Variables,
        history_mgr: &crate::history::HistoryManager,
        command_history: &mut Vec<String>,
        oldpwd: &mut Option<String>,
    ) -> Result<(), String> {
        match cmd {
            Command::Simple(simple_cmd) => {
                Self::execute_simple(simple_cmd, vars, history_mgr, command_history, oldpwd)
            }
            Command::Subshell(cmds) => {
                Self::execute_subshell(cmds, vars, history_mgr, command_history, oldpwd)
            }
        }
    }

    fn execute_simple(
        cmd: &SimpleCommand,
        vars: &mut Variables,
        history_mgr: &crate::history::HistoryManager,
        command_history: &mut Vec<String>,
        oldpwd: &mut Option<String>,
    ) -> Result<(), String> {
        // Handle variable assignments
        let mut temp_vars = vars.to_env_vars();
        for (key, value) in &cmd.assignments {
            let expanded_value = vars.expand(value);
            if cmd.name.is_empty() {
                // Permanent assignment
                vars.set(key.clone(), expanded_value.clone());
                temp_vars.insert(key.clone(), expanded_value);
            } else {
                // Temporary assignment for this command
                temp_vars.insert(key.clone(), expanded_value);
            }
        }

        if cmd.name.is_empty() {
            return Ok(());
        }

        // Check for builtins
        match handle_builtin(cmd, history_mgr, command_history, oldpwd) {
            Ok(BuiltinResult::HandledContinue) => return Ok(()),
            Ok(BuiltinResult::HandledExit(_)) => {
                return Err("exit not supported in subshell/pipeline".to_string())
            }
            Ok(BuiltinResult::SourceFile(_)) => {
                return Err("source not supported in subshell/pipeline".to_string())
            }
            Ok(BuiltinResult::NotHandled) => {}
            Err(e) => return Err(e),
        }

        let name = vars.expand(&cmd.name);
        if name.is_empty() {
            return Ok(());
        }

        let args: Vec<String> = cmd.args.iter().map(|arg| vars.expand(arg)).collect();

        let mut child = StdCommand::new(&name)
            .args(&args)
            .envs(&temp_vars)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|e| format!("Failed to execute '{}': {}", name, e))?;

        child
            .wait()
            .map_err(|e| format!("Failed to wait for command: {}", e))?;

        Ok(())
    }

    fn execute_subshell(
        pipelines: &[Vec<Command>],
        vars: &mut Variables,
        history_mgr: &crate::history::HistoryManager,
        command_history: &mut Vec<String>,
        oldpwd: &mut Option<String>,
    ) -> Result<(), String> {
        // Clone variables to simulate subshell environment
        let mut subshell_vars = vars.clone();

        // Save current directory
        let current_dir = std::env::current_dir().map_err(|e| e.to_string())?;

        // Execute pipelines sequentially
        let mut last_result = Ok(());
        for pipeline in pipelines {
            last_result = Self::execute_pipeline(
                pipeline,
                &mut subshell_vars,
                history_mgr,
                command_history,
                oldpwd,
            );
            // If a pipeline fails, do we stop?
            // In shell, 'false; echo hi' runs 'echo hi'.
            // But if execute_pipeline returns Err, it means internal error or failure to spawn?
            // If it returns Ok, it means commands ran (exit code might be non-zero).
            // execute_pipeline returns Result<(), String>. Err is for "failed to execute".
            // So if Err, we probably should stop.
            if last_result.is_err() {
                break;
            }
        }

        // Restore current directory
        std::env::set_current_dir(current_dir).map_err(|e| e.to_string())?;

        last_result
    }

    /// Execute a pipeline of commands (e.g., "echo hello | wc -c").
    /// Spawns all processes with connected pipes and waits for all to complete.
    pub fn execute_pipeline(
        pipeline: &[Command],
        vars: &mut Variables,
        history_mgr: &crate::history::HistoryManager,
        command_history: &mut Vec<String>,
        oldpwd: &mut Option<String>,
    ) -> Result<(), String> {
        if pipeline.is_empty() {
            return Err("Empty pipeline".to_string());
        }

        if pipeline.len() == 1 {
            return Self::execute(&pipeline[0], vars, history_mgr, command_history, oldpwd);
        }

        let mut children = Vec::new();
        let mut prev_stdout: Option<std::process::ChildStdout> = None;

        for (i, cmd) in pipeline.iter().enumerate() {
            // For pipeline, we only support SimpleCommands for now (or handle Subshells)
            // If it's a subshell, we can't easily pipe to/from it without more complex process management.
            // For now, let's assume pipeline components are SimpleCommands.
            // If we encounter a Subshell in a pipeline, we should probably spawn a new shell process or handle it.

            let (name, args, assignments) = match cmd {
                Command::Simple(simple) => (
                    simple.name.clone(),
                    simple.args.clone(),
                    simple.assignments.clone(),
                ),
                Command::Subshell(_) => {
                    return Err("Subshells in pipelines not yet fully supported".to_string())
                }
            };

            // Expand name and args
            let name = vars.expand(&name);
            if name.is_empty() {
                if !assignments.is_empty() {
                    // Assignment in pipeline - ignore for now as it would be in subshell
                    continue;
                }
                return Err("Empty command in pipeline".to_string());
            }

            let args: Vec<String> = args.iter().map(|arg| vars.expand(arg)).collect();

            // Prepare env vars (only temporary assignments for pipeline commands)
            let mut temp_vars = vars.to_env_vars();
            for (key, value) in &assignments {
                let expanded_value = vars.expand(value);
                temp_vars.insert(key.clone(), expanded_value);
            }

            let mut child_cmd = StdCommand::new(&name);
            child_cmd.args(&args);
            child_cmd.envs(&temp_vars);

            // Set stdin: inherit for first command, piped from previous command otherwise
            if i == 0 {
                child_cmd.stdin(Stdio::inherit());
            } else {
                match prev_stdout.take() {
                    Some(stdout) => {
                        child_cmd.stdin(stdout);
                    }
                    None => return Err("Failed to connect pipeline stdin".to_string()),
                }
            }

            // Set stdout: piped for all but last command, inherit for last
            if i < pipeline.len() - 1 {
                child_cmd.stdout(Stdio::piped());
            } else {
                child_cmd.stdout(Stdio::inherit());
            }

            child_cmd.stderr(Stdio::inherit());

            let mut child = child_cmd
                .spawn()
                .map_err(|e| format!("Failed to execute '{}' in pipeline: {}", name, e))?;

            // Save stdout for next iteration if not last command
            if i < pipeline.len() - 1 {
                prev_stdout = child.stdout.take();
            }

            children.push(child);
        }

        // Wait for all children to complete
        let mut last_status = Ok(());
        for (i, mut child) in children.into_iter().enumerate() {
            let status = child
                .wait()
                .map_err(|e| format!("Failed to wait for pipeline command: {}", e))?;

            // We only care about the exit status of the last command in the pipeline
            if i == pipeline.len() - 1 && !status.success() {
                if let Some(code) = status.code() {
                    last_status = Err(format!("Pipeline command exited with code {}", code));
                } else {
                    last_status = Err("Pipeline command terminated by signal".to_string());
                }
            }
        }

        last_status
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_echo() {
        let mut vars = Variables::new();
        let cmd = Command::Simple(SimpleCommand {
            name: "echo".into(),
            args: vec!["hello".into()],
            assignments: vec![],
        });
        let history_mgr = crate::history::HistoryManager::default();
        let mut command_history = vec![];
        let mut oldpwd = None;
        let res = Executor::execute(
            &cmd,
            &mut vars,
            &history_mgr,
            &mut command_history,
            &mut oldpwd,
        );
        assert!(res.is_ok());
    }

    #[test]
    fn test_execute_pipeline_single_command() {
        let mut vars = Variables::new();
        let pipeline = vec![Command::Simple(SimpleCommand {
            name: "echo".into(),
            args: vec!["hello".into()],
            assignments: vec![],
        })];
        let history_mgr = crate::history::HistoryManager::default();
        let mut command_history = vec![];
        let mut oldpwd = None;
        let res = Executor::execute_pipeline(
            &pipeline,
            &mut vars,
            &history_mgr,
            &mut command_history,
            &mut oldpwd,
        );
        assert!(res.is_ok());
    }

    #[test]
    fn test_execute_pipeline_echo_to_wc() {
        // Pipeline: echo "hello world" | wc -w
        // Expected: wc counts the words (should be 2)
        let mut vars = Variables::new();
        let pipeline = vec![
            Command::Simple(SimpleCommand {
                name: "echo".into(),
                args: vec!["hello".into(), "world".into()],
                assignments: vec![],
            }),
            Command::Simple(SimpleCommand {
                name: "wc".into(),
                args: vec!["-w".into()],
                assignments: vec![],
            }),
        ];
        let history_mgr = crate::history::HistoryManager::default();
        let mut command_history = vec![];
        let mut oldpwd = None;
        let res = Executor::execute_pipeline(
            &pipeline,
            &mut vars,
            &history_mgr,
            &mut command_history,
            &mut oldpwd,
        );
        assert!(res.is_ok());
    }

    #[test]
    fn test_execute_pipeline_empty() {
        let mut vars = Variables::new();
        let pipeline: Vec<Command> = vec![];
        let history_mgr = crate::history::HistoryManager::default();
        let mut command_history = vec![];
        let mut oldpwd = None;
        let res = Executor::execute_pipeline(
            &pipeline,
            &mut vars,
            &history_mgr,
            &mut command_history,
            &mut oldpwd,
        );
        assert!(res.is_err());
    }

    #[test]
    fn test_execute_pipeline_exit_status() {
        // Test that pipeline exit status is determined by the last command
        let mut vars = Variables::new();
        let history_mgr = crate::history::HistoryManager::default();
        let mut command_history = vec![];
        let mut oldpwd = None;

        // Case 1: false | true -> should succeed
        let pipeline_success = vec![
            Command::Simple(SimpleCommand {
                name: "false".into(),
                args: vec![],
                assignments: vec![],
            }),
            Command::Simple(SimpleCommand {
                name: "true".into(),
                args: vec![],
                assignments: vec![],
            }),
        ];
        let res = Executor::execute_pipeline(
            &pipeline_success,
            &mut vars,
            &history_mgr,
            &mut command_history,
            &mut oldpwd,
        );
        assert!(res.is_ok(), "false | true should succeed");

        // Case 2: true | false -> should fail
        let pipeline_fail = vec![
            Command::Simple(SimpleCommand {
                name: "true".into(),
                args: vec![],
                assignments: vec![],
            }),
            Command::Simple(SimpleCommand {
                name: "false".into(),
                args: vec![],
                assignments: vec![],
            }),
        ];
        let res = Executor::execute_pipeline(
            &pipeline_fail,
            &mut vars,
            &history_mgr,
            &mut command_history,
            &mut oldpwd,
        );
        assert!(res.is_err(), "true | false should fail");
    }
}
