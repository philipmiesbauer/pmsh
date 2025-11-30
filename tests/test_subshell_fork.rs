use expectrl::spawn;
use expectrl::Expect;
use std::time::Duration;

#[test]
fn test_subshell_variable_isolation() {
    let mut p = spawn("cargo run").expect("Error spawning");
    p.set_expect_timeout(Some(Duration::from_secs(5)));

    p.expect("$ ").expect("Error reading prompt");

    // Set variable in parent
    p.send_line("VAR=parent").expect("Error sending line");
    p.expect("$ ").expect("Error reading prompt");

    // Run subshell
    p.send_line("(VAR=child; echo $VAR)")
        .expect("Error sending line");
    p.expect("child").expect("Error reading child output");
    p.expect("$ ").expect("Error reading prompt");

    // Check parent variable
    p.send_line("echo $VAR").expect("Error sending line");
    p.expect("parent").expect("Error reading parent output");
    p.expect("$ ").expect("Error reading prompt");

    p.send_line("exit").expect("Error sending exit");
}

#[test]
fn test_subshell_directory_isolation() {
    let mut p = spawn("cargo run").expect("Error spawning");
    p.set_expect_timeout(Some(Duration::from_secs(5)));

    p.expect("$ ").expect("Error reading prompt");

    // Get current dir
    p.send_line("pwd").expect("Error sending line");
    let output = p.expect("$ ").expect("Error reading prompt");
    // output contains the pwd output + prompt.
    // Actually expectrl returns the match.
    // We can't easily capture the output this way without regex.
    // But we can verify isolation by checking if we are back.

    // Let's use a known directory.
    p.send_line("cd /").expect("Error sending line");
    p.expect("$ ").expect("Error reading prompt");

    // Subshell cd
    p.send_line("(cd /tmp; pwd)").expect("Error sending line");
    p.expect("/tmp").expect("Error reading subshell pwd");
    p.expect("$ ").expect("Error reading prompt");

    // Parent pwd should be /
    p.send_line("pwd").expect("Error sending line");
    p.expect("/\r\n").expect("Error reading parent pwd");
    // Note: expectrl might capture newlines.
    // "pwd" output is usually "/\n".
    // We might need to be careful with matching.

    p.expect("$ ").expect("Error reading prompt");

    p.send_line("exit").expect("Error sending exit");
}

#[test]
fn test_nested_subshells() {
    let mut p = spawn("cargo run").expect("Error spawning");
    p.set_expect_timeout(Some(Duration::from_secs(5)));

    p.expect("$ ").expect("Error reading prompt");

    p.send_line("(echo level1; (echo level2))")
        .expect("Error sending line");
    p.expect("level1").expect("Error reading level1");
    p.expect("level2").expect("Error reading level2");
    p.expect("$ ").expect("Error reading prompt");

    p.send_line("exit").expect("Error sending exit");
}
