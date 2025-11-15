use crate::builtins::{handle_builtin, BuiltinResult};
use crate::history::HistoryManager;
use crate::parser::Command;
use crate::ui;

pub enum ReadlineEvent {
    Line(String),
    Interrupted,
    Eof,
    Other,
}

pub trait LineEditor {
    fn readline(&mut self, prompt: &str) -> ReadlineEvent;
    fn add_history_entry(&mut self, entry: &str);
}

pub trait ExecutorTrait {
    fn execute(&self, cmd: &Command) -> Result<(), String>;
}

pub struct RealExecutor;

impl ExecutorTrait for RealExecutor {
    fn execute(&self, cmd: &Command) -> Result<(), String> {
        crate::executor::Executor::execute(cmd)
    }
}

pub fn run_repl<E: ExecutorTrait, L: LineEditor>(
    editor: &mut L,
    history_mgr: &HistoryManager,
    command_history: &mut Vec<String>,
    executor: &E,
) {
    loop {
        let event = editor.readline(&ui::format_prompt());

        match event {
            ReadlineEvent::Line(line) => {
                editor.add_history_entry(&line);

                if let Some(cmd) = Command::parse(&line) {
                    match handle_builtin(&cmd, history_mgr, command_history) {
                        Ok(BuiltinResult::HandledExit) => break,
                        Ok(BuiltinResult::HandledContinue) => continue,
                        Ok(BuiltinResult::NotHandled) => match executor.execute(&cmd) {
                            Ok(()) => {
                                if let Err(e) = history_mgr.add_entry(&line, command_history) {
                                    eprintln!("Warning: Could not save to history: {}", e);
                                }
                            }
                            Err(e) => eprintln!("pmsh: {}", e),
                        },
                        Err(e) => eprintln!("Builtin error: {}", e),
                    }
                }
            }
            ReadlineEvent::Interrupted => {
                println!("^C");
                continue;
            }
            ReadlineEvent::Eof => {
                if let Err(e) = history_mgr.save(command_history) {
                    eprintln!("Warning: Could not save history: {}", e);
                }
                println!("^D");
                break;
            }
            ReadlineEvent::Other => {
                // treat as generic error and break
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockEditor {
        events: std::collections::VecDeque<ReadlineEvent>,
        history: Vec<String>,
    }

    impl MockEditor {
        fn new(events: Vec<ReadlineEvent>) -> Self {
            Self {
                events: events.into(),
                history: Vec::new(),
            }
        }
    }

    impl LineEditor for MockEditor {
        fn readline(&mut self, _prompt: &str) -> ReadlineEvent {
            self.events.pop_front().unwrap_or(ReadlineEvent::Eof)
        }

        fn add_history_entry(&mut self, entry: &str) {
            self.history.push(entry.to_string());
        }
    }

    struct MockExecutor {
        calls: std::cell::RefCell<Vec<Command>>,
    }

    impl MockExecutor {
        fn new() -> Self {
            Self {
                calls: Default::default(),
            }
        }
    }

    impl ExecutorTrait for MockExecutor {
        fn execute(&self, cmd: &Command) -> Result<(), String> {
            self.calls.borrow_mut().push(cmd.clone());
            Ok(())
        }
    }

    #[test]
    fn test_repl_executes_command_and_exits_on_eof() {
        let events = vec![
            ReadlineEvent::Line("echo hello".to_string()),
            ReadlineEvent::Eof,
        ];
        let mut editor = MockEditor::new(events);

        let mgr = HistoryManager::new().unwrap_or_else(|_| HistoryManager::default());
        let mut history: Vec<String> = Vec::new();

        let executor = MockExecutor::new();

        run_repl(&mut editor, &mgr, &mut history, &executor);

        // executor should have been called once with echo
        let calls = executor.calls.borrow();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "echo");
        assert_eq!(calls[0].args, vec!["hello".to_string()]);
    }

    #[test]
    fn test_repl_builtins_flow() {
        // create tmp dir to cd into
        let tmp = tempfile::TempDir::new().unwrap();
        let tmp_path = tmp.path().to_string_lossy().to_string();

        // events: cd tmp; history; exit
        let events = vec![
            ReadlineEvent::Line(format!("cd {}", tmp_path)),
            ReadlineEvent::Line("history".to_string()),
            ReadlineEvent::Line("exit".to_string()),
        ];

        let mut editor = MockEditor::new(events);

        let mgr = HistoryManager::new().unwrap_or_else(|_| HistoryManager::default());
        let mut history: Vec<String> = Vec::new();

        let executor = MockExecutor::new();

        let orig = std::env::current_dir().unwrap();
        run_repl(&mut editor, &mgr, &mut history, &executor);

        // ensure history recorded the cd entry and restore cwd
        assert!(history.iter().any(|h| h.starts_with("cd ")));
        let _ = std::env::set_current_dir(orig);
    }

    #[test]
    fn test_repl_executor_error_does_not_save_history() {
        // Simulate an executor that returns an error
        struct FailingExecutor;
        impl ExecutorTrait for FailingExecutor {
            fn execute(&self, _cmd: &Command) -> Result<(), String> {
                Err("execution failed".to_string())
            }
        }

        let events = vec![
            ReadlineEvent::Line("nonexistent arg".to_string()),
            ReadlineEvent::Eof,
        ];
        let mut editor = MockEditor::new(events);

        // ensure history is written to a temp HOME so add_entry/save won't interfere with real HOME
        let tmp_home = tempfile::TempDir::new().unwrap();
        let original = std::env::var("HOME").ok();
        std::env::set_var("HOME", tmp_home.path().to_string_lossy().as_ref());

        let mgr = HistoryManager::new().unwrap_or_else(|_| HistoryManager::default());
        let mut history: Vec<String> = Vec::new();

        let exec = FailingExecutor;
        run_repl(&mut editor, &mgr, &mut history, &exec);

        // executor failed so history should not contain the failed command
        assert!(history.is_empty());

        match original {
            Some(v) => std::env::set_var("HOME", v),
            None => std::env::remove_var("HOME"),
        }
    }
}
