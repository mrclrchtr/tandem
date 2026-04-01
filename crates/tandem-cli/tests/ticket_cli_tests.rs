#![allow(clippy::disallowed_types)]

use std::{fs, process::Command};

use regex::Regex;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_create_prints_generated_id_and_writes_ticket_files() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .arg("ticket")
        .arg("create")
        .arg("Ship ticket create")
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm ticket create");

    assert!(
        output.status.success(),
        "expected success, stderr was: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    let id_pattern = Regex::new(r"^TNDM-[0-9A-Z]{6}\n$").expect("regex should compile");
    assert!(
        id_pattern.is_match(&stdout),
        "expected generated ID, got stdout: {stdout:?}"
    );

    let id = stdout.trim();
    let ticket_dir = repo_root.path().join(".tndm").join("tickets").join(id);

    assert!(repo_root.path().join(".tndm").is_dir());
    assert!(repo_root.path().join(".tndm").join("tickets").is_dir());
    assert!(ticket_dir.is_dir());
    assert!(ticket_dir.join("meta.toml").is_file());
    assert!(ticket_dir.join("state.toml").is_file());
    assert!(ticket_dir.join("content.md").is_file());
}

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
    let updated_at_pattern =
        Regex::new(r#"updated_at = \"([^\"]+)\""#).expect("regex should compile");
    let captures = updated_at_pattern
        .captures(&stdout)
        .expect("ticket show output should include updated_at");
    let updated_at = captures
        .get(1)
        .expect("updated_at capture should exist")
        .as_str();
    OffsetDateTime::parse(updated_at, &Rfc3339).expect("updated_at should parse as RFC3339");

    let expected = format!(
        concat!(
            "## meta.toml\n",
            "schema_version = 1\n",
            "id = \"{ticket_id}\"\n",
            "title = \"Show ticket content\"\n",
            "\n",
            "type = \"task\"\n",
            "priority = \"p2\"\n",
            "\n",
            "depends_on = []\n",
            "tags = []\n",
            "\n",
            "## state.toml\n",
            "schema_version = 1\n",
            "status = \"todo\"\n",
            "updated_at = \"{updated_at}\"\n",
            "revision = 1\n",
            "\n",
            "## content.md\n",
            "{content}"
        ),
        ticket_id = ticket_id,
        updated_at = updated_at,
        content = content,
    );
    assert_eq!(stdout, expected);
}

#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_list_prints_sorted_tab_separated_lines() {
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
    assert_eq!(
        stdout,
        "TNDM-1\ttodo\tp2\t-\t\tFirst ticket\nTNDM-2\ttodo\tp2\t-\t\tSecond ticket\n"
    );
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
        show_stdout.contains("status = \"in_progress\""),
        "show output was: {show_stdout}"
    );
    assert!(
        show_stdout.contains("revision = 2"),
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
        show_stdout.contains("status = \"done\""),
        "show output was: {show_stdout}"
    );
    assert!(
        show_stdout.contains("priority = \"p0\""),
        "show output was: {show_stdout}"
    );
    assert!(
        show_stdout.contains("title = \"New title\""),
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
        show_stdout.contains("tags = [\"a\", \"b\"]"),
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
        show_stdout.contains("tags = []"),
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
        show_stdout.contains("depends_on = [\"TNDM-X\", \"TNDM-Y\"]"),
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
        show_stdout.contains("depends_on = []"),
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
        show_stdout.contains("revision = 3"),
        "show output was: {show_stdout}"
    );

    let updated_at_pattern =
        Regex::new(r#"updated_at = \"([^\"]+)\""#).expect("regex should compile");
    let captures = updated_at_pattern
        .captures(&show_stdout)
        .expect("should contain updated_at");
    let updated_at = captures.get(1).expect("capture should exist").as_str();
    OffsetDateTime::parse(updated_at, &Rfc3339).expect("updated_at should parse as RFC3339");
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
        show_stdout.contains("type = \"bug\""),
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
fn ticket_create_json_outputs_full_ticket_envelope() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args([
            "ticket",
            "create",
            "JSON create test",
            "--id",
            "TNDM-CJ01",
            "--json",
        ])
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm ticket create --json");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    let json: serde_json::Value =
        serde_json::from_str(&stdout).expect("stdout should be valid JSON");
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
        .map(|l| l.split('\t').next().unwrap())
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
fn ticket_create_with_all_metadata_flags() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    // Create prerequisite tickets for depends_on
    for id in ["TNDM-A1", "TNDM-A2"] {
        Command::new(env!("CARGO_BIN_EXE_tndm"))
            .args(["ticket", "create", "prereq", "--id", id])
            .current_dir(repo_root.path())
            .output()
            .expect("create prereq ticket")
            .status
            .success()
            .then_some(())
            .expect("create should succeed");
    }

    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args([
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
        ])
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm ticket create with all flags");

    assert!(
        output.status.success(),
        "expected success, stderr was: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let show = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "show", "TNDM-FL01"])
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm ticket show");

    let show_stdout = String::from_utf8(show.stdout).expect("stdout should be UTF-8");
    assert!(
        show_stdout.contains("priority = \"p0\""),
        "show output was: {show_stdout}"
    );
    assert!(
        show_stdout.contains("type = \"bug\""),
        "show output was: {show_stdout}"
    );
    assert!(
        show_stdout.contains("tags = [\"auth\", \"security\"]"),
        "show output was: {show_stdout}"
    );
    assert!(
        show_stdout.contains("depends_on = [\"TNDM-A1\", \"TNDM-A2\"]"),
        "show output was: {show_stdout}"
    );
    assert!(
        show_stdout.contains("status = \"in_progress\""),
        "show output was: {show_stdout}"
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_create_with_priority_flag() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args([
            "ticket",
            "create",
            "Priority test",
            "--id",
            "TNDM-PR01",
            "--priority",
            "p1",
        ])
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm ticket create");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let show = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "show", "TNDM-PR01"])
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm ticket show");

    let show_stdout = String::from_utf8(show.stdout).expect("stdout should be UTF-8");
    assert!(
        show_stdout.contains("priority = \"p1\""),
        "show output was: {show_stdout}"
    );
    assert!(
        show_stdout.contains("type = \"task\""),
        "show output was: {show_stdout}"
    );
    assert!(
        show_stdout.contains("status = \"todo\""),
        "show output was: {show_stdout}"
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_create_rejects_invalid_priority() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args([
            "ticket",
            "create",
            "Bad priority",
            "--id",
            "TNDM-BP01",
            "--priority",
            "p9",
        ])
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm ticket create");

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
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args([
            "ticket",
            "create",
            "Bad depends",
            "--id",
            "TNDM-BD01",
            "--depends-on",
            "not a valid id",
        ])
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm ticket create");

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
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args([
            "ticket",
            "create",
            "Effort create test",
            "--id",
            "TNDM-EF01",
            "--effort",
            "m",
        ])
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm ticket create with effort");

    assert!(
        output.status.success(),
        "expected success, stderr was: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let show = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "show", "TNDM-EF01"])
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm ticket show");

    let show_stdout = String::from_utf8(show.stdout).expect("stdout should be UTF-8");
    assert!(
        show_stdout.contains("effort = \"m\""),
        "show output was: {show_stdout}"
    );
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
    assert!(
        show_stdout.contains("effort = \"xl\""),
        "show output was: {show_stdout}"
    );
}
