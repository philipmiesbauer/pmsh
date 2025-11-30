use crate::builtins::{handle_builtin, BuiltinResult};
use crate::colors::red;
use crate::history::HistoryManager;
use crate::parser::Command;

use crate::ui;
use crate::variables::Variables;

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
    fn execute(
        &self,
        cmd: &Command,
        vars: &mut Variables,
        history_mgr: &HistoryManager,
        command_history: &mut Vec<String>,
        oldpwd: &mut Option<String>,
    ) -> Result<(), String>;
    fn execute_pipeline(
        &self,
        pipeline: &[Command],
        vars: &mut Variables,
        history_mgr: &HistoryManager,
        command_history: &mut Vec<String>,
        oldpwd: &mut Option<String>,
    ) -> Result<(), String>;
}

pub struct RealExecutor;

impl ExecutorTrait for RealExecutor {
    fn execute(
        &self,
        cmd: &Command,
        vars: &mut Variables,
        history_mgr: &HistoryManager,
        command_history: &mut Vec<String>,
        oldpwd: &mut Option<String>,
    ) -> Result<(), String> {
        crate::executor::Executor::execute(cmd, vars, history_mgr, command_history, oldpwd)
    }

    fn execute_pipeline(
        &self,
        pipeline: &[Command],
        vars: &mut Variables,
        history_mgr: &HistoryManager,
        command_history: &mut Vec<String>,
        oldpwd: &mut Option<String>,
    ) -> Result<(), String> {
        crate::executor::Executor::execute_pipeline(
            pipeline,
            vars,
            history_mgr,
            command_history,
            oldpwd,
        )
    }
}

#[allow(dead_code)]
pub struct NoOpEditor;
impl LineEditor for NoOpEditor {
    fn readline(&mut self, _prompt: &str) -> ReadlineEvent {
        ReadlineEvent::Eof
    }
    fn add_history_entry(&mut self, _entry: &str) {}
}

pub fn execute_line<E: ExecutorTrait, L: LineEditor>(
    line: &str,
    editor: &mut L,
    history_mgr: &HistoryManager,
    command_history: &mut Vec<String>,
    executor: &E,
    oldpwd: &mut Option<String>,
    vars: &mut Variables,
) -> bool {
    editor.add_history_entry(line);

    if let Some(pipeline) = Command::parse_pipeline(line) {
        return execute_pipeline_struct(
            &pipeline,
            history_mgr,
            command_history,
            executor,
            oldpwd,
            vars,
        );
    }
    true
}

pub fn execute_pipeline_struct<E: ExecutorTrait>(
    pipeline: &[Command],
    history_mgr: &HistoryManager,
    command_history: &mut Vec<String>,
    executor: &E,
    oldpwd: &mut Option<String>,
    vars: &mut Variables,
) -> bool {
    if pipeline.len() == 1 {
        // Single command: check for builtins
        let cmd = &pipeline[0];
        let builtin_res = if let Command::Simple(simple) = cmd {
            handle_builtin(simple, history_mgr, command_history, oldpwd)
        } else {
            Ok(BuiltinResult::NotHandled)
        };

        match builtin_res {
            Ok(BuiltinResult::HandledExit(code)) => std::process::exit(code),
            Ok(BuiltinResult::HandledContinue) => return true,
            Ok(BuiltinResult::SourceFile(path)) => {
                let contents = match std::fs::read_to_string(&path) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("pmsh: source: {}: {}", path, e);
                        return true;
                    }
                };
                // Use parse_script to handle multiline commands correctly
                if let Some(pipelines) = Command::parse_script(&contents) {
                    for pipeline in pipelines {
                        if !execute_pipeline_struct(
                            &pipeline,
                            history_mgr,
                            command_history,
                            executor,
                            oldpwd,
                            vars,
                        ) {
                            return false;
                        }
                    }
                }
                return true;
            }
            Ok(BuiltinResult::NotHandled) => {
                match executor.execute(cmd, vars, history_mgr, command_history, oldpwd) {
                    Ok(()) => {
                        // History saving is handled by the caller (execute_line) for the full line.
                        // We don't save individual commands from scripts/pipelines here.
                    }
                    Err(e) => eprintln!("pmsh: {}", red(&e.to_string())),
                }
            }
            Err(e) => eprintln!("Builtin error: {}", red(&e.to_string())),
        }
    } else {
        // Pipeline of multiple commands: execute via pipeline
        match executor.execute_pipeline(pipeline, vars, history_mgr, command_history, oldpwd) {
            Ok(()) => {
                // History saving removed
            }
            Err(e) => eprintln!("pmsh: {}", red(&e.to_string())),
        }
    }
    true
}

pub fn run_repl<E: ExecutorTrait, L: LineEditor>(
    editor: &mut L,
    history_mgr: &HistoryManager,
    command_history: &mut Vec<String>,
    executor: &E,
) {
    let mut oldpwd: Option<String> = None;
    let mut vars = Variables::new();

    // REPL: Read-Eval-Print Loop
    loop {
        // Read a line from the user
        let event = editor.readline(&ui::format_prompt());

        // Evaluate the line and print output or handle errors
        match event {
            ReadlineEvent::Line(line) => {
                if !execute_line(
                    &line,
                    editor,
                    history_mgr,
                    command_history,
                    executor,
                    &mut oldpwd,
                    &mut vars,
                ) {
                    break;
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
        fn execute(
            &self,
            cmd: &Command,
            _vars: &mut Variables,
            _history_mgr: &HistoryManager,
            _command_history: &mut Vec<String>,
            _oldpwd: &mut Option<String>,
        ) -> Result<(), String> {
            self.calls.borrow_mut().push(cmd.clone());
            Ok(())
        }

        fn execute_pipeline(
            &self,
            pipeline: &[Command],
            _vars: &mut Variables,
            _history_mgr: &HistoryManager,
            _command_history: &mut Vec<String>,
            _oldpwd: &mut Option<String>,
        ) -> Result<(), String> {
            for cmd in pipeline {
                self.calls.borrow_mut().push(cmd.clone());
            }
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
        // executor should have been called once with echo
        let calls = executor.calls.borrow();
        assert_eq!(calls.len(), 1);
        if let Command::Simple(cmd) = &calls[0] {
            assert_eq!(cmd.name, "echo");
            assert_eq!(cmd.args, vec!["hello".to_string()]);
        } else {
            panic!("Expected Simple command");
        }
    }

    #[test]
    fn test_repl_executes_pipeline() {
        let events = vec![
            ReadlineEvent::Line("echo hello | wc -w".to_string()),
            ReadlineEvent::Eof,
        ];
        let mut editor = MockEditor::new(events);

        let mgr = HistoryManager::new().unwrap_or_else(|_| HistoryManager::default());
        let mut history: Vec<String> = Vec::new();

        let executor = MockExecutor::new();

        run_repl(&mut editor, &mgr, &mut history, &executor);

        // executor's execute_pipeline should have been called with 2 commands
        // executor's execute_pipeline should have been called with 2 commands
        let calls = executor.calls.borrow();
        assert_eq!(calls.len(), 2);
        if let Command::Simple(cmd) = &calls[0] {
            assert_eq!(cmd.name, "echo");
            assert_eq!(cmd.args, vec!["hello".to_string()]);
        } else {
            panic!("Expected Simple command");
        }
        if let Command::Simple(cmd) = &calls[1] {
            assert_eq!(cmd.name, "wc");
            assert_eq!(cmd.args, vec!["-w".to_string()]);
        } else {
            panic!("Expected Simple command");
        }
    }

    #[test]
    #[serial_test::serial]
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
    #[serial_test::serial]
    fn test_repl_executor_error_does_not_save_history() {
        // Simulate an executor that returns an error
        struct FailingExecutor;
        impl ExecutorTrait for FailingExecutor {
            fn execute(
                &self,
                _cmd: &Command,
                _vars: &mut Variables,
                _history_mgr: &HistoryManager,
                _command_history: &mut Vec<String>,
                _oldpwd: &mut Option<String>,
            ) -> Result<(), String> {
                Err("execution failed".to_string())
            }

            fn execute_pipeline(
                &self,
                _pipeline: &[Command],
                _vars: &mut Variables,
                _history_mgr: &HistoryManager,
                _command_history: &mut Vec<String>,
                _oldpwd: &mut Option<String>,
            ) -> Result<(), String> {
                Err("pipeline failed".to_string())
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
