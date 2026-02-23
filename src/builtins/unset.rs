use crate::parser::SimpleCommand;
use crate::variables::Variables;

use super::BuiltinResult;

pub fn execute(cmd: &SimpleCommand, vars: &mut Variables) -> Result<BuiltinResult, String> {
    if cmd.args.is_empty() {
        return Err("unset: expected at least one argument".to_string());
    }
    for name in &cmd.args {
        vars.unset(name);
    }
    Ok(BuiltinResult::HandledContinue)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::variables::Variables;

    fn make_cmd(args: Vec<&str>) -> SimpleCommand {
        SimpleCommand {
            name: "unset".into(),
            args: args.into_iter().map(String::from).collect(),
            assignments: vec![],
        }
    }

    #[test]
    fn test_unset_existing_var() {
        let mut vars = Variables::new();
        vars.set("MY_VAR".to_string(), "hello".to_string());
        assert!(vars.get("MY_VAR").is_some());

        let cmd = make_cmd(vec!["MY_VAR"]);
        let res = execute(&cmd, &mut vars).unwrap();
        assert!(matches!(res, BuiltinResult::HandledContinue));
        assert!(vars.get("MY_VAR").is_none());
    }

    #[test]
    fn test_unset_nonexistent_var_is_noop() {
        let mut vars = Variables::new();
        let cmd = make_cmd(vec!["DOES_NOT_EXIST_XYZ"]);
        let res = execute(&cmd, &mut vars);
        assert!(res.is_ok());
    }

    #[test]
    fn test_unset_multiple_vars() {
        let mut vars = Variables::new();
        vars.set("A".to_string(), "1".to_string());
        vars.set("B".to_string(), "2".to_string());

        let cmd = make_cmd(vec!["A", "B"]);
        execute(&cmd, &mut vars).unwrap();

        assert!(vars.get("A").is_none());
        assert!(vars.get("B").is_none());
    }

    #[test]
    fn test_unset_no_args_returns_error() {
        let mut vars = Variables::new();
        let cmd = make_cmd(vec![]);
        let res = execute(&cmd, &mut vars);
        assert!(res.is_err());
    }
}
