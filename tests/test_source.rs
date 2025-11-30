use expectrl::{spawn, Expect, Regex};
use std::io::Write;

#[test]
fn test_source_builtin() {
    let bin = std::env::var("CARGO_BIN_EXE_pmsh").unwrap_or_else(|_| {
        let manifest = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
        format!("{}/target/debug/pmsh", manifest)
    });

    // Create a temporary script file
    let mut script_file = tempfile::NamedTempFile::new().expect("failed to create temp file");
    let script_path = script_file.path().to_string_lossy().to_string();

    // Write "cd /tmp" to the script
    // We use /tmp because it's guaranteed to exist
    writeln!(script_file, "cd /tmp").expect("failed to write to script");

    let mut p = spawn(&bin).expect("failed to spawn pmsh");

    p.expect(Regex("\\$ ")).expect("did not see prompt");

    // Run "source script_path"
    p.send_line(format!("source {}", script_path))
        .expect("failed to send source command");

    // Wait for prompt again
    p.expect(Regex("\\$ "))
        .expect("did not see prompt after source");

    // Check if directory changed by running "ls" and checking for something in /tmp?
    // Or better, we can't easily check pwd without a pwd builtin.
    // But we can check if "ls" output corresponds to /tmp.
    // However, /tmp content varies.
    // Let's rely on the fact that if "cd" failed, it would print an error.
    // If it succeeded, no error.
    // But we want positive confirmation.
    // Let's try to cd to a specific directory we create.

    let tmp_dir = tempfile::TempDir::new().expect("failed to create temp dir");
    let tmp_dir_path = tmp_dir.path().to_string_lossy().to_string();

    // Re-write script to cd to our temp dir
    let mut script_file = tempfile::NamedTempFile::new().expect("failed to create temp file");
    let script_path = script_file.path().to_string_lossy().to_string();
    writeln!(script_file, "cd {}", tmp_dir_path).expect("failed to write to script");

    // Create a unique file in that temp dir
    let unique_file = tmp_dir.path().join("unique_marker_file");
    std::fs::File::create(&unique_file).expect("failed to create marker file");

    // Run source again
    p.send_line(format!("source {}", script_path))
        .expect("failed to send source command");
    p.expect(Regex("\\$ "))
        .expect("did not see prompt after source 2");

    // Run ls
    p.send_line("ls").expect("failed to send ls");

    // Expect "unique_marker_file"
    p.expect(Regex("unique_marker_file"))
        .expect("did not see marker file");
}
