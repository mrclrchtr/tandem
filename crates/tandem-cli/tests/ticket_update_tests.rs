#![allow(clippy::disallowed_types, unused_imports)]

mod common;

use common::*;
use std::{fs, process::Command};

use regex::Regex;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_show_prints_exact_canonical_sections() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let ticket_id = "TNDM-ABC123";
    let content = "# Details\n\nshow output body";
    let content_file = repo_root.path().join("ticket-content.md");
    fs::write(&content_file, content).expect("write content file");

    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .arg("ticket")
        .arg("create")
        .arg("Show ticket content")
        .arg("--id")
        .arg(ticket_id)
        .arg("--content-file")
        .arg(&content_file)
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm ticket create");

    assert!(
        output.status.success(),
        "expected success, stderr was: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .arg("ticket")
        .arg("show")
        .arg(ticket_id)
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm ticket show");

    assert!(
        output.status.success(),
        "expected success, stderr was: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    let updated_at_pattern = Regex::new(r#"Updated +· (.+?) \(rev"#).expect("regex should compile");
    let captures = updated_at_pattern
        .captures(&stdout)
        .expect("ticket show output should include updated_at");
    let ts_display = captures
        .get(1)
        .expect("updated_at capture should exist")
        .as_str();
    // Reconstruct RFC 3339 (T separator) for validation
    let rfc3339_ts = ts_display.replacen(' ', "T", 1).to_string();
    OffsetDateTime::parse(&rfc3339_ts, &Rfc3339).expect("updated_at should parse as RFC3339");

    let indented_content: String = content
        .lines()
        .map(|line| format!("  {line}"))
        .collect::<Vec<_>>()
        .join("\n");
    let sep = format!("  {}", "─".repeat(46));
    let expected = format!(
        concat!(
            "  {ticket_id} · Show ticket content\n",
            "{sep}\n",
            "\n",
            "  Status      · todo\n",
            "  Priority    · p2\n",
            "  Type        · task\n",
            "\n",
            "  Updated     · {ts_display} (rev 1)\n",
            "\n",
            "{sep}\n",
            "  Content\n",
            "{sep}\n",
            "{indented_content}\n"
        ),
        ticket_id = ticket_id,
        sep = sep,
        ts_display = ts_display,
        indented_content = indented_content,
    );
    assert_eq!(stdout, expected);
}

#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_show_surfaces_invalid_meta_toml_errors() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let ticket_dir = repo_root
        .path()
        .join(".tndm")
        .join("tickets")
        .join("TNDM-BROKEN");
    fs::create_dir_all(&ticket_dir).expect("create ticket dir");
    fs::write(
        ticket_dir.join("meta.toml"),
        "schema_version = 1\nid = \"TNDM-BROKEN\"\n",
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
        .arg("ticket")
        .arg("show")
        .arg("TNDM-BROKEN")
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm ticket show");

    assert!(
        !output.status.success(),
        "show should fail on invalid stored files"
    );
    let stderr = String::from_utf8(output.stderr).expect("stderr should be UTF-8");
    assert!(stderr.contains("meta.toml"), "stderr was: {stderr:?}");
    assert!(
        stderr.contains("missing field `title`") || stderr.contains("missing field 'title'"),
        "stderr was: {stderr:?}"
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_update_changes_status() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let ticket_id = "TNDM-UPST";
    Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "create", "Status test", "--id", ticket_id])
        .current_dir(repo_root.path())
        .output()
        .expect("create ticket")
        .status
        .success()
        .then_some(())
        .expect("create should succeed");

    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "update", ticket_id, "--status", "in_progress"])
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm ticket update");

    assert!(
        output.status.success(),
        "expected success, stderr was: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert_eq!(stdout.trim(), ticket_id);

    let show = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "show", ticket_id])
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm ticket show");

    let show_stdout = String::from_utf8(show.stdout).expect("stdout should be UTF-8");
    assert!(
        show_stdout.contains("in_progress"),
        "show output was: {show_stdout}"
    );
    assert!(
        show_stdout.contains("rev 2"),
        "show output was: {show_stdout}"
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_update_changes_multiple_fields() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let ticket_id = "TNDM-UPMF";
    Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "create", "Multi-field test", "--id", ticket_id])
        .current_dir(repo_root.path())
        .output()
        .expect("create ticket")
        .status
        .success()
        .then_some(())
        .expect("create should succeed");

    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args([
            "ticket",
            "update",
            ticket_id,
            "--status",
            "done",
            "--priority",
            "p0",
            "--title",
            "New title",
        ])
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm ticket update");

    assert!(
        output.status.success(),
        "expected success, stderr was: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let show = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "show", ticket_id])
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm ticket show");

    let show_stdout = String::from_utf8(show.stdout).expect("stdout should be UTF-8");
    assert!(
        show_stdout.contains("done"),
        "show output was: {show_stdout}"
    );
    assert!(show_stdout.contains("p0"), "show output was: {show_stdout}");
    assert!(
        show_stdout.contains("New title"),
        "show output was: {show_stdout}"
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_update_replaces_tags() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let ticket_id = "TNDM-UPTG";
    Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "create", "Tags test", "--id", ticket_id])
        .current_dir(repo_root.path())
        .output()
        .expect("create ticket")
        .status
        .success()
        .then_some(())
        .expect("create should succeed");

    // Set tags
    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "update", ticket_id, "--tags", "a,b"])
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm ticket update");
    assert!(output.status.success());

    let show = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "show", ticket_id])
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm ticket show");
    let show_stdout = String::from_utf8(show.stdout).expect("stdout should be UTF-8");
    assert!(
        show_stdout.contains("a, b"),
        "show output was: {show_stdout}"
    );

    // Clear tags
    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "update", ticket_id, "--tags", ""])
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm ticket update");
    assert!(output.status.success());

    let show = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "show", ticket_id])
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm ticket show");
    let show_stdout = String::from_utf8(show.stdout).expect("stdout should be UTF-8");
    assert!(
        !show_stdout.contains("a, b"),
        "show output was: {show_stdout}"
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_update_replaces_depends_on() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let ticket_id = "TNDM-UPDP";
    Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "create", "Deps test", "--id", ticket_id])
        .current_dir(repo_root.path())
        .output()
        .expect("create ticket")
        .status
        .success()
        .then_some(())
        .expect("create should succeed");

    // Set depends-on
    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args([
            "ticket",
            "update",
            ticket_id,
            "--depends-on",
            "TNDM-X,TNDM-Y",
        ])
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm ticket update");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let show = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "show", ticket_id])
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm ticket show");
    let show_stdout = String::from_utf8(show.stdout).expect("stdout should be UTF-8");
    assert!(
        show_stdout.contains("TNDM-X, TNDM-Y"),
        "show output was: {show_stdout}"
    );

    // Clear depends-on
    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "update", ticket_id, "--depends-on", ""])
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm ticket update");
    assert!(output.status.success());

    let show = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "show", ticket_id])
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm ticket show");
    let show_stdout = String::from_utf8(show.stdout).expect("stdout should be UTF-8");
    assert!(
        !show_stdout.contains("TNDM-X"),
        "show output was: {show_stdout}"
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_update_fails_with_no_flags() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let ticket_id = "TNDM-UPNF";
    Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "create", "No flags test", "--id", ticket_id])
        .current_dir(repo_root.path())
        .output()
        .expect("create ticket")
        .status
        .success()
        .then_some(())
        .expect("create should succeed");

    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "update", ticket_id])
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm ticket update");

    assert!(!output.status.success(), "update with no flags should fail");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be UTF-8");
    assert!(
        stderr.contains("at least one update flag is required"),
        "stderr was: {stderr}"
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_update_fails_for_nonexistent_ticket() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "update", "TNDM-GHOST", "--status", "done"])
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm ticket update");

    assert!(
        !output.status.success(),
        "update of nonexistent ticket should fail"
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_update_bumps_revision_and_timestamp() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let ticket_id = "TNDM-UPREV";
    Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "create", "Revision test", "--id", ticket_id])
        .current_dir(repo_root.path())
        .output()
        .expect("create ticket")
        .status
        .success()
        .then_some(())
        .expect("create should succeed");

    // First update → revision 2
    Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "update", ticket_id, "--status", "in_progress"])
        .current_dir(repo_root.path())
        .output()
        .expect("first update")
        .status
        .success()
        .then_some(())
        .expect("first update should succeed");

    // Second update → revision 3
    Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "update", ticket_id, "--status", "done"])
        .current_dir(repo_root.path())
        .output()
        .expect("second update")
        .status
        .success()
        .then_some(())
        .expect("second update should succeed");

    let show = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "show", ticket_id])
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm ticket show");

    let show_stdout = String::from_utf8(show.stdout).expect("stdout should be UTF-8");
    assert!(
        show_stdout.contains("rev 3"),
        "show output was: {show_stdout}"
    );

    let updated_at_pattern = Regex::new(r#"Updated +· (.+?) \(rev"#).expect("regex should compile");
    let captures = updated_at_pattern
        .captures(&show_stdout)
        .expect("should contain updated_at");
    let ts_display = captures.get(1).expect("capture should exist").as_str();
    let rfc3339_ts = ts_display.replacen(' ', "T", 1).to_string();
    OffsetDateTime::parse(&rfc3339_ts, &Rfc3339).expect("updated_at should parse as RFC3339");
}

#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_update_with_content_file() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let ticket_id = "TNDM-UPCF";
    Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "create", "Content file test", "--id", ticket_id])
        .current_dir(repo_root.path())
        .output()
        .expect("create ticket")
        .status
        .success()
        .then_some(())
        .expect("create should succeed");

    let content_file = repo_root.path().join("new-content.md");
    fs::write(&content_file, "# Updated Content\n\nNew body here.\n").expect("write content file");

    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args([
            "ticket",
            "update",
            ticket_id,
            "--content-file",
            content_file.to_str().unwrap(),
        ])
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm ticket update");

    assert!(
        output.status.success(),
        "expected success, stderr was: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let show = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "show", ticket_id])
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm ticket show");

    let show_stdout = String::from_utf8(show.stdout).expect("stdout should be UTF-8");
    assert!(
        show_stdout.contains("# Updated Content"),
        "show output was: {show_stdout}"
    );
    assert!(
        show_stdout.contains("New body here."),
        "show output was: {show_stdout}"
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_update_changes_type() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let ticket_id = "TNDM-UPTYP";
    Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "create", "Type test", "--id", ticket_id])
        .current_dir(repo_root.path())
        .output()
        .expect("create ticket")
        .status
        .success()
        .then_some(())
        .expect("create should succeed");

    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "update", ticket_id, "--type", "bug"])
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm ticket update");

    assert!(
        output.status.success(),
        "expected success, stderr was: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let show = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "show", ticket_id])
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm ticket show");

    let show_stdout = String::from_utf8(show.stdout).expect("stdout should be UTF-8");
    assert!(
        show_stdout.contains("bug"),
        "show output was: {show_stdout}"
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_update_rejects_empty_title() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let ticket_id = "TNDM-UPET";
    Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "create", "Empty title test", "--id", ticket_id])
        .current_dir(repo_root.path())
        .output()
        .expect("create ticket")
        .status
        .success()
        .then_some(())
        .expect("create should succeed");

    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "update", ticket_id, "--title", ""])
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm ticket update");

    assert!(
        !output.status.success(),
        "update with empty title should fail"
    );
    let stderr = String::from_utf8(output.stderr).expect("stderr should be UTF-8");
    assert!(
        stderr.contains("title must not be empty"),
        "stderr was: {stderr}"
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_update_rejects_invalid_status() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let ticket_id = "TNDM-UPIS";
    Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "create", "Invalid status test", "--id", ticket_id])
        .current_dir(repo_root.path())
        .output()
        .expect("create ticket")
        .status
        .success()
        .then_some(())
        .expect("create should succeed");

    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "update", ticket_id, "--status", "invalid"])
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm ticket update");

    assert!(!output.status.success(), "invalid status should fail");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be UTF-8");
    assert!(
        stderr.contains("invalid ticket status"),
        "stderr was: {stderr}"
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_update_rejects_invalid_priority() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let ticket_id = "TNDM-UPIP";
    Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args([
            "ticket",
            "create",
            "Invalid priority test",
            "--id",
            ticket_id,
        ])
        .current_dir(repo_root.path())
        .output()
        .expect("create ticket")
        .status
        .success()
        .then_some(())
        .expect("create should succeed");

    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "update", ticket_id, "--priority", "p9"])
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm ticket update");

    assert!(!output.status.success(), "invalid priority should fail");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be UTF-8");
    assert!(
        stderr.contains("invalid ticket priority"),
        "stderr was: {stderr}"
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_update_rejects_invalid_type() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let ticket_id = "TNDM-UPIT";
    Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "create", "Invalid type test", "--id", ticket_id])
        .current_dir(repo_root.path())
        .output()
        .expect("create ticket")
        .status
        .success()
        .then_some(())
        .expect("create should succeed");

    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "update", ticket_id, "--type", "story"])
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm ticket update");

    assert!(!output.status.success(), "invalid type should fail");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be UTF-8");
    assert!(
        stderr.contains("invalid ticket type"),
        "stderr was: {stderr}"
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_show_json_outputs_flat_ticket_with_content_path() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let ticket_id = "TNDM-SHOWJ";
    Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "create", "JSON show test", "--id", ticket_id])
        .current_dir(repo_root.path())
        .output()
        .expect("create ticket")
        .status
        .success()
        .then_some(())
        .expect("create should succeed");

    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "show", ticket_id, "--json"])
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm ticket show --json");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    let json: serde_json::Value =
        serde_json::from_str(&stdout).expect("stdout should be valid JSON");
    assert_eq!(json["schema_version"], 1);
    assert_eq!(json["id"], "TNDM-SHOWJ");
    assert_eq!(json["title"], "JSON show test");
    assert_eq!(json["type"], "task");
    assert_eq!(json["priority"], "p2");
    assert_eq!(json["status"], "todo");
    assert_eq!(json["revision"], 1);
    assert_eq!(json["content_path"], ".tndm/tickets/TNDM-SHOWJ/content.md");
    assert!(
        json.get("content").is_none(),
        "content should not be in JSON"
    );
    assert!(json["updated_at"].is_string());
}

#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_update_json_outputs_updated_ticket_envelope() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let ticket_id = "TNDM-UJ01";
    Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "create", "Update JSON test", "--id", ticket_id])
        .current_dir(repo_root.path())
        .output()
        .expect("create ticket")
        .status
        .success()
        .then_some(())
        .expect("create should succeed");

    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args([
            "ticket",
            "update",
            ticket_id,
            "--status",
            "in_progress",
            "--json",
        ])
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm ticket update --json");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    let json: serde_json::Value =
        serde_json::from_str(&stdout).expect("stdout should be valid JSON");
    assert_eq!(json["schema_version"], 1);
    assert_eq!(json["id"], "TNDM-UJ01");
    assert_eq!(json["status"], "in_progress");
    assert_eq!(json["revision"], 2);
    assert_eq!(json["content_path"], ".tndm/tickets/TNDM-UJ01/content.md");
}

#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_update_with_effort_flag() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args([
            "ticket",
            "create",
            "Effort update test",
            "--id",
            "TNDM-EF02",
        ])
        .current_dir(repo_root.path())
        .output()
        .expect("create ticket")
        .status
        .success()
        .then_some(())
        .expect("create should succeed");

    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "update", "TNDM-EF02", "--effort", "xl"])
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm ticket update with effort");

    assert!(
        output.status.success(),
        "expected success, stderr was: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let show = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "show", "TNDM-EF02"])
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm ticket show");

    let show_stdout = String::from_utf8(show.stdout).expect("stdout should be UTF-8");
    assert!(show_stdout.contains("xl"), "show output was: {show_stdout}");
}

#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_update_add_tags_preserves_existing_and_sorts() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let ticket_id = "TNDM-ADTG";
    Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args([
            "ticket",
            "create",
            "Add tags test",
            "--id",
            ticket_id,
            "--tags",
            "beta,alpha",
        ])
        .current_dir(repo_root.path())
        .output()
        .expect("create ticket")
        .status
        .success()
        .then_some(())
        .expect("create should succeed");

    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "update", ticket_id, "--add-tags", "gamma,alpha"])
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm ticket update --add-tags");

    assert!(
        output.status.success(),
        "expected success, stderr was: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let show = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "show", ticket_id])
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm ticket show");

    let show_stdout = String::from_utf8(show.stdout).expect("stdout should be UTF-8");
    assert!(
        show_stdout.contains("alpha, beta, gamma"),
        "tags should be deduped and sorted; show output was: {show_stdout}"
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_update_remove_tags_filters_existing() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let ticket_id = "TNDM-RMTG";
    Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args([
            "ticket",
            "create",
            "Remove tags test",
            "--id",
            ticket_id,
            "--tags",
            "a,b,c",
        ])
        .current_dir(repo_root.path())
        .output()
        .expect("create ticket")
        .status
        .success()
        .then_some(())
        .expect("create should succeed");

    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "update", ticket_id, "--remove-tags", "b,d"])
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm ticket update --remove-tags");

    assert!(
        output.status.success(),
        "expected success, stderr was: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let show = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "show", ticket_id])
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm ticket show");

    let show_stdout = String::from_utf8(show.stdout).expect("stdout should be UTF-8");
    assert!(
        show_stdout.contains("a, c"),
        "tags should have b removed; show output was: {show_stdout}"
    );
    assert!(
        !show_stdout.contains("b,"),
        "removed tag b should not appear; show output was: {show_stdout}"
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_update_can_replace_flow_tag_in_single_call() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let ticket_id = "TNDM-FLOWRP";
    Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args([
            "ticket",
            "create",
            "Replace flow tag test",
            "--id",
            ticket_id,
            "--tags",
            "flow:brainstorm",
        ])
        .current_dir(repo_root.path())
        .output()
        .expect("create ticket")
        .status
        .success()
        .then_some(())
        .expect("create should succeed");

    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args([
            "ticket",
            "update",
            ticket_id,
            "--remove-tags",
            "flow:brainstorm,flow:planned,flow:applying,flow:done",
            "--add-tags",
            "flow:planned",
            "--json",
        ])
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm ticket update replacing flow tag");

    assert!(
        output.status.success(),
        "expected success, stderr was: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let show_json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout should be JSON");
    assert_eq!(show_json["tags"], serde_json::json!(["flow:planned"]));
}

#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_update_add_tags_conflicts_with_tags_flag() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let ticket_id = "TNDM-CFTG";
    Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "create", "Conflict test", "--id", ticket_id])
        .current_dir(repo_root.path())
        .output()
        .expect("create ticket")
        .status
        .success()
        .then_some(())
        .expect("create should succeed");

    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args([
            "ticket",
            "update",
            ticket_id,
            "--tags",
            "x",
            "--add-tags",
            "y",
        ])
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm ticket update with conflicting flags");

    assert!(
        !output.status.success(),
        "--tags and --add-tags together should fail"
    );
    let stderr = String::from_utf8(output.stderr).expect("stderr should be UTF-8");
    assert!(
        stderr.contains("cannot be used with") || stderr.contains("error"),
        "stderr was: {stderr}"
    );
}
