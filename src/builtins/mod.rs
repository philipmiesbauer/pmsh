mod cd;
mod exit;
mod history;

use crate::history::HistoryManager;
use crate::parser::Command;

pub enum BuiltinResult {
    HandledContinue,
    HandledExit,
    NotHandled,
}

pub fn handle_builtin(
    cmd: &Command,
    history_mgr: &HistoryManager,
    command_history: &mut Vec<String>,
    oldpwd: &mut Option<String>,
) -> Result<BuiltinResult, String> {
    match cmd.name.as_str() {
        "exit" => exit::execute(cmd, history_mgr, command_history),
        "history" => history::execute(cmd, history_mgr, command_history),
        "cd" => cd::execute(cmd, history_mgr, command_history, oldpwd),
        _ => Ok(BuiltinResult::NotHandled),
    }
}
