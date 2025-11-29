use expectrl::{spawn, Expect, Regex};
use regex;

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

#[test]
fn integration_repl_subshell_env_isolation() {
    let bin = std::env::var("CARGO_BIN_EXE_pmsh").unwrap_or_else(|_| {
        let manifest = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
        format!("{}/target/debug/pmsh", manifest)
    });
    let mut p = spawn(&bin).expect("failed to spawn pmsh");
    p.expect(Regex("\\$ ")).expect("did not see prompt");

    p.send_line("(export SUBSHELL_TEST=123)").expect("failed to send line");
    p.expect(Regex("\\$ ")).expect("did not see prompt");

    p.send_line("echo Value: $SUBSHELL_TEST").expect("failed to send echo");
    // Expect "Value: " followed by newline (and maybe \r)
    // We use a regex that matches "Value: " at the end of a line
    p.expect(Regex("Value: ")).expect("Output mismatch");
    // Verify 123 is NOT present
    // We can't easily verify absence with expectrl without reading the whole buffer.
    // But if we expect prompt immediately after "Value: ", it implies no "123".
    p.expect(Regex("\\$ ")).expect("did not see prompt");
}

#[test]
fn integration_repl_subshell_cd_isolation() {
    let bin = std::env::var("CARGO_BIN_EXE_pmsh").unwrap_or_else(|_| {
        let manifest = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
        format!("{}/target/debug/pmsh", manifest)
    });
    let mut p = spawn(&bin).expect("failed to spawn pmsh");
    p.expect(Regex("\\$ ")).expect("did not see prompt");

    let current_dir = std::env::current_dir().unwrap();
    let current_dir_str = current_dir.to_str().unwrap();

    p.send_line("(cd /tmp)").expect("failed to send line");
    p.expect(Regex("\\$ ")).expect("did not see prompt");

    p.send_line("pwd").expect("failed to send pwd");
    // We expect the original directory
    p.expect(Regex(regex::escape(current_dir_str).as_str())).expect("CD leaked!");
}
