use expectrl::{spawn, Expect, Regex, ControlCode};
use std::{thread, time};

#[test]
fn test_autocomplete_cargo_toml() {
    let bin = std::env::var("CARGO_BIN_EXE_pmsh").unwrap_or_else(|_| {
        let manifest = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
        format!("{}/target/debug/pmsh", manifest)
    });

    let mut p = spawn(&bin).expect("failed to spawn pmsh");

    // Wait for prompt
    p.expect(Regex("\\$ ")).expect("did not see prompt");

    // Type partial filename
    p.send("ls Cargo.t").expect("failed to send partial command");
    
    // Send TAB
    p.send("\t").expect("failed to send tab");

    // Expect completion to Cargo.toml (or at least Cargo.t -> Cargo.to...)
    // Since Cargo.toml and Cargo.lock both exist, it might complete to Cargo.
    // Let's try something more unique if possible, or check if it lists options.
    // Actually, Cargo.toml and Cargo.lock share "Cargo.", so it should complete to "Cargo.".
    // If I type "Cargo.t", it should complete to "Cargo.toml" because "Cargo.lock" doesn't match "t".
    
    // We need to wait a bit for the completion to happen and be echoed back
    thread::sleep(time::Duration::from_millis(100));

    // Check if the line now contains "Cargo.toml"
    // Note: expectrl might consume the output, so we might need to check what's on screen.
    // But send_line usually waits for the prompt. Here we are in the middle of a line.
    // We can send a newline and check the output of ls.
    
    p.send_line("").expect("failed to send newline");
    
    // If completion worked, it should have executed "ls Cargo.toml"
    p.expect(Regex("Cargo.toml")).expect("did not see Cargo.toml in output");
}
