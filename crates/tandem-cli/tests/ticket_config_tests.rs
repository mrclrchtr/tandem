#![allow(clippy::disallowed_types, unused_imports)]

mod common;

use common::*;
use std::{fs, process::Command};

use regex::Regex;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

#[test]
#[allow(clippy::disallowed_methods)]
fn bare_ticket_show_uses_configured_prefix() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");
    write_prefix_config(repo_root.path(), "PROJ");

    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "create", "Show bare", "--id", "PROJ-ABC123"])
        .current_dir(repo_root.path())
        .output()
        .expect("create ticket");
    assert!(output.status.success());

    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "show", "ABC123"])
        .current_dir(repo_root.path())
        .output()
        .expect("show ticket");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert!(stdout.contains("PROJ-ABC123"), "stdout was: {stdout}");
}

#[test]
#[allow(clippy::disallowed_methods)]
fn bare_ticket_update_uses_configured_prefix() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");
    write_prefix_config(repo_root.path(), "PROJ");

    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "create", "Update bare", "--id", "PROJ-UPD123"])
        .current_dir(repo_root.path())
        .output()
        .expect("create ticket");
    assert!(output.status.success());

    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "update", "UPD123", "--status", "done"])
        .current_dir(repo_root.path())
        .output()
        .expect("update ticket");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let show = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "show", "PROJ-UPD123"])
        .current_dir(repo_root.path())
        .output()
        .expect("show ticket");
    let show_stdout = String::from_utf8(show.stdout).expect("stdout should be UTF-8");
    assert!(
        show_stdout.contains("done"),
        "show output was: {show_stdout}"
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn bare_ticket_sync_uses_configured_prefix() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");
    write_prefix_config(repo_root.path(), "PROJ");

    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "create", "Sync bare", "--id", "PROJ-SYNC01"])
        .current_dir(repo_root.path())
        .output()
        .expect("create ticket");
    assert!(output.status.success());

    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "sync", "SYNC01"])
        .current_dir(repo_root.path())
        .output()
        .expect("sync ticket");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert!(stdout.contains("PROJ-SYNC01"), "stdout was: {stdout}");
}

#[test]
#[allow(clippy::disallowed_methods)]
fn bare_ticket_doc_create_uses_configured_prefix() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");
    write_prefix_config(repo_root.path(), "PROJ");

    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "create", "Doc bare", "--id", "PROJ-DOC123"])
        .current_dir(repo_root.path())
        .output()
        .expect("create ticket");
    assert!(output.status.success());

    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "doc", "create", "DOC123", "plan"])
        .current_dir(repo_root.path())
        .output()
        .expect("create doc");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        repo_root
            .path()
            .join(".tndm")
            .join("tickets")
            .join("PROJ-DOC123")
            .join("plan.md")
            .is_file(),
        "plan.md should be created under the prefixed ticket directory"
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_doc_create_accepts_nested_ticket_relative_path() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");
    create_test_ticket(repo_root.path(), "TNDM-DOCPTH", "Nested doc path test");

    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args([
            "ticket",
            "doc",
            "create",
            "TNDM-DOCPTH",
            "task-01",
            "--path",
            "tasks/task-01.md",
        ])
        .current_dir(repo_root.path())
        .output()
        .expect("create nested doc");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        repo_root
            .path()
            .join(".tndm")
            .join("tickets")
            .join("TNDM-DOCPTH")
            .join("tasks")
            .join("task-01.md")
            .is_file(),
        "nested task doc should be created under tasks/"
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_doc_create_rejects_absolute_and_traversing_paths() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");
    create_test_ticket(repo_root.path(), "TNDM-DOCPATHERR", "Invalid doc path test");

    let absolute = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args([
            "ticket",
            "doc",
            "create",
            "TNDM-DOCPATHERR",
            "task-abs",
            "--path",
            "/tmp/task.md",
        ])
        .current_dir(repo_root.path())
        .output()
        .expect("create doc with absolute path");
    assert!(!absolute.status.success());
    let stderr_abs = String::from_utf8_lossy(&absolute.stderr);
    assert!(
        stderr_abs.contains("must be ticket-relative"),
        "stderr: {stderr_abs}"
    );

    let parent = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args([
            "ticket",
            "doc",
            "create",
            "TNDM-DOCPATHERR",
            "task-parent",
            "--path",
            "../escape.md",
        ])
        .current_dir(repo_root.path())
        .output()
        .expect("create doc with parent traversal");
    assert!(!parent.status.success());
    let stderr_parent = String::from_utf8_lossy(&parent.stderr);
    assert!(
        stderr_parent.contains("must not traverse"),
        "stderr: {stderr_parent}"
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_doc_create_rejects_existing_registered_path() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");
    create_test_ticket(repo_root.path(), "TNDM-DOCDUP", "Duplicate doc path test");

    let content_path = repo_root
        .path()
        .join(".tndm")
        .join("tickets")
        .join("TNDM-DOCDUP")
        .join("content.md");
    let original_content = fs::read_to_string(&content_path).expect("read original content");

    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args([
            "ticket",
            "doc",
            "create",
            "TNDM-DOCDUP",
            "copy",
            "--path",
            "content.md",
        ])
        .current_dir(repo_root.path())
        .output()
        .expect("create duplicate doc path");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("already registered"),
        "stderr should explain the duplicate path: {stderr}"
    );
    assert_eq!(
        fs::read_to_string(&content_path).expect("re-read content"),
        original_content,
        "content.md should remain unchanged when duplicate paths are rejected"
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn bare_ticket_create_depends_on_uses_configured_prefix() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");
    write_prefix_config(repo_root.path(), "PROJ");

    for id in ["PROJ-A1", "PROJ-A2"] {
        let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
            .args(["ticket", "create", "prereq", "--id", id])
            .current_dir(repo_root.path())
            .output()
            .expect("create prereq ticket");
        assert!(output.status.success());
    }

    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args([
            "ticket",
            "create",
            "Depends bare",
            "--id",
            "PROJ-DEP123",
            "--depends-on",
            "A1,A2",
        ])
        .current_dir(repo_root.path())
        .output()
        .expect("create ticket with bare depends_on");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let show = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "show", "PROJ-DEP123"])
        .current_dir(repo_root.path())
        .output()
        .expect("show ticket");
    let show_stdout = String::from_utf8(show.stdout).expect("stdout should be UTF-8");
    assert!(
        show_stdout.contains("PROJ-A1, PROJ-A2"),
        "show output was: {show_stdout}"
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn bare_ticket_update_depends_on_uses_configured_prefix() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");
    write_prefix_config(repo_root.path(), "PROJ");

    for id in ["PROJ-U1", "PROJ-U2", "PROJ-UPD456"] {
        let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
            .args(["ticket", "create", "prereq", "--id", id])
            .current_dir(repo_root.path())
            .output()
            .expect("create ticket");
        assert!(output.status.success());
    }

    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "update", "UPD456", "--depends-on", "U1,U2"])
        .current_dir(repo_root.path())
        .output()
        .expect("update ticket with bare depends_on");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let show = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "show", "PROJ-UPD456"])
        .current_dir(repo_root.path())
        .output()
        .expect("show ticket");
    let show_stdout = String::from_utf8(show.stdout).expect("stdout should be UTF-8");
    assert!(
        show_stdout.contains("PROJ-U1, PROJ-U2"),
        "show output was: {show_stdout}"
    );
}

// ─── ticket task integration tests ────────────────────────────

fn create_test_ticket(repo_root: &std::path::Path, id: &str, title: &str) {
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

#[test]
#[allow(clippy::disallowed_methods)]
fn doc_create_rejects_conflicting_path_for_existing_name() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");
    create_test_ticket(repo_root.path(), "TNDM-DOCPATH", "Doc path mismatch test");

    let first = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "doc", "create", "TNDM-DOCPATH", "archive"])
        .current_dir(repo_root.path())
        .output()
        .expect("create archive doc");
    assert!(
        first.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&first.stderr)
    );

    let conflicting = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args([
            "ticket",
            "doc",
            "create",
            "TNDM-DOCPATH",
            "archive",
            "--path",
            "nested/archive.md",
        ])
        .current_dir(repo_root.path())
        .output()
        .expect("create conflicting archive doc");
    assert!(
        !conflicting.status.success(),
        "conflicting --path should fail"
    );
    let stderr = String::from_utf8_lossy(&conflicting.stderr);
    assert!(
        stderr.contains("already registered") || stderr.contains("expected"),
        "stderr was: {stderr}"
    );
}
