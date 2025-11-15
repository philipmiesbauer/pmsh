mod builtins;
mod executor;
mod history;
mod parser;
mod path_utils;
mod ui;

use history::HistoryManager;
use repl::{run_repl, LineEditor, ReadlineEvent, RealExecutor};
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
mod repl;

fn main() {
    // Initialize history manager
    let history_mgr = HistoryManager::new().unwrap_or_else(|e| {
        eprintln!("Warning: Could not initialize history: {}", e);
        HistoryManager::default()
    });

    // Load existing history
    let mut command_history = history_mgr.load().unwrap_or_default();

    // This gets us the line editor with history
    let rl = DefaultEditor::new().expect("Failed to create editor");

    // Wrap the rustyline editor as a LineEditor implementation
    struct RustyEditor {
        inner: rustyline::DefaultEditor,
    }
    impl LineEditor for RustyEditor {
        fn readline(&mut self, prompt: &str) -> ReadlineEvent {
            match self.inner.readline(prompt) {
                Ok(line) => ReadlineEvent::Line(line),
                Err(ReadlineError::Interrupted) => ReadlineEvent::Interrupted,
                Err(ReadlineError::Eof) => ReadlineEvent::Eof,
                Err(_e) => ReadlineEvent::Other,
            }
        }

        fn add_history_entry(&mut self, entry: &str) {
            let _ = self.inner.add_history_entry(entry);
        }
    }

    let mut editor = RustyEditor { inner: rl };

    // Run the refactored REPL loop
    run_repl(
        &mut editor,
        &history_mgr,
        &mut command_history,
        &RealExecutor {},
    );
}
