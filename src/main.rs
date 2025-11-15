mod executor;
mod parser;

use executor::Executor;
use parser::Command;
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;

fn main() {
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
                    println!("Exiting.");
                    break;
                }

                // Parse and execute command
                if let Some(cmd) = Command::parse(&line) {
                    match Executor::execute(&cmd) {
                        Ok(()) => {},
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