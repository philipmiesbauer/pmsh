use expectrl::{spawn, Expect, Regex};
use std::{thread, time};

#[test]
fn test_compgen_builtin() {
    let bin = std::env::var("CARGO_BIN_EXE_pmsh").unwrap_or_else(|_| {
        let manifest = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
        format!("{}/target/debug/pmsh", manifest)
    });

    let mut p = spawn(&bin).expect("failed to spawn pmsh");

    p.expect(Regex("\\$ ")).expect("did not see prompt");

    // Send compgen command
    p.send_line("compgen -W \"foo bar baz\" -- b")
        .expect("failed to send compgen");

    p.expect("bar\r\nbaz\r\n").expect("did not see compgen output");
    p.expect(Regex("\\$ ")).expect("did not see prompt");

    p.send_line("exit").expect("failed to send exit");
}
