use expectrl::spawn;
use expectrl::ControlCode;
use expectrl::Expect;
use std::thread;
use std::time::Duration;

#[test]
fn test_reverse_search() {
    // 1. Start pmsh, run a command, and exit.
    let mut p = spawn("cargo run --bin pmsh").expect("Error spawning pmsh");
    
    // Wait for prompt - assuming it ends with "$ " or similar, or just wait a bit
    thread::sleep(Duration::from_secs(2)); 
    
    p.send_line("echo unique_search_term").expect("Failed to send command");
    thread::sleep(Duration::from_millis(500));
    
    p.send_line("exit").expect("Failed to exit");
    thread::sleep(Duration::from_millis(500));

    // 2. Restart pmsh
    let mut p = spawn("cargo run --bin pmsh").expect("Error spawning pmsh");
    thread::sleep(Duration::from_secs(2));

    // 3. Press Ctrl+R
    p.send("\x12").expect("Failed to send Ctrl+R");
    thread::sleep(Duration::from_millis(500));

    // 4. Type "unique"
    p.send("unique").expect("Failed to send search term");
    thread::sleep(Duration::from_millis(500));

    // 5. Expect the full command to appear
    // Note: expectrl might capture the output including the search prompt
    // We want to verify that "echo unique_search_term" is visible/selected.
    // In rustyline, it usually shows "(reverse-i-search)`unique': echo unique_search_term"
    
    // We can try to read a line or check buffer. 
    // For simplicity in this environment, let's just hit enter and check output.
    p.send("\n").expect("Failed to hit enter");
    thread::sleep(Duration::from_millis(500));
    
    // The output of the command should be "unique_search_term"
    p.expect("unique_search_term").expect("Did not find expected output from reverse search");
}
