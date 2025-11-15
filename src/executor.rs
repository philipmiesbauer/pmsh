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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_echo() {
        let cmd = Command { name: "echo".into(), args: vec!["hello".into()] };
        let res = Executor::execute(&cmd);
        assert!(res.is_ok());
    }
}
