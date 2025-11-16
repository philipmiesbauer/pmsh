use crate::parser::Command;
use std::process::{Command as StdCommand, Stdio};

pub struct Executor;

impl Executor {
    pub fn execute(cmd: &Command) -> Result<(), String> {
        if cmd.name.is_empty() {
            return Err("Empty command".to_string());
        }

        let mut child = StdCommand::new(&cmd.name)
            .args(&cmd.args)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|e| format!("Failed to execute '{}': {}", cmd.name, e))?;

        child
            .wait()
            .map_err(|e| format!("Failed to wait for command: {}", e))?;

        Ok(())
    }

    /// Execute a pipeline of commands (e.g., "echo hello | wc -c").
    /// Spawns all processes with connected pipes and waits for all to complete.
    pub fn execute_pipeline(pipeline: &[Command]) -> Result<(), String> {
        if pipeline.is_empty() {
            return Err("Empty pipeline".to_string());
        }

        if pipeline.len() == 1 {
            return Self::execute(&pipeline[0]);
        }

        let mut children = Vec::new();
        let mut prev_stdout: Option<std::process::ChildStdout> = None;

        for (i, cmd) in pipeline.iter().enumerate() {
            if cmd.name.is_empty() {
                return Err("Empty command in pipeline".to_string());
            }

            let mut child_cmd = StdCommand::new(&cmd.name);
            child_cmd.args(&cmd.args);

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
                .map_err(|e| format!("Failed to execute '{}' in pipeline: {}", cmd.name, e))?;

            // Save stdout for next iteration if not last command
            if i < pipeline.len() - 1 {
                prev_stdout = child.stdout.take();
            }

            children.push(child);
        }

        // Wait for all children to complete
        for mut child in children {
            let status = child
                .wait()
                .map_err(|e| format!("Failed to wait for pipeline command: {}", e))?;

            if !status.success() {
                if let Some(code) = status.code() {
                    return Err(format!("Pipeline command exited with code {}", code));
                } else {
                    return Err("Pipeline command terminated by signal".to_string());
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_echo() {
        let cmd = Command {
            name: "echo".into(),
            args: vec!["hello".into()],
        };
        let res = Executor::execute(&cmd);
        assert!(res.is_ok());
    }

    #[test]
    fn test_execute_pipeline_single_command() {
        let pipeline = vec![Command {
            name: "echo".into(),
            args: vec!["hello".into()],
        }];
        let res = Executor::execute_pipeline(&pipeline);
        assert!(res.is_ok());
    }

    #[test]
    fn test_execute_pipeline_echo_to_wc() {
        // Pipeline: echo "hello world" | wc -w
        // Expected: wc counts the words (should be 2)
        let pipeline = vec![
            Command {
                name: "echo".into(),
                args: vec!["hello".into(), "world".into()],
            },
            Command {
                name: "wc".into(),
                args: vec!["-w".into()],
            },
        ];
        let res = Executor::execute_pipeline(&pipeline);
        assert!(res.is_ok());
    }

    #[test]
    fn test_execute_pipeline_empty() {
        let pipeline: Vec<Command> = vec![];
        let res = Executor::execute_pipeline(&pipeline);
        assert!(res.is_err());
    }
}
