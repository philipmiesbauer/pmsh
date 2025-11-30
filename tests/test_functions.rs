use expectrl::spawn;
use expectrl::Expect;

#[test]
fn test_function_definition_and_execution() {
    let mut p = spawn("cargo run").expect("Error spawning");

    // Wait for prompt
    p.expect("$ ").expect("Error reading prompt");

    // Define function
    p.send_line("foo() { echo inside_foo; }")
        .expect("Error sending line");
    p.expect("$ ").expect("Error reading prompt");

    // Call function
    p.send_line("foo").expect("Error sending line");
    p.expect("inside_foo").expect("Error reading output");
    p.expect("$ ").expect("Error reading prompt");

    // Exit
    p.send_line("exit").expect("Error sending exit");
}

#[test]
fn test_function_arguments() {
    let mut p = spawn("cargo run").expect("Error spawning");

    // Wait for prompt
    p.expect("$ ").expect("Error reading prompt");

    // Define function with args
    p.send_line("greet() { echo Hello $1; }")
        .expect("Error sending line");
    p.expect("$ ").expect("Error reading prompt");

    // Call function with arg
    p.send_line("greet World").expect("Error sending line");
    p.expect("Hello World").expect("Error reading output");
    p.expect("$ ").expect("Error reading prompt");

    // Call function with another arg
    p.send_line("greet Pmsh").expect("Error sending line");
    p.expect("Hello Pmsh").expect("Error reading output");
    p.expect("$ ").expect("Error reading prompt");

    // Exit
    p.send_line("exit").expect("Error sending exit");
}

use tempfile::NamedTempFile;
use std::io::Write;

#[test]
fn test_multiline_function_script() {
    let script_content = "
my_func() {
    echo line1
    echo line2
}
my_func
";
    let mut temp_file = NamedTempFile::new().expect("Error creating temp file");
    write!(temp_file, "{}", script_content).expect("Error writing script");
    let script_path = temp_file.path().to_str().unwrap();

    let mut p = spawn(format!("cargo run -- {}", script_path)).expect("Error spawning");
    
    // Script execution doesn't print prompts, just output
    p.expect("line1").expect("Error reading line1");
    p.expect("line2").expect("Error reading line2");
    
    // temp_file is automatically deleted when it goes out of scope
}

#[test]
fn test_function_keyword() {
    let mut p = spawn("cargo run").expect("Error spawning");

    // Wait for prompt
    p.expect("$ ").expect("Error reading prompt");

    // Define function using 'function' keyword (Bash style)
    p.send_line("function my_func { echo inside_func; }")
        .expect("Error sending line");
    p.expect("$ ").expect("Error reading prompt");

    // Call function
    p.send_line("my_func").expect("Error sending line");
    p.expect("inside_func").expect("Error reading output");
    p.expect("$ ").expect("Error reading prompt");

    // Exit
    p.send_line("exit").expect("Error sending exit");
}
