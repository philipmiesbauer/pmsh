use expectrl::{spawn, Expect, Regex};

#[test]
fn integration_repl_echo_and_exit() {
    // Path to the compiled binary - try Cargo-provided env var, fall back to target path
    let bin = std::env::var("CARGO_BIN_EXE_pmsh").unwrap_or_else(|_| {
        let manifest = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
        format!("{}/target/debug/pmsh", manifest)
    });

    let mut p = spawn(&bin).expect("failed to spawn pmsh");

    // Wait for initial prompt
    p.expect(Regex("\\$ ")).expect("did not see prompt");

    // Send echo command
    p.send_line("echo hello").expect("failed to send line");

    // Expect the echo output
    p.expect(Regex("hello")).expect("did not see echo output");

    // Send exit to quit
    p.send_line("exit").expect("failed to send exit");

    // Expect the Exiting message
    p.expect(Regex("Exiting."))
        .expect("did not see exiting message");
}
