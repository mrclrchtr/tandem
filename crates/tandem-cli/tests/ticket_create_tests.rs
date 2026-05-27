#![allow(clippy::disallowed_types, unused_imports)]

mod common;

use common::*;
use std::fs;

use regex::Regex;

#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_create_prints_generated_id_and_writes_ticket_files() {
    let repo = TestRepo::new();

    let stdout = repo.run_assert(&["ticket", "create", "Ship ticket create"]);
    let id_pattern = Regex::new(r"^TNDM-[0-9A-Z]{6}\n$").expect("regex should compile");
    assert!(
        id_pattern.is_match(&stdout),
        "expected generated ID, got stdout: {stdout:?}"
    );

    let id = stdout.trim();
    let ticket_dir = repo.path().join(".tndm").join("tickets").join(id);

    assert!(repo.path().join(".tndm").is_dir());
    assert!(repo.path().join(".tndm").join("tickets").is_dir());
    assert!(ticket_dir.is_dir());
    assert!(ticket_dir.join("meta.toml").is_file());
    assert!(ticket_dir.join("state.toml").is_file());
    assert!(ticket_dir.join("content.md").is_file());
}

#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_create_json_outputs_full_ticket_envelope() {
    let repo = TestRepo::new();

    let json = repo.run_json(&["ticket", "create", "JSON create test", "--id", "TNDM-CJ01"]);
    assert_eq!(json["schema_version"], 1);
    assert_eq!(json["id"], "TNDM-CJ01");
    assert_eq!(json["title"], "JSON create test");
    assert_eq!(json["type"], "task");
    assert_eq!(json["priority"], "p2");
    assert_eq!(json["status"], "todo");
    assert_eq!(json["revision"], 1);
    assert_eq!(json["content_path"], ".tndm/tickets/TNDM-CJ01/content.md");
    assert!(json.get("content").is_none());
}

#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_create_uses_definition_friendly_default_template() {
    let repo = TestRepo::new();

    repo.run_assert(&["ticket", "create", "Template test", "--id", "TNDM-TMPL"]);

    let content = fs::read_to_string(
        repo.path()
            .join(".tndm")
            .join("tickets")
            .join("TNDM-TMPL")
            .join("content.md"),
    )
    .expect("read content.md");

    assert_eq!(content, tandem_core::ticket::DEFAULT_CONTENT_TEMPLATE);
}

#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_create_with_all_metadata_flags() {
    let repo = TestRepo::new();

    // Create prerequisite tickets for depends_on
    for id in ["TNDM-A1", "TNDM-A2"] {
        repo.create_ticket(Some(id), "prereq");
    }

    repo.run_assert(&[
        "ticket",
        "create",
        "Full flags test",
        "--id",
        "TNDM-FL01",
        "--priority",
        "p0",
        "--type",
        "bug",
        "--tags",
        "auth,security",
        "--depends-on",
        "TNDM-A1,TNDM-A2",
        "--status",
        "in_progress",
    ]);

    let show_stdout = repo.run_assert(&["ticket", "show", "TNDM-FL01"]);
    assert!(show_stdout.contains("p0"), "show output was: {show_stdout}");
    assert!(
        show_stdout.contains("bug"),
        "show output was: {show_stdout}"
    );
    assert!(
        show_stdout.contains("auth, security"),
        "show output was: {show_stdout}"
    );
    assert!(
        show_stdout.contains("TNDM-A1, TNDM-A2"),
        "show output was: {show_stdout}"
    );
    assert!(
        show_stdout.contains("in_progress"),
        "show output was: {show_stdout}"
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_create_with_priority_flag() {
    let repo = TestRepo::new();

    repo.run_assert(&[
        "ticket",
        "create",
        "Priority test",
        "--id",
        "TNDM-PR01",
        "--priority",
        "p1",
    ]);

    let show_stdout = repo.run_assert(&["ticket", "show", "TNDM-PR01"]);
    assert!(show_stdout.contains("p1"), "show output was: {show_stdout}");
    assert!(
        show_stdout.contains("task"),
        "show output was: {show_stdout}"
    );
    assert!(
        show_stdout.contains("todo"),
        "show output was: {show_stdout}"
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_create_rejects_invalid_priority() {
    let repo = TestRepo::new();

    let output = repo.run(&[
        "ticket",
        "create",
        "Bad priority",
        "--id",
        "TNDM-BP01",
        "--priority",
        "p9",
    ]);

    assert!(!output.status.success(), "invalid priority should fail");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be UTF-8");
    assert!(
        stderr.contains("invalid ticket priority"),
        "stderr was: {stderr}"
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_create_rejects_invalid_depends_on() {
    let repo = TestRepo::new();

    let output = repo.run(&[
        "ticket",
        "create",
        "Bad depends",
        "--id",
        "TNDM-BD01",
        "--depends-on",
        "not a valid id",
    ]);

    assert!(!output.status.success(), "invalid depends_on should fail");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be UTF-8");
    assert!(
        stderr.contains("ticket id must not contain whitespace"),
        "stderr was: {stderr}"
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_create_with_effort_flag() {
    let repo = TestRepo::new();

    repo.run_assert(&[
        "ticket",
        "create",
        "Effort create test",
        "--id",
        "TNDM-EF01",
        "--effort",
        "m",
    ]);

    let show_stdout = repo.run_assert(&["ticket", "show", "TNDM-EF01"]);
    assert!(
        show_stdout.contains("Effort"),
        "show output was: {show_stdout}"
    );
}
