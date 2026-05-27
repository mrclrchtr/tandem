#![allow(clippy::disallowed_types, unused_imports)]

mod common;

use common::*;
use std::fs;

#[test]
#[allow(clippy::disallowed_methods)]
fn bare_ticket_show_uses_configured_prefix() {
    let repo = TestRepo::with_config("PROJ");

    repo.create_ticket(Some("PROJ-ABC123"), "Show bare");

    let stdout = repo.run_assert(&["ticket", "show", "ABC123"]);
    assert!(stdout.contains("PROJ-ABC123"), "stdout was: {stdout}");
}

#[test]
#[allow(clippy::disallowed_methods)]
fn bare_ticket_update_uses_configured_prefix() {
    let repo = TestRepo::with_config("PROJ");

    repo.create_ticket(Some("PROJ-UPD123"), "Update bare");

    repo.run_assert(&["ticket", "update", "UPD123", "--status", "done"]);

    let show_stdout = repo.run_assert(&["ticket", "show", "PROJ-UPD123"]);
    assert!(
        show_stdout.contains("done"),
        "show output was: {show_stdout}"
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn bare_ticket_sync_uses_configured_prefix() {
    let repo = TestRepo::with_config("PROJ");

    repo.create_ticket(Some("PROJ-SYNC01"), "Sync bare");

    let stdout = repo.run_assert(&["ticket", "sync", "SYNC01"]);
    assert!(stdout.contains("PROJ-SYNC01"), "stdout was: {stdout}");
}

#[test]
#[allow(clippy::disallowed_methods)]
fn bare_ticket_doc_create_uses_configured_prefix() {
    let repo = TestRepo::with_config("PROJ");

    repo.create_ticket(Some("PROJ-DOC123"), "Doc bare");

    let stdout = repo.run_assert(&["ticket", "doc", "create", "DOC123", "archive"]);
    assert!(stdout.contains("PROJ-DOC123"), "stdout was: {stdout}");
    assert!(stdout.contains("archive.md"), "stdout was: {stdout}");

    let doc_path = repo
        .path()
        .join(".tndm")
        .join("tickets")
        .join("PROJ-DOC123")
        .join("archive.md");
    assert!(
        doc_path.is_file(),
        "archive.md should exist at: {}",
        doc_path.display()
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_doc_create_accepts_nested_ticket_relative_path() {
    let repo = TestRepo::new();

    repo.create_ticket(Some("TNDM-DOCNST"), "Nested doc path test");

    let stdout = repo.run_assert(&[
        "ticket",
        "doc",
        "create",
        "TNDM-DOCNST",
        "mydoc",
        "--path",
        "subdir/mydoc.md",
    ]);
    assert!(stdout.contains("TNDM-DOCNST"), "stdout was: {stdout}");
    assert!(stdout.contains("subdir"), "stdout was: {stdout}");

    let doc_path = repo
        .path()
        .join(".tndm")
        .join("tickets")
        .join("TNDM-DOCNST")
        .join("subdir")
        .join("mydoc.md");
    assert!(doc_path.is_file(), "doc should exist at nested path");
}

#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_doc_create_rejects_absolute_and_traversing_paths() {
    let repo = TestRepo::new();

    repo.create_ticket(Some("TNDM-DOCERR"), "Doc path error test");

    // Absolute path should be rejected
    let output = repo.run(&[
        "ticket",
        "doc",
        "create",
        "TNDM-DOCERR",
        "absdoc",
        "--path",
        "/etc/passwd.md",
    ]);
    assert!(!output.status.success(), "absolute path should be rejected");

    // Parent traversal should be rejected
    let output = repo.run(&[
        "ticket",
        "doc",
        "create",
        "TNDM-DOCERR",
        "traverse",
        "--path",
        "../../escape.md",
    ]);
    assert!(
        !output.status.success(),
        "parent traversal should be rejected"
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_doc_create_rejects_existing_registered_path() {
    let repo = TestRepo::new();

    repo.create_ticket(Some("TNDM-DOCREG"), "Doc conflict test");

    // First create registers the path
    repo.run_assert(&[
        "ticket",
        "doc",
        "create",
        "TNDM-DOCREG",
        "first",
        "--path",
        "shared.md",
    ]);

    // Second create with different name but same path should be rejected
    let output = repo.run(&[
        "ticket",
        "doc",
        "create",
        "TNDM-DOCREG",
        "second",
        "--path",
        "shared.md",
    ]);
    assert!(
        !output.status.success(),
        "duplicate registered path should be rejected"
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn bare_ticket_create_depends_on_uses_configured_prefix() {
    let repo = TestRepo::with_config("PROJ");

    // Create prerequisite tickets
    repo.create_ticket(Some("PROJ-X"), "dependency X");
    repo.create_ticket(Some("PROJ-Y"), "dependency Y");

    // Use bare IDs in --depends-on
    repo.run_assert(&[
        "ticket",
        "create",
        "Depends-on bare",
        "--id",
        "PROJ-DEP01",
        "--depends-on",
        "X,Y",
    ]);

    let json = repo.run_json(&["ticket", "show", "PROJ-DEP01"]);
    assert_eq!(json["depends_on"], serde_json::json!(["PROJ-X", "PROJ-Y"]));
}

#[test]
#[allow(clippy::disallowed_methods)]
fn bare_ticket_update_depends_on_uses_configured_prefix() {
    let repo = TestRepo::with_config("PROJ");

    // Create real prerequisite tickets
    repo.create_ticket(Some("PROJ-X"), "dependency X");
    repo.create_ticket(Some("PROJ-Y"), "dependency Y");
    repo.create_ticket(Some("PROJ-UPDEP"), "Update deps bare");

    // Use bare ID in --depends-on
    repo.run_assert(&["ticket", "update", "UPDEP", "--depends-on", "PROJ-X,PROJ-Y"]);

    let json = repo.run_json(&["ticket", "show", "PROJ-UPDEP"]);
    assert_eq!(json["depends_on"], serde_json::json!(["PROJ-X", "PROJ-Y"]));
}

#[test]
#[allow(clippy::disallowed_methods)]
fn doc_create_rejects_conflicting_path_for_existing_name() {
    let repo = TestRepo::new();

    let ticket_id = "TNDM-DOCCONF";
    repo.create_ticket(Some(ticket_id), "Doc conflict test");

    // Create doc with default path
    repo.run_assert(&["ticket", "doc", "create", ticket_id, "mydoc"]);

    // Try to create same doc name with different path
    let output = repo.run(&[
        "ticket", "doc", "create", ticket_id, "mydoc", "--path", "other.md",
    ]);
    assert!(
        !output.status.success(),
        "conflicting path should be rejected"
    );
    let stderr = String::from_utf8(output.stderr).expect("stderr should be UTF-8");
    assert!(
        stderr.contains("already registered"),
        "error should mention existing registration; stderr was: {stderr}"
    );
}
