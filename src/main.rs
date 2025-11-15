mod executor;
mod history;
mod parser;

use executor::Executor;
use history::HistoryManager;
use parser::Command;
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
        // 1. READ
        let readline = rl.readline("pmsh> "); // Your prompt

        match readline {
            Ok(line) => {
                // Add the line to history
                let _ = rl.add_history_entry(line.as_str());

                // Handle built-in commands
                if line.trim() == "exit" {
                    // Save history before exiting
                    if let Err(e) = history_mgr.save(&command_history) {
                        eprintln!("Warning: Could not save history: {}", e);
                    }
                    println!("Exiting.");
                    break; // Exit the loop
                }

                if line.trim() == "history" {
                    for (idx, entry) in command_history.iter().enumerate() {
                        println!("{}: {}", idx + 1, entry);
                    }
                    continue; // Continue to the next loop iteration
                }

                // Parse and execute command
                if let Some(cmd) = Command::parse(&line) {
                    match Executor::execute(&cmd) {
                        Ok(()) => {
                            // Add to persistent history on success
                            if let Err(e) = history_mgr.add_entry(&line, &mut command_history) {
                                eprintln!("Warning: Could not save to history: {}", e);
                            }
                        }
                        Err(e) => eprintln!("pmsh: {}", e),
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
