#![allow(clippy::disallowed_types)]

use std::{fs, process::Command};

#[test]
#[allow(clippy::disallowed_methods)]
fn fmt_check_reports_non_canonical_structured_files() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let ticket_dir = repo_root
        .path()
        .join(".tndm")
        .join("tickets")
        .join("TNDM-ABC123");
    fs::create_dir_all(&ticket_dir).expect("create ticket dir");

    fs::write(
        ticket_dir.join("meta.toml"),
        concat!(
            "title = \"Drifted title\"\n",
            "id = \"TNDM-ABC123\"\n",
            "schema_version = 1\n",
            "priority = \"p2\"\n",
            "type = \"task\"\n",
            "tags = []\n",
            "depends_on = []\n",
        ),
    )
    .expect("write meta.toml");
    fs::write(
        ticket_dir.join("state.toml"),
        concat!(
            "revision = 1\n",
            "updated_at = \"2026-03-03T12:34:56Z\"\n",
            "status = \"todo\"\n",
            "schema_version = 1\n",
        ),
    )
    .expect("write state.toml");
    fs::write(ticket_dir.join("content.md"), "body\n").expect("write content.md");

    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .arg("fmt")
        .arg("--check")
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm fmt --check");

    assert!(!output.status.success(), "fmt --check should fail on drift");

    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert!(stdout.contains("meta.toml"), "stdout was: {stdout:?}");
    assert!(stdout.contains("state.toml"), "stdout was: {stdout:?}");
}

#[test]
#[allow(clippy::disallowed_methods)]
fn fmt_rewrites_non_canonical_structured_files() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let ticket_dir = repo_root
        .path()
        .join(".tndm")
        .join("tickets")
        .join("TNDM-ABC123");
    fs::create_dir_all(&ticket_dir).expect("create ticket dir");

    fs::write(
        ticket_dir.join("meta.toml"),
        concat!(
            "title = \"Drifted title\"\n",
            "id = \"TNDM-ABC123\"\n",
            "schema_version = 1\n",
            "priority = \"p2\"\n",
            "type = \"task\"\n",
            "tags = []\n",
            "depends_on = []\n",
        ),
    )
    .expect("write meta.toml");
    fs::write(
        ticket_dir.join("state.toml"),
        concat!(
            "revision = 1\n",
            "updated_at = \"2026-03-03T12:34:56Z\"\n",
            "status = \"todo\"\n",
            "schema_version = 1\n",
        ),
    )
    .expect("write state.toml");
    fs::write(ticket_dir.join("content.md"), "body\n").expect("write content.md");

    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .arg("fmt")
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm fmt");

    assert!(
        output.status.success(),
        "stderr was: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let meta = fs::read_to_string(ticket_dir.join("meta.toml")).expect("read meta.toml");
    let state = fs::read_to_string(ticket_dir.join("state.toml")).expect("read state.toml");

    assert_eq!(
        meta,
        concat!(
            "schema_version = 1\n",
            "id = \"TNDM-ABC123\"\n",
            "title = \"Drifted title\"\n",
            "\n",
            "type = \"task\"\n",
            "priority = \"p2\"\n",
            "\n",
            "depends_on = []\n",
            "tags = []\n",
        )
    );
    assert_eq!(
        state,
        concat!(
            "schema_version = 1\n",
            "status = \"todo\"\n",
            "updated_at = \"2026-03-03T12:34:56Z\"\n",
            "revision = 1\n",
        )
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn fmt_fails_when_managed_files_are_invalid() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let ticket_dir = repo_root
        .path()
        .join(".tndm")
        .join("tickets")
        .join("TNDM-BAD123");
    fs::create_dir_all(&ticket_dir).expect("create ticket dir");

    fs::write(
        ticket_dir.join("meta.toml"),
        "schema_version = 1\nid = \"TNDM-BAD123\"\n",
    )
    .expect("write bad meta.toml");
    fs::write(
        ticket_dir.join("state.toml"),
        concat!(
            "schema_version = 1\n",
            "status = \"todo\"\n",
            "updated_at = \"2026-03-03T12:34:56Z\"\n",
            "revision = 1\n",
        ),
    )
    .expect("write state.toml");
    fs::write(ticket_dir.join("content.md"), "body\n").expect("write content.md");

    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .arg("fmt")
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm fmt");

    assert!(
        !output.status.success(),
        "fmt should fail on invalid managed files"
    );
    let stderr = String::from_utf8(output.stderr).expect("stderr should be UTF-8");
    assert!(stderr.contains("meta.toml"), "stderr was: {stderr:?}");
}
