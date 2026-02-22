use crate::parser::SimpleCommand;

pub fn execute(cmd: &SimpleCommand) -> Result<(), String> {
    if !cmd.args.is_empty() {
        return Err(format!("{}: no arguments expected", cmd.name));
    }

    let version = env!("CARGO_PKG_VERSION");
    let name = env!("CARGO_PKG_NAME");

    println!("{} version {}", name, version);
    println!("A simple shell written in Rust.");
    println!("GitHub: https://github.com/philipmiesbauer/pmsh");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::SimpleCommand;

    #[test]
    fn test_version_builtin_success() {
        let cmd = SimpleCommand {
            name: "version".to_string(),
            args: vec![],
            assignments: vec![],
        };
        assert!(execute(&cmd).is_ok());
    }

    #[test]
    fn test_version_builtin_with_args() {
        let cmd = SimpleCommand {
            name: "version".to_string(),
            args: vec!["extra".to_string()],
            assignments: vec![],
        };
        let result = execute(&cmd);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "version: no arguments expected");
    }
}
