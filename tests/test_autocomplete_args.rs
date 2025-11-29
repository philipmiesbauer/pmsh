use expectrl::{spawn, Expect, Regex};
use std::{thread, time};

#[test]
fn test_autocomplete_argument() {
    let bin = std::env::var("CARGO_BIN_EXE_pmsh").unwrap_or_else(|_| {
        let manifest = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
        format!("{}/target/debug/pmsh", manifest)
    });

    let mut p = spawn(&bin).expect("failed to spawn pmsh");

    p.expect(Regex("\\$ ")).expect("did not see prompt");

    // Try to complete an argument: "ls Cargo.t"
    p.send("ls Cargo.t").expect("failed to send partial command");
    p.send("\t").expect("failed to send tab");
    
    thread::sleep(time::Duration::from_millis(100));
    p.send_line("").expect("failed to send newline");
    
    p.expect(Regex("Cargo.toml")).expect("did not see Cargo.toml in output");
}
