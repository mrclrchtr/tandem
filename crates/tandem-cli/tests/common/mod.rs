use std::{fs, path::Path, process::Command};

/// Write a `.tndm/config.toml` with the given ID prefix.
#[allow(clippy::disallowed_methods, dead_code)]
pub fn write_prefix_config(repo_root: &Path, prefix: &str) {
    fs::create_dir_all(repo_root.join(".tndm")).expect("create .tndm dir");
    fs::write(
        repo_root.join(".tndm").join("config.toml"),
        format!("schema_version = 1\n\n[id]\nprefix = \"{prefix}\"\n"),
    )
    .expect("write config.toml");
}

/// Create a test ticket using the CLI binary, asserting success.
#[allow(dead_code)]
pub fn create_test_ticket(repo_root: &Path, id: &str, title: &str) {
    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "create", title, "--id", id])
        .current_dir(repo_root)
        .output()
        .expect("create test ticket");
    assert!(
        output.status.success(),
        "create ticket failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
