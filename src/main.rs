mod builtins;
mod colors;
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

    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 {
        // Script execution mode
        let script_path = &args[1];
        let contents = std::fs::read_to_string(script_path).unwrap_or_else(|e| {
            eprintln!("Error reading script {}: {}", script_path, e);
            std::process::exit(1);
        });

        // Dummy editor for script execution (no history needed)
        struct ScriptEditor;
        impl LineEditor for ScriptEditor {
            fn readline(&mut self, _prompt: &str) -> ReadlineEvent {
                ReadlineEvent::Eof
            }
            fn add_history_entry(&mut self, _entry: &str) {}
        }

        let mut editor = ScriptEditor;
        let mut oldpwd: Option<String> = None;
        let executor = RealExecutor {};

        for line in contents.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            repl::execute_line(
                line,
                &mut editor,
                &history_mgr,
                &mut command_history,
                &executor,
                &mut oldpwd,
            );
        }
    } else {
        // Interactive REPL mode
        
        // This gets us the line editor with history
        let mut rl = DefaultEditor::new().expect("Failed to create editor");

        // Load history into rustyline
        for entry in &command_history {
            let _ = rl.add_history_entry(entry.as_str());
        }

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
}
