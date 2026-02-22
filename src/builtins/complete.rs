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
