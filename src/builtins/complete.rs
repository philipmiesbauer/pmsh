use crate::completion_registry::{CompSpec, COMP_REGISTRY};
use crate::parser::SimpleCommand;

pub fn execute(cmd: &SimpleCommand) -> Result<(), String> {
    // Usage: complete [-r] [-W wordlist] command...
    let mut args = cmd.args.iter().peekable();
    let mut wordlist = None;
    let mut remove = false;
    let mut commands = Vec::new();

    while let Some(arg) = args.next() {
        if arg == "-r" {
            remove = true;
        } else if arg == "-W" {
            if let Some(w) = args.next() {
                wordlist = Some(w.clone());
            } else {
                return Err("complete: option requires an argument -- W".to_string());
            }
        } else {
            commands.push(arg.clone());
        }
    }

    if commands.is_empty() {
        return Ok(());
    }

    let mut registry = match COMP_REGISTRY.write() {
        Ok(guard) => guard,
        Err(_) => return Err("failed to acquire completion registry lock".to_string()),
    };

    if remove {
        for command in commands {
            registry.remove(&command);
        }
    } else {
        let spec = CompSpec { wordlist };
        for command in commands {
            registry.register(command, spec.clone());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::SimpleCommand;

    #[test]
    fn test_complete_builtin_no_args() {
        let cmd = SimpleCommand {
            name: "complete".to_string(),
            args: vec![],
            assignments: vec![],
        };
        assert!(execute(&cmd).is_ok());
    }

    #[test]
    fn test_complete_builtin_register() {
        let cmd = SimpleCommand {
            name: "complete".to_string(),
            args: vec!["-W".to_string(), "foo bar".to_string(), "mycmd".to_string()],
            assignments: vec![],
        };
        assert!(execute(&cmd).is_ok());

        let registry = COMP_REGISTRY.read().unwrap();
        let spec = registry.get("mycmd");
        assert!(spec.is_some());
        assert_eq!(spec.unwrap().wordlist.unwrap(), "foo bar");
    }

    #[test]
    fn test_complete_builtin_remove() {
        // Register first
        let cmd_reg = SimpleCommand {
            name: "complete".to_string(),
            args: vec!["-W".to_string(), "foo".to_string(), "rmcmd".to_string()],
            assignments: vec![],
        };
        assert!(execute(&cmd_reg).is_ok());

        assert!(COMP_REGISTRY.read().unwrap().get("rmcmd").is_some());

        // Remove
        let cmd_rm = SimpleCommand {
            name: "complete".to_string(),
            args: vec!["-r".to_string(), "rmcmd".to_string()],
            assignments: vec![],
        };
        assert!(execute(&cmd_rm).is_ok());

        assert!(COMP_REGISTRY.read().unwrap().get("rmcmd").is_none());
    }

    #[test]
    fn test_complete_builtin_missing_wordlist() {
        let cmd = SimpleCommand {
            name: "complete".to_string(),
            args: vec!["-W".to_string()],
            assignments: vec![],
        };
        let result = execute(&cmd);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "complete: option requires an argument -- W"
        );
    }
}
