mod executor;
mod history;
mod parser;
mod path_utils;
mod builtins;

use executor::Executor;
use history::HistoryManager;
use parser::Command;
use path_utils::expand_home;
use builtins::{handle_builtin, BuiltinResult};
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

fn main() {
    // Initialize history manager
    let history_mgr = HistoryManager::new().unwrap_or_else(|e| {
        eprintln!("Warning: Could not initialize history: {}", e);
        HistoryManager::default()
    });

    // Load existing history
    let mut command_history = history_mgr.load().unwrap_or_default();

    // This gets us the line editor with history
    let mut rl = DefaultEditor::new().expect("Failed to create editor");

    // The main loop
    loop {
        // Get current working directory and user
        let cwd = std::env::current_dir()
            .ok()
            .and_then(|p| p.to_str().map(|s| s.to_string()))
            .unwrap_or_else(|| ".".to_string());
        let cwd_display = expand_home(&cwd);

        let user = std::env::var("USER").unwrap_or_else(|_| "user".to_string());

        let prompt = format!("{}:{}$ ", user, cwd_display);

        // 1. READ
        let readline = rl.readline(&prompt);

        match readline {
            Ok(line) => {
                // Add the line to history
                let _ = rl.add_history_entry(line.as_str());

                // Handle builtins via testable handler
                if let Some(cmd) = Command::parse(&line) {
                    match handle_builtin(&cmd, &history_mgr, &mut command_history) {
                        Ok(BuiltinResult::HandledExit) => break,
                        Ok(BuiltinResult::HandledContinue) => continue,
                        Ok(BuiltinResult::NotHandled) => {
                            // Not a builtin; execute externally
                            match Executor::execute(&cmd) {
                                Ok(()) => {
                                    if let Err(e) = history_mgr.add_entry(&line, &mut command_history) {
                                        eprintln!("Warning: Could not save to history: {}", e);
                                    }
                                }
                                Err(e) => eprintln!("pmsh: {}", e),
                            }
                        }
                        Err(e) => eprintln!("Builtin error: {}", e),
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                // This is Ctrl+C
                println!("^C");
                continue; // Continue to the next loop iteration
            }
            Err(ReadlineError::Eof) => {
                // This is Ctrl+D
                // Save history before exiting
                if let Err(e) = history_mgr.save(&command_history) {
                    eprintln!("Warning: Could not save history: {}", e);
                }
                println!("^D");
                break; // Exit the loop
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
}
