mod builtins;
mod colors;
mod executor;
mod functions;
mod history;
mod parser;
mod path_utils;
mod ui;
mod variables;

use history::HistoryManager;
use repl::{run_repl, LineEditor, ReadlineEvent, RealExecutor};
use rustyline::error::ReadlineError;
use rustyline::{history::DefaultHistory, Editor};
mod autocomplete;
mod repl;

use autocomplete::PmshHelper;
use functions::Functions;

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

        let _editor = ScriptEditor;
        let mut oldpwd: Option<String> = None;
        let executor = RealExecutor {};
        let mut vars = variables::Variables::new();
        let mut functions = Functions::new();

        use crate::parser::Command;
        match Command::parse_script(&contents) {
            Ok(pipelines) => {
                for pipeline in pipelines {
                    if !repl::execute_pipeline_struct(
                        &pipeline,
                        &history_mgr,
                        &mut command_history,
                        &executor,
                        &mut oldpwd,
                        &mut vars,
                        &mut functions,
                    ) {
                        break;
                    }
                }
            }
            Err(e) => {
                eprintln!("Error parsing script: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        // Interactive REPL mode

        // This gets us the line editor with history
        let config = rustyline::Config::builder()
            .completion_type(rustyline::CompletionType::List)
            .build();
        let mut rl: Editor<PmshHelper, DefaultHistory> =
            Editor::with_config(config).expect("Failed to create editor");
        rl.set_helper(Some(PmshHelper::new()));

        // Load history into rustyline
        for entry in &command_history {
            let _ = rl.add_history_entry(entry.as_str());
        }

        // Wrap the rustyline editor as a LineEditor implementation
        struct RustyEditor {
            inner: Editor<PmshHelper, DefaultHistory>,
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
