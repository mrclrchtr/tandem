#![allow(clippy::disallowed_types)]

mod common;

use std::fs;

use common::*;

#[test]
#[allow(clippy::disallowed_methods)]
fn sync_single_ticket() {
    let repo = TestRepo::new();

    repo.create_ticket(Some("TNDM-1"), "Test ticket");

    let stdout = repo.run_assert(&["ticket", "sync", "TNDM-1"]);
    assert!(
        stdout.contains("TNDM-1"),
        "expected synced ticket ID: {stdout}"
    );
    assert_eq!(
        stdout.lines().count(),
        1,
        "expected single line output: {stdout}"
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn sync_single_ticket_json() {
    let repo = TestRepo::new();

    repo.create_ticket(Some("TNDM-1"), "Test ticket");

    let value = repo.run_json(&["ticket", "sync", "TNDM-1"]);
    assert_eq!(
        value["id"], "TNDM-1",
        "expected TNDM-1 id in JSON output: {value}"
    );
    assert_eq!(
        value["title"], "Test ticket",
        "expected title in JSON output: {value}"
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn sync_all_two_tickets() {
    let repo = TestRepo::new();

    repo.create_ticket(Some("TNDM-1"), "First ticket");
    repo.create_ticket(Some("TNDM-2"), "Second ticket");

    let output = repo.run(&["ticket", "sync", "--all"]);
    assert!(
        output.status.success(),
        "sync --all failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert!(
        stdout.contains("TNDM-1"),
        "expected TNDM-1 in output: {stdout}"
    );
    assert!(
        stdout.contains("TNDM-2"),
        "expected TNDM-2 in output: {stdout}"
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn sync_all_includes_done_tickets() {
    let repo = TestRepo::new();

    repo.create_ticket(Some("TNDM-1"), "Open ticket");
    repo.create_ticket(Some("TNDM-2"), "Done ticket");

    // Mark TNDM-2 as done
    repo.run_assert(&["ticket", "update", "TNDM-2", "--status", "done"]);

    let stdout = repo.run_assert(&["ticket", "sync", "--all"]);

    assert!(
        stdout.contains("TNDM-2"),
        "expected done ticket TNDM-2 to be included in --all sync: {stdout}"
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn sync_all_mutually_exclusive_with_id() {
    let repo = TestRepo::new();

    repo.create_ticket(Some("TNDM-1"), "Test ticket");

    let output = repo.run(&["ticket", "sync", "TNDM-1", "--all"]);
    assert!(
        !output.status.success(),
        "expected error when providing both id and --all"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("cannot provide both a ticket ID and --all")
            || stderr.contains("the argument '--all' cannot be used")
            || stderr.contains("TNDM-1"),
        "unexpected error message: {stderr}"
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn sync_requires_id_or_all() {
    let repo = TestRepo::new();

    let output = repo.run(&["ticket", "sync"]);
    assert!(
        !output.status.success(),
        "expected error when providing neither id nor --all"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);

    // The error may be from clap (arg_required_else_help) or from our handler
    let acceptable_msgs = [
        "provide a ticket ID or use --all",
        "required",
        "the following required arguments were not provided",
    ];

    let matched = acceptable_msgs.iter().any(|msg| stderr.contains(msg));
    assert!(matched, "unexpected error message: {stderr}");
}

#[test]
#[allow(clippy::disallowed_methods)]
fn sync_all_json_output() {
    let repo = TestRepo::new();

    repo.create_ticket(Some("TNDM-1"), "First");
    repo.create_ticket(Some("TNDM-2"), "Second");

    let stdout = repo.run_assert(&["ticket", "sync", "--all", "--json"]);
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("expected valid JSON output");

    assert_eq!(
        parsed["schema_version"], 1,
        "expected schema_version: {parsed}"
    );
    assert_eq!(parsed["synced"], 2, "expected 2 synced: {parsed}");
    assert_eq!(parsed["failed"], 0, "expected 0 failed: {parsed}");
    assert_eq!(parsed["total"], 2, "expected 2 total: {parsed}");

    let results = parsed["results"]
        .as_array()
        .expect("results should be an array");
    assert_eq!(results.len(), 2);

    let ids: Vec<&str> = results
        .iter()
        .map(|r| r["id"].as_str().expect("id should be a string"))
        .collect();
    assert!(ids.contains(&"TNDM-1"), "missing TNDM-1: {ids:?}");
    assert!(ids.contains(&"TNDM-2"), "missing TNDM-2: {ids:?}");

    for result in results {
        assert_eq!(result["status"], "ok", "expected all ok: {result}");
    }
}

#[test]
#[allow(clippy::disallowed_methods)]
fn sync_all_continues_after_per_ticket_error() {
    let repo = TestRepo::new();

    repo.create_ticket(Some("TNDM-1"), "Good ticket");
    repo.create_ticket(Some("TNDM-2"), "Bad ticket");

    // Corrupt TNDM-2's state.toml so load_ticket fails.
    let state_path = repo
        .path()
        .join(".tndm")
        .join("tickets")
        .join("TNDM-2")
        .join("state.toml");
    fs::write(&state_path, "this is not valid toml [[").expect("write corrupt state.toml");

    let output = repo.run(&["ticket", "sync", "--all"]);
    assert!(
        !output.status.success(),
        "expected non-zero exit when one ticket fails"
    );

    // TNDM-1 should still have been synced.
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert!(
        stdout.contains("TNDM-1"),
        "expected TNDM-1 to succeed before TNDM-2 failure: {stdout}"
    );

    // stderr should show the failure.
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("1 ticket(s) failed to sync"),
        "expected failure summary on stderr: {stderr}"
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn sync_all_error_recovery_json() {
    let repo = TestRepo::new();

    repo.create_ticket(Some("TNDM-1"), "Good ticket");
    repo.create_ticket(Some("TNDM-2"), "Bad ticket");

    // Corrupt TNDM-2's state.toml.
    let state_path = repo
        .path()
        .join(".tndm")
        .join("tickets")
        .join("TNDM-2")
        .join("state.toml");
    fs::write(&state_path, "this is not valid toml [[").expect("write corrupt state.toml");

    let output = repo.run(&["ticket", "sync", "--all", "--json"]);
    assert!(
        !output.status.success(),
        "expected non-zero exit when one ticket fails"
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("expected valid JSON output");

    assert_eq!(parsed["synced"], 1, "expected 1 synced: {parsed}");
    assert_eq!(parsed["failed"], 1, "expected 1 failed: {parsed}");
    assert_eq!(parsed["total"], 2, "expected 2 total: {parsed}");

    let results = parsed["results"]
        .as_array()
        .expect("results should be an array");
    assert_eq!(results.len(), 2);

    let ok_entry = results
        .iter()
        .find(|r| r["status"] == "ok")
        .expect("expected one ok entry");
    assert_eq!(ok_entry["id"], "TNDM-1");

    let err_entry = results
        .iter()
        .find(|r| r["status"] == "error")
        .expect("expected one error entry");
    assert_eq!(err_entry["id"], "TNDM-2");
    assert!(
        err_entry["message"].as_str().is_some_and(|s| !s.is_empty()),
        "expected error message: {err_entry}"
    );
}
