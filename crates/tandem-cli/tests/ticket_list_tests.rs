#![allow(clippy::disallowed_types, unused_imports)]

mod common;

use common::*;
use std::{fs, process::Command};

use regex::Regex;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_list_prints_sorted_rows() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .arg("ticket")
        .arg("create")
        .arg("Second ticket")
        .arg("--id")
        .arg("TNDM-2")
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm ticket create for TNDM-2");

    assert!(
        output.status.success(),
        "expected success, stderr was: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .arg("ticket")
        .arg("create")
        .arg("First ticket")
        .arg("--id")
        .arg("TNDM-1")
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm ticket create for TNDM-1");

    assert!(
        output.status.success(),
        "expected success, stderr was: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .arg("ticket")
        .arg("list")
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm ticket list");

    assert!(
        output.status.success(),
        "expected success, stderr was: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3, "unexpected list output: {stdout}");
    assert!(lines[0].contains("ID"));
    assert!(lines[0].contains("STATUS"));
    assert!(lines[0].contains("TITLE"));
    assert!(lines[1].contains("TNDM-1"));
    assert!(lines[1].contains("First ticket"));
    assert!(lines[2].contains("TNDM-2"));
    assert!(lines[2].contains("Second ticket"));
}

#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_list_hides_done_tickets_by_default() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    // Create two tickets
    for (id, title) in [("TNDM-1", "Open ticket"), ("TNDM-2", "Done ticket")] {
        let out = Command::new(env!("CARGO_BIN_EXE_tndm"))
            .args(["ticket", "create", title, "--id", id])
            .current_dir(repo_root.path())
            .output()
            .expect("create ticket");
        assert!(out.status.success());
    }

    // Mark TNDM-2 as done
    let out = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "update", "TNDM-2", "--status", "done"])
        .current_dir(repo_root.path())
        .output()
        .expect("update ticket");
    assert!(out.status.success());

    // Default list should hide done tickets
    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "list"])
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm ticket list");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert!(stdout.contains("TNDM-1"), "open ticket should appear");
    assert!(!stdout.contains("TNDM-2"), "done ticket should be hidden");

    // --all should include done tickets
    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "list", "--all"])
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm ticket list --all");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert!(
        stdout.contains("TNDM-1"),
        "open ticket should appear with --all"
    );
    assert!(
        stdout.contains("TNDM-2"),
        "done ticket should appear with --all"
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_list_json_hides_done_tickets_by_default() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    for (id, title) in [("TNDM-1", "Open"), ("TNDM-2", "Done")] {
        let out = Command::new(env!("CARGO_BIN_EXE_tndm"))
            .args(["ticket", "create", title, "--id", id])
            .current_dir(repo_root.path())
            .output()
            .expect("create ticket");
        assert!(out.status.success());
    }

    let out = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "update", "TNDM-2", "--status", "done"])
        .current_dir(repo_root.path())
        .output()
        .expect("update ticket");
    assert!(out.status.success());

    // Default JSON list should hide done tickets
    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "list", "--json"])
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm ticket list --json");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    let tickets = json["tickets"].as_array().expect("tickets array");
    assert_eq!(tickets.len(), 1);
    assert_eq!(tickets[0]["id"], "TNDM-1");

    // --all --json should include done tickets
    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "list", "--all", "--json"])
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm ticket list --all --json");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    let tickets = json["tickets"].as_array().expect("tickets array");
    assert_eq!(tickets.len(), 2);
}

#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_list_json_outputs_schema_versioned_array() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "create", "First", "--id", "TNDM-1"])
        .current_dir(repo_root.path())
        .output()
        .expect("create ticket 1")
        .status
        .success()
        .then_some(())
        .expect("create should succeed");

    Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "create", "Second", "--id", "TNDM-2"])
        .current_dir(repo_root.path())
        .output()
        .expect("create ticket 2")
        .status
        .success()
        .then_some(())
        .expect("create should succeed");

    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "list", "--json"])
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm ticket list --json");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    let json: serde_json::Value =
        serde_json::from_str(&stdout).expect("stdout should be valid JSON");
    assert_eq!(json["schema_version"], 1);
    let tickets = json["tickets"]
        .as_array()
        .expect("tickets should be an array");
    assert_eq!(tickets.len(), 2);
    assert_eq!(tickets[0]["id"], "TNDM-1");
    assert_eq!(tickets[0]["title"], "First");
    assert_eq!(
        tickets[0]["content_path"],
        ".tndm/tickets/TNDM-1/content.md"
    );
    assert!(
        tickets[0].get("schema_version").is_none(),
        "individual tickets should not have schema_version"
    );
    assert_eq!(tickets[1]["id"], "TNDM-2");
    assert_eq!(tickets[1]["title"], "Second");
}

#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_list_sorts_by_priority_then_id() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    // Create tickets and set different priorities
    for (id, title, prio) in [
        ("TNDM-1", "Low prio", "p3"),
        ("TNDM-2", "High prio", "p0"),
        ("TNDM-3", "Also high prio", "p0"),
        ("TNDM-4", "Medium prio", "p2"),
    ] {
        let out = Command::new(env!("CARGO_BIN_EXE_tndm"))
            .args(["ticket", "create", title, "--id", id])
            .current_dir(repo_root.path())
            .output()
            .expect("create ticket");
        assert!(out.status.success());

        let out = Command::new(env!("CARGO_BIN_EXE_tndm"))
            .args(["ticket", "update", id, "--priority", prio])
            .current_dir(repo_root.path())
            .output()
            .expect("update priority");
        assert!(out.status.success());
    }

    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "list"])
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm ticket list");
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    let ids: Vec<&str> = stdout
        .lines()
        .skip(1)
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            line.split_whitespace()
                .next()
                .expect("ticket row should start with id")
        })
        .collect();
    assert_eq!(ids, vec!["TNDM-2", "TNDM-3", "TNDM-4", "TNDM-1"]);
}

#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_list_json_empty_produces_empty_array() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "list", "--json"])
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm ticket list --json");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    let json: serde_json::Value =
        serde_json::from_str(&stdout).expect("stdout should be valid JSON");
    assert_eq!(json["schema_version"], 1);
    assert_eq!(json["tickets"], serde_json::json!([]));
}

#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_list_filters_by_definition_tags_in_plain_text() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    for (id, title, tags) in [
        ("TNDM-READY", "Ready ticket", "definition:ready"),
        ("TNDM-QUES", "Questions ticket", "definition:questions"),
        ("TNDM-UNKW", "Unknown ticket", ""),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
            .args(["ticket", "create", title, "--id", id, "--tags", tags])
            .current_dir(repo_root.path())
            .output()
            .expect("create ticket");
        assert!(output.status.success());
    }

    let ready = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "list", "--definition", "ready"])
        .current_dir(repo_root.path())
        .output()
        .expect("run ready filter");
    assert!(ready.status.success());
    let ready_stdout = String::from_utf8(ready.stdout).expect("stdout should be UTF-8");
    assert!(ready_stdout.contains("TNDM-READY"));
    assert!(!ready_stdout.contains("TNDM-QUES"));
    assert!(!ready_stdout.contains("TNDM-UNKW"));

    let questions = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "list", "--definition", "questions"])
        .current_dir(repo_root.path())
        .output()
        .expect("run questions filter");
    assert!(questions.status.success());
    let questions_stdout = String::from_utf8(questions.stdout).expect("stdout should be UTF-8");
    assert!(questions_stdout.contains("TNDM-QUES"));
    assert!(!questions_stdout.contains("TNDM-READY"));
    assert!(!questions_stdout.contains("TNDM-UNKW"));

    let unknown = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "list", "--definition", "unknown"])
        .current_dir(repo_root.path())
        .output()
        .expect("run unknown filter");
    assert!(unknown.status.success());
    let unknown_stdout = String::from_utf8(unknown.stdout).expect("stdout should be UTF-8");
    assert!(unknown_stdout.contains("TNDM-UNKW"));
    assert!(!unknown_stdout.contains("TNDM-READY"));
    assert!(!unknown_stdout.contains("TNDM-QUES"));
}

#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_list_filters_by_definition_tags_in_json() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    for (id, title, tags) in [
        ("TNDM-READY", "Ready ticket", "definition:ready"),
        ("TNDM-QUES", "Questions ticket", "definition:questions"),
        ("TNDM-UNKW", "Unknown ticket", ""),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
            .args(["ticket", "create", title, "--id", id, "--tags", tags])
            .current_dir(repo_root.path())
            .output()
            .expect("create ticket");
        assert!(output.status.success());
    }

    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "list", "--definition", "questions", "--json"])
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm ticket list with json definition filter");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    let json: serde_json::Value =
        serde_json::from_str(&stdout).expect("stdout should be valid JSON");
    let tickets = json["tickets"]
        .as_array()
        .expect("tickets should be an array");
    assert_eq!(tickets.len(), 1);
    assert_eq!(tickets[0]["id"], "TNDM-QUES");
    assert_eq!(
        tickets[0]["tags"],
        serde_json::json!(["definition:questions"])
    );
}
