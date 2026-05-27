#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

mod common;

use common::*;
use std::fs;

#[test]
fn awareness_prints_empty_json_when_snapshots_match() {
    let repo = TestRepo::new();
    repo.run_git(&["init", "-b", "main"]);

    repo.write_ticket("TNDM-1", "One", "todo", "p2", &[]);
    repo.run_git(&["add", "."]);
    repo.run_git(&["commit", "-m", "base"]);

    let json = repo.run_json(&["awareness", "--against", "HEAD"]);
    assert_eq!(
        json,
        serde_json::json!({
            "schema_version": 1,
            "against": "HEAD",
            "tickets": []
        })
    );
}

#[test]
fn awareness_reports_added_current_added_against_and_diverged_sorted() {
    let repo = TestRepo::new();
    repo.run_git(&["init", "-b", "main"]);

    repo.write_ticket("TNDM-1", "Against only", "todo", "p2", &[]);
    repo.write_ticket("TNDM-3", "Diverged", "todo", "p2", &[]);
    repo.run_git(&["add", "."]);
    repo.run_git(&["commit", "-m", "base"]);

    fs::remove_dir_all(repo.path().join(".tndm").join("tickets").join("TNDM-1"))
        .expect("remove TNDM-1 from working tree");
    repo.write_ticket("TNDM-2", "Current only", "todo", "p2", &[]);
    repo.write_ticket("TNDM-3", "Diverged", "in_progress", "p2", &["TNDM-1"]);

    let json = repo.run_json(&["awareness", "--against", "HEAD"]);
    assert_eq!(
        json,
        serde_json::json!({
            "schema_version": 1,
            "against": "HEAD",
            "tickets": [
                {"id": "TNDM-1", "change": "added_against"},
                {"id": "TNDM-2", "change": "added_current"},
                {
                    "id": "TNDM-3",
                    "change": "diverged",
                    "fields": {
                        "status": {"current": "in_progress", "against": "todo"},
                        "depends_on": {"current": ["TNDM-1"], "against": []}
                    }
                }
            ]
        })
    );
}

#[test]
fn awareness_errors_for_invalid_ref() {
    let repo = TestRepo::new();
    repo.run_git(&["init", "-b", "main"]);
    repo.run_git(&["add", "."]);
    repo.run_git(&["commit", "--allow-empty", "-m", "init"]);

    let output = repo.run(&["awareness", "--against", "does-not-exist"]);

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("does-not-exist") || stderr.contains("bad revision"),
        "stderr was: {stderr}"
    );
}

#[test]
fn awareness_text_output_shows_human_readable_format() {
    let repo = TestRepo::new();
    repo.run_git(&["init", "-b", "main"]);
    repo.run_git(&["add", "."]);
    repo.run_git(&["commit", "--allow-empty", "-m", "init"]);

    repo.write_ticket("TNDM-A1", "New ticket", "in_progress", "p1", &[]);

    let stdout = repo.run_assert(&["awareness", "--against", "HEAD"]);
    assert!(
        stdout.contains("TNDM-A1"),
        "human-readable output should include the ticket ID; got: {stdout}"
    );
    assert!(
        stdout.contains("added (current)"),
        "human-readable output should include the change type; got: {stdout}"
    );
}

#[test]
fn awareness_text_output_empty_shows_no_changes() {
    let repo = TestRepo::new();
    repo.run_git(&["init", "-b", "main"]);

    repo.write_ticket("TNDM-1", "No change", "todo", "p2", &[]);
    repo.run_git(&["add", "."]);
    repo.run_git(&["commit", "-m", "init"]);

    let stdout = repo.run_assert(&["awareness", "--against", "HEAD"]);
    assert!(
        stdout.contains("No"),
        "should indicate no changes; got: {stdout}"
    );
}
