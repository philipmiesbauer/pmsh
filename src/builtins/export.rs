use crate::parser::SimpleCommand;
use crate::variables::Variables;

use super::BuiltinResult;

pub fn execute(cmd: &SimpleCommand, vars: &mut Variables) -> Result<BuiltinResult, String> {
    if cmd.args.is_empty() {
        // Print all exported variables
        let mut exported: Vec<(String, String)> = vars
            .exported_vars()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        exported.sort_by(|a, b| a.0.cmp(&b.0));
        for (k, v) in exported {
            println!("export {}={}", k, v);
        }
        return Ok(BuiltinResult::HandledContinue);
    }

    for arg in &cmd.args {
        if let Some(eq_pos) = arg.find('=') {
            // export NAME=value — set and export
            let name = &arg[..eq_pos];
            let value = &arg[eq_pos + 1..];
            let expanded = vars.expand(value);
            vars.set(name.to_string(), expanded);
            vars.export(name);
        } else {
            // export NAME — mark existing variable as exported
            vars.export(arg);
        }
    }

    Ok(BuiltinResult::HandledContinue)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::variables::Variables;

    fn make_cmd(args: Vec<&str>) -> SimpleCommand {
        SimpleCommand {
            name: "export".into(),
            args: args.into_iter().map(String::from).collect(),
            assignments: vec![],
        }
    }

    #[test]
    fn test_export_with_value() {
        let mut vars = Variables::new();
        let cmd = make_cmd(vec!["MY_EXPORT=hello"]);
        execute(&cmd, &mut vars).unwrap();

        assert_eq!(vars.get("MY_EXPORT").map(|s| s.as_str()), Some("hello"));
        assert!(vars.is_exported("MY_EXPORT"));
    }

    #[test]
    fn test_export_existing_var() {
        let mut vars = Variables::new();
        vars.set("EXISTING".to_string(), "world".to_string());
        assert!(!vars.is_exported("EXISTING"));

        let cmd = make_cmd(vec!["EXISTING"]);
        execute(&cmd, &mut vars).unwrap();

        assert!(vars.is_exported("EXISTING"));
        assert_eq!(vars.get("EXISTING").map(|s| s.as_str()), Some("world"));
    }

    #[test]
    fn test_export_no_args_prints_exports() {
        // Just verify it returns HandledContinue without panicking
        let mut vars = Variables::new();
        let cmd = make_cmd(vec![]);
        let res = execute(&cmd, &mut vars).unwrap();
        assert!(matches!(res, BuiltinResult::HandledContinue));
    }

    #[test]
    fn test_export_unknown_var_is_noop() {
        // Mark a non-existent var as exported — POSIX says this is allowed
        let mut vars = Variables::new();
        let cmd = make_cmd(vec!["TOTALLY_NEW_VAR_XYZ"]);
        let res = execute(&cmd, &mut vars);
        assert!(res.is_ok());
        // var doesn't exist in shell yet
        assert!(vars.get("TOTALLY_NEW_VAR_XYZ").is_none());
        // but it is flagged as exported for when it gets set later
        assert!(vars.is_exported("TOTALLY_NEW_VAR_XYZ"));
    }

    #[test]
    fn test_export_multiple_args() {
        let mut vars = Variables::new();
        vars.set("A".to_string(), "1".to_string());
        vars.set("B".to_string(), "2".to_string());

        let cmd = make_cmd(vec!["A", "B"]);
        execute(&cmd, &mut vars).unwrap();

        assert!(vars.is_exported("A"));
        assert!(vars.is_exported("B"));
    }
}
