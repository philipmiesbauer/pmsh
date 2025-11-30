mod cd;
pub mod common;
mod exit;
mod history;

use crate::history::HistoryManager;
use crate::parser::SimpleCommand;

pub enum BuiltinResult {
    HandledContinue,
    HandledExit(i32),   // Exit with code
    SourceFile(String), // Source a file
    NotHandled,
}

pub fn handle_builtin(
    cmd: &SimpleCommand,
    history_mgr: &HistoryManager,
    command_history: &mut Vec<String>,
    oldpwd: &mut Option<String>,
) -> Result<BuiltinResult, String> {
    let simple_cmd = cmd;

    match simple_cmd.name.as_str() {
        "exit" => exit::execute(simple_cmd, history_mgr, command_history),
        "history" => history::execute(simple_cmd, history_mgr, command_history),
        "cd" => cd::execute(simple_cmd, history_mgr, command_history, oldpwd),
        "source" | "." => {
            if simple_cmd.args.len() != 1 {
                return Err(format!("{}: expected 1 argument", simple_cmd.name));
            }
            Ok(BuiltinResult::SourceFile(simple_cmd.args[0].clone()))
        }
        _ => Ok(BuiltinResult::NotHandled),
    }
}
