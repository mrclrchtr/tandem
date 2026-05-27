#![allow(clippy::disallowed_types, unused_imports)]

mod common;

use common::*;
use std::fs;

use regex::Regex;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_show_prints_exact_canonical_sections() {
    let repo = TestRepo::new();

    let ticket_id = "TNDM-ABC123";
    let content = "# Details\n\nshow output body";
    let content_file = repo.path().join("ticket-content.md");
    fs::write(&content_file, content).expect("write content file");

    repo.run_assert(&[
        "ticket",
        "create",
        "Show ticket content",
        "--id",
        ticket_id,
        "--content-file",
        content_file.to_str().unwrap(),
    ]);

    let stdout = repo.run_assert(&["ticket", "show", ticket_id]);
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
    let repo = TestRepo::new();

    let ticket_dir = repo
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

    let output = repo.run(&["ticket", "show", "TNDM-BROKEN"]);

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
    let repo = TestRepo::new();

    let ticket_id = "TNDM-UPST";
    repo.create_ticket(Some(ticket_id), "Status test");

    repo.run_assert(&["ticket", "update", ticket_id, "--status", "in_progress"]);

    let show_stdout = repo.run_assert(&["ticket", "show", ticket_id]);
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
    let repo = TestRepo::new();

    let ticket_id = "TNDM-UPMF";
    repo.create_ticket(Some(ticket_id), "Multi-field test");

    repo.run_assert(&[
        "ticket",
        "update",
        ticket_id,
        "--status",
        "done",
        "--priority",
        "p0",
        "--title",
        "New title",
    ]);

    let show_stdout = repo.run_assert(&["ticket", "show", ticket_id]);
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
    let repo = TestRepo::new();

    let ticket_id = "TNDM-UPTG";
    repo.create_ticket(Some(ticket_id), "Tags test");

    // Set tags
    repo.run_assert(&["ticket", "update", ticket_id, "--tags", "a,b"]);

    let show_stdout = repo.run_assert(&["ticket", "show", ticket_id]);
    assert!(
        show_stdout.contains("a, b"),
        "show output was: {show_stdout}"
    );

    // Clear tags
    repo.run_assert(&["ticket", "update", ticket_id, "--tags", ""]);

    let show_stdout = repo.run_assert(&["ticket", "show", ticket_id]);
    assert!(
        !show_stdout.contains("a, b"),
        "show output was: {show_stdout}"
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_update_replaces_depends_on() {
    let repo = TestRepo::new();

    let ticket_id = "TNDM-UPDP";
    repo.create_ticket(Some(ticket_id), "Deps test");

    // Set depends-on
    repo.run_assert(&[
        "ticket",
        "update",
        ticket_id,
        "--depends-on",
        "TNDM-X,TNDM-Y",
    ]);

    let show_stdout = repo.run_assert(&["ticket", "show", ticket_id]);
    assert!(
        show_stdout.contains("TNDM-X, TNDM-Y"),
        "show output was: {show_stdout}"
    );

    // Clear depends-on
    repo.run_assert(&["ticket", "update", ticket_id, "--depends-on", ""]);

    let show_stdout = repo.run_assert(&["ticket", "show", ticket_id]);
    assert!(
        !show_stdout.contains("TNDM-X"),
        "show output was: {show_stdout}"
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_update_fails_with_no_flags() {
    let repo = TestRepo::new();

    let ticket_id = "TNDM-UPNF";
    repo.create_ticket(Some(ticket_id), "No flags test");

    let output = repo.run(&["ticket", "update", ticket_id]);

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
    let repo = TestRepo::new();

    let output = repo.run(&["ticket", "update", "TNDM-GHOST", "--status", "done"]);

    assert!(
        !output.status.success(),
        "update of nonexistent ticket should fail"
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_update_bumps_revision_and_timestamp() {
    let repo = TestRepo::new();

    let ticket_id = "TNDM-UPREV";
    repo.create_ticket(Some(ticket_id), "Revision test");

    // First update → revision 2
    repo.run_assert(&["ticket", "update", ticket_id, "--status", "in_progress"]);

    // Second update → revision 3
    repo.run_assert(&["ticket", "update", ticket_id, "--status", "done"]);

    let show_stdout = repo.run_assert(&["ticket", "show", ticket_id]);
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
    let repo = TestRepo::new();

    let ticket_id = "TNDM-UPCF";
    repo.create_ticket(Some(ticket_id), "Content file test");

    let content_file = repo.path().join("new-content.md");
    fs::write(&content_file, "# Updated Content\n\nNew body here.\n").expect("write content file");

    repo.run_assert(&[
        "ticket",
        "update",
        ticket_id,
        "--content-file",
        content_file.to_str().unwrap(),
    ]);

    let show_stdout = repo.run_assert(&["ticket", "show", ticket_id]);
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
    let repo = TestRepo::new();

    let ticket_id = "TNDM-UPTYP";
    repo.create_ticket(Some(ticket_id), "Type test");

    repo.run_assert(&["ticket", "update", ticket_id, "--type", "bug"]);

    let show_stdout = repo.run_assert(&["ticket", "show", ticket_id]);
    assert!(
        show_stdout.contains("bug"),
        "show output was: {show_stdout}"
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_update_rejects_empty_title() {
    let repo = TestRepo::new();

    let ticket_id = "TNDM-UPET";
    repo.create_ticket(Some(ticket_id), "Empty title test");

    let output = repo.run(&["ticket", "update", ticket_id, "--title", ""]);

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
    let repo = TestRepo::new();

    let ticket_id = "TNDM-UPIS";
    repo.create_ticket(Some(ticket_id), "Invalid status test");

    let output = repo.run(&["ticket", "update", ticket_id, "--status", "invalid"]);

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
    let repo = TestRepo::new();

    let ticket_id = "TNDM-UPIP";
    repo.create_ticket(Some(ticket_id), "Invalid priority test");

    let output = repo.run(&["ticket", "update", ticket_id, "--priority", "p9"]);

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
    let repo = TestRepo::new();

    let ticket_id = "TNDM-UPIT";
    repo.create_ticket(Some(ticket_id), "Invalid type test");

    let output = repo.run(&["ticket", "update", ticket_id, "--type", "story"]);

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
    let repo = TestRepo::new();

    let ticket_id = "TNDM-SHOWJ";
    repo.create_ticket(Some(ticket_id), "JSON show test");

    let json = repo.run_json(&["ticket", "show", ticket_id]);
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
    let repo = TestRepo::new();

    let ticket_id = "TNDM-UJ01";
    repo.create_ticket(Some(ticket_id), "Update JSON test");

    let json = repo.run_json(&["ticket", "update", ticket_id, "--status", "in_progress"]);
    assert_eq!(json["schema_version"], 1);
    assert_eq!(json["id"], "TNDM-UJ01");
    assert_eq!(json["status"], "in_progress");
    assert_eq!(json["revision"], 2);
    assert_eq!(json["content_path"], ".tndm/tickets/TNDM-UJ01/content.md");
}

#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_update_with_effort_flag() {
    let repo = TestRepo::new();

    repo.create_ticket(Some("TNDM-EF02"), "Effort update test");
    repo.run_assert(&["ticket", "update", "TNDM-EF02", "--effort", "xl"]);

    let show_stdout = repo.run_assert(&["ticket", "show", "TNDM-EF02"]);
    assert!(show_stdout.contains("xl"), "show output was: {show_stdout}");
}

#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_update_add_tags_preserves_existing_and_sorts() {
    let repo = TestRepo::new();

    let ticket_id = "TNDM-ADTG";
    repo.run_assert(&[
        "ticket",
        "create",
        "Add tags test",
        "--id",
        ticket_id,
        "--tags",
        "beta,alpha",
    ]);

    repo.run_assert(&["ticket", "update", ticket_id, "--add-tags", "gamma,alpha"]);

    let show_stdout = repo.run_assert(&["ticket", "show", ticket_id]);
    assert!(
        show_stdout.contains("alpha, beta, gamma"),
        "tags should be deduped and sorted; show output was: {show_stdout}"
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_update_remove_tags_filters_existing() {
    let repo = TestRepo::new();

    let ticket_id = "TNDM-RMTG";
    repo.run_assert(&[
        "ticket",
        "create",
        "Remove tags test",
        "--id",
        ticket_id,
        "--tags",
        "a,b,c",
    ]);

    repo.run_assert(&["ticket", "update", ticket_id, "--remove-tags", "b,d"]);

    let show_stdout = repo.run_assert(&["ticket", "show", ticket_id]);
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
    let repo = TestRepo::new();

    let ticket_id = "TNDM-FLOWRP";
    repo.run_assert(&[
        "ticket",
        "create",
        "Replace flow tag test",
        "--id",
        ticket_id,
        "--tags",
        "flow:brainstorm",
    ]);

    let json = repo.run_json(&[
        "ticket",
        "update",
        ticket_id,
        "--remove-tags",
        "flow:brainstorm,flow:planned,flow:applying,flow:done",
        "--add-tags",
        "flow:planned",
    ]);
    assert_eq!(json["tags"], serde_json::json!(["flow:planned"]));
}

#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_update_add_tags_conflicts_with_tags_flag() {
    let repo = TestRepo::new();

    let ticket_id = "TNDM-CFTG";
    repo.create_ticket(Some(ticket_id), "Conflict test");

    let output = repo.run(&[
        "ticket",
        "update",
        ticket_id,
        "--tags",
        "x",
        "--add-tags",
        "y",
    ]);

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
