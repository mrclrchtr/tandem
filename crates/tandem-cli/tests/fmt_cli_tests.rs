#![allow(clippy::disallowed_types)]

use std::{fs, process::Command};
use tandem_storage::fingerprint_bytes;

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
            "type = \"task\"\n",
            "priority = \"p2\"\n",
            "depends_on = []\n",
            "tags = []\n",
            "\n",
            "[[documents]]\n",
            "name = \"content\"\n",
            "path = \"content.md\"\n",
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
fn fmt_adds_trailing_newline_to_content_md() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let ticket_dir = repo_root
        .path()
        .join(".tndm")
        .join("tickets")
        .join("TNDM-ABC123");
    fs::create_dir_all(&ticket_dir).expect("create ticket dir");

    // Write meta.toml with content.md registered
    fs::write(
        ticket_dir.join("meta.toml"),
        concat!(
            "schema_version = 1\n",
            "id = \"TNDM-ABC123\"\n",
            "title = \"Test\"\n",
            "type = \"task\"\n",
            "priority = \"p2\"\n",
            "depends_on = []\n",
            "tags = []\n",
            "\n",
            "[[documents]]\n",
            "name = \"content\"\n",
            "path = \"content.md\"\n",
        ),
    )
    .expect("write meta.toml");

    // Write content.md WITHOUT trailing newline
    let body = "hello world";
    fs::write(ticket_dir.join("content.md"), body).expect("write content.md");

    // Build a matching fingerprint for the body (no trailing newline)
    let stored_fp = fingerprint_bytes(body.as_bytes());

    fs::write(
        ticket_dir.join("state.toml"),
        format!(
            "schema_version = 1\nstatus = \"todo\"\nupdated_at = \"2026-03-03T12:34:56Z\"\nrevision = 1\n\n[document_fingerprints]\ncontent = \"{stored_fp}\"\n"
        ),
    )
    .expect("write state.toml");

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

    // content.md should now end with a trailing newline
    let content = fs::read_to_string(ticket_dir.join("content.md")).expect("read content.md");
    assert_eq!(
        content, "hello world\n",
        "content.md should end with trailing newline"
    );

    // state.toml fingerprint should be updated
    let updated_fp = fingerprint_bytes(b"hello world\n");
    let state_text = fs::read_to_string(ticket_dir.join("state.toml")).expect("read state.toml");
    assert!(
        state_text.contains(&updated_fp),
        "state.toml should contain updated fingerprint"
    );
    // Old fingerprint should be gone
    assert!(
        !state_text.contains(&stored_fp),
        "state.toml should NOT contain old fingerprint"
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn fmt_adds_trailing_newline_to_plan_md() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let ticket_dir = repo_root
        .path()
        .join(".tndm")
        .join("tickets")
        .join("TNDM-PLAN01");
    fs::create_dir_all(&ticket_dir).expect("create ticket dir");

    // Write meta.toml with content.md AND plan.md registered
    fs::write(
        ticket_dir.join("meta.toml"),
        concat!(
            "schema_version = 1\n",
            "id = \"TNDM-PLAN01\"\n",
            "title = \"Plan test\"\n",
            "type = \"task\"\n",
            "priority = \"p2\"\n",
            "depends_on = []\n",
            "tags = []\n",
            "\n",
            "[[documents]]\n",
            "name = \"content\"\n",
            "path = \"content.md\"\n",
            "\n",
            "[[documents]]\n",
            "name = \"plan\"\n",
            "path = \"plan.md\"\n",
        ),
    )
    .expect("write meta.toml");

    // Write content.md WITH trailing newline (already canonical)
    fs::write(ticket_dir.join("content.md"), "ok\n").expect("write content.md");
    // Write plan.md WITHOUT trailing newline
    fs::write(ticket_dir.join("plan.md"), "plan body").expect("write plan.md");

    let content_fp = fingerprint_bytes(b"ok\n");
    let plan_fp = fingerprint_bytes(b"plan body");

    fs::write(
        ticket_dir.join("state.toml"),
        format!(
            "schema_version = 1\nstatus = \"todo\"\nupdated_at = \"2026-03-03T12:34:56Z\"\nrevision = 1\n\n[document_fingerprints]\ncontent = \"{content_fp}\"\nplan = \"{plan_fp}\"\n"
        ),
    )
    .expect("write state.toml");

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

    // plan.md should now end with trailing newline
    let plan = fs::read_to_string(ticket_dir.join("plan.md")).expect("read plan.md");
    assert_eq!(plan, "plan body\n");

    // content.md should remain unchanged
    let content = fs::read_to_string(ticket_dir.join("content.md")).expect("read content.md");
    assert_eq!(content, "ok\n");

    // Fingerprint for plan should be updated
    let updated_plan_fp = fingerprint_bytes(b"plan body\n");
    let state_text = fs::read_to_string(ticket_dir.join("state.toml")).expect("read state.toml");
    assert!(
        state_text.contains(&updated_plan_fp),
        "state.toml should contain updated plan fingerprint"
    );
    assert!(
        !state_text.contains(&plan_fp),
        "state.toml should NOT contain old plan fingerprint"
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn fmt_adds_trailing_newline_to_task_detail_doc() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let ticket_dir = repo_root
        .path()
        .join(".tndm")
        .join("tickets")
        .join("TNDM-TASK01");
    fs::create_dir_all(&ticket_dir).expect("create ticket dir");

    // Write meta.toml with content.md AND a task detail doc
    fs::write(
        ticket_dir.join("meta.toml"),
        concat!(
            "schema_version = 1\n",
            "id = \"TNDM-TASK01\"\n",
            "title = \"Task detail test\"\n",
            "type = \"task\"\n",
            "priority = \"p2\"\n",
            "depends_on = []\n",
            "tags = []\n",
            "\n",
            "[[documents]]\n",
            "name = \"content\"\n",
            "path = \"content.md\"\n",
            "\n",
            "[[documents]]\n",
            "name = \"task-01\"\n",
            "path = \"tasks/task-01.md\"\n",
        ),
    )
    .expect("write meta.toml");

    fs::create_dir_all(ticket_dir.join("tasks")).expect("create tasks dir");

    // Write content.md WITH trailing newline
    fs::write(ticket_dir.join("content.md"), "ok\n").expect("write content.md");
    // Write task detail doc WITHOUT trailing newline
    fs::write(ticket_dir.join("tasks").join("task-01.md"), "task detail")
        .expect("write task-01.md");

    let content_fp = fingerprint_bytes(b"ok\n");
    let task_fp = fingerprint_bytes(b"task detail");

    fs::write(
        ticket_dir.join("state.toml"),
        format!(
            "schema_version = 1\nstatus = \"todo\"\nupdated_at = \"2026-03-03T12:34:56Z\"\nrevision = 1\n\n[document_fingerprints]\ncontent = \"{content_fp}\"\ntask-01 = \"{task_fp}\"\n"
        ),
    )
    .expect("write state.toml");

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

    // task-01.md should now end with trailing newline
    let task =
        fs::read_to_string(ticket_dir.join("tasks").join("task-01.md")).expect("read task-01.md");
    assert_eq!(task, "task detail\n");

    // Fingerprint for task should be updated
    let updated_task_fp = fingerprint_bytes(b"task detail\n");
    let state_text = fs::read_to_string(ticket_dir.join("state.toml")).expect("read state.toml");
    assert!(
        state_text.contains(&updated_task_fp),
        "state.toml should contain updated task fingerprint"
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn fmt_check_reports_missing_trailing_newline() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let ticket_dir = repo_root
        .path()
        .join(".tndm")
        .join("tickets")
        .join("TNDM-CHECK01");
    fs::create_dir_all(&ticket_dir).expect("create ticket dir");

    // Canonical meta.toml
    fs::write(
        ticket_dir.join("meta.toml"),
        concat!(
            "schema_version = 1\n",
            "id = \"TNDM-CHECK01\"\n",
            "title = \"Check test\"\n",
            "type = \"task\"\n",
            "priority = \"p2\"\n",
            "depends_on = []\n",
            "tags = []\n",
            "\n",
            "[[documents]]\n",
            "name = \"content\"\n",
            "path = \"content.md\"\n",
        ),
    )
    .expect("write meta.toml");

    // Canonical state.toml with correct fingerprint
    let body = "no newline at end";
    let fp = fingerprint_bytes(body.as_bytes());
    fs::write(ticket_dir.join("content.md"), body).expect("write content.md");
    fs::write(
        ticket_dir.join("state.toml"),
        format!(
            "schema_version = 1\nstatus = \"todo\"\nupdated_at = \"2026-03-03T12:34:56Z\"\nrevision = 1\n\n[document_fingerprints]\ncontent = \"{fp}\"\n"
        ),
    )
    .expect("write state.toml");

    // --check should fail
    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .arg("fmt")
        .arg("--check")
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm fmt --check");

    assert!(
        !output.status.success(),
        "fmt --check should fail when content.md is missing trailing newline"
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert!(
        stdout.contains("content.md"),
        "stdout should mention content.md, was: {stdout:?}"
    );
    // state.toml should also be reported since fingerprint would change
    assert!(
        stdout.contains("state.toml"),
        "stdout should mention state.toml alongside content.md, was: {stdout:?}"
    );

    // File should NOT have been modified in --check mode
    let content = fs::read_to_string(ticket_dir.join("content.md")).expect("read content.md");
    assert_eq!(content, "no newline at end");
}

#[test]
#[allow(clippy::disallowed_methods)]
fn fmt_canonical_content_md_is_noop() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let ticket_dir = repo_root
        .path()
        .join(".tndm")
        .join("tickets")
        .join("TNDM-NOOP01");
    fs::create_dir_all(&ticket_dir).expect("create ticket dir");

    // Write everything in canonical form
    fs::write(
        ticket_dir.join("meta.toml"),
        concat!(
            "schema_version = 1\n",
            "id = \"TNDM-NOOP01\"\n",
            "title = \"Noop\"\n",
            "type = \"task\"\n",
            "priority = \"p2\"\n",
            "depends_on = []\n",
            "tags = []\n",
            "\n",
            "[[documents]]\n",
            "name = \"content\"\n",
            "path = \"content.md\"\n",
        ),
    )
    .expect("write meta.toml");

    // content.md ALREADY has trailing newline
    let body = "already canonical\n";
    let fp = fingerprint_bytes(body.as_bytes());
    fs::write(ticket_dir.join("content.md"), body).expect("write content.md");
    fs::write(
        ticket_dir.join("state.toml"),
        format!(
            "schema_version = 1\nstatus = \"todo\"\nupdated_at = \"2026-03-03T12:34:56Z\"\nrevision = 1\n\n[document_fingerprints]\ncontent = \"{fp}\"\n"
        ),
    )
    .expect("write state.toml");

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

    // content.md should remain unchanged
    let content = fs::read_to_string(ticket_dir.join("content.md")).expect("read content.md");
    assert_eq!(content, "already canonical\n");

    // fingerprint should also remain unchanged
    let state_text = fs::read_to_string(ticket_dir.join("state.toml")).expect("read state.toml");
    assert!(state_text.contains(&fp));
}

#[test]
#[allow(clippy::disallowed_methods)]
fn fmt_check_passes_when_content_md_has_trailing_newline() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let ticket_dir = repo_root
        .path()
        .join(".tndm")
        .join("tickets")
        .join("TNDM-CLEAN01");
    fs::create_dir_all(&ticket_dir).expect("create ticket dir");

    // Meta canonical
    fs::write(
        ticket_dir.join("meta.toml"),
        concat!(
            "schema_version = 1\n",
            "id = \"TNDM-CLEAN01\"\n",
            "title = \"Clean\"\n",
            "type = \"task\"\n",
            "priority = \"p2\"\n",
            "depends_on = []\n",
            "tags = []\n",
            "\n",
            "[[documents]]\n",
            "name = \"content\"\n",
            "path = \"content.md\"\n",
        ),
    )
    .expect("write meta.toml");

    let body = "clean\n";
    let fp = fingerprint_bytes(body.as_bytes());
    fs::write(ticket_dir.join("content.md"), body).expect("write content.md");
    fs::write(
        ticket_dir.join("state.toml"),
        format!(
            "schema_version = 1\nstatus = \"todo\"\nupdated_at = \"2026-03-03T12:34:56Z\"\nrevision = 1\n\n[document_fingerprints]\ncontent = \"{fp}\"\n"
        ),
    )
    .expect("write state.toml");

    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .arg("fmt")
        .arg("--check")
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm fmt --check");

    assert!(
        output.status.success(),
        "fmt --check should pass when everything is canonical, stderr was: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn fmt_collapses_multiple_trailing_newlines() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let ticket_dir = repo_root
        .path()
        .join(".tndm")
        .join("tickets")
        .join("TNDM-NL01");
    fs::create_dir_all(&ticket_dir).expect("create ticket dir");

    fs::write(
        ticket_dir.join("meta.toml"),
        concat!(
            "schema_version = 1\n",
            "id = \"TNDM-NL01\"\n",
            "title = \"Newlines\"\n",
            "type = \"task\"\n",
            "priority = \"p2\"\n",
            "depends_on = []\n",
            "tags = []\n",
            "\n",
            "[[documents]]\n",
            "name = \"content\"\n",
            "path = \"content.md\"\n",
        ),
    )
    .expect("write meta.toml");

    // content.md with multiple trailing newlines
    let body = "body\n\n\n";
    fs::write(ticket_dir.join("content.md"), body).expect("write content.md");
    let stored_fp = fingerprint_bytes(body.as_bytes());

    fs::write(
        ticket_dir.join("state.toml"),
        format!(
            "schema_version = 1\nstatus = \"todo\"\nupdated_at = \"2026-03-03T12:34:56Z\"\nrevision = 1\n\n[document_fingerprints]\ncontent = \"{stored_fp}\"\n"
        ),
    )
    .expect("write state.toml");

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

    // Multiple trailing newlines should be collapsed to one
    let content = fs::read_to_string(ticket_dir.join("content.md")).expect("read content.md");
    assert_eq!(content, "body\n");

    // Fingerprint should be updated
    let updated_fp = fingerprint_bytes(b"body\n");
    let state_text = fs::read_to_string(ticket_dir.join("state.toml")).expect("read state.toml");
    assert!(
        state_text.contains(&updated_fp),
        "state.toml should contain updated fingerprint"
    );
    assert!(
        !state_text.contains(&stored_fp),
        "state.toml should NOT contain old fingerprint"
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn fmt_normalizes_empty_content() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let ticket_dir = repo_root
        .path()
        .join(".tndm")
        .join("tickets")
        .join("TNDM-EMPTY01");
    fs::create_dir_all(&ticket_dir).expect("create ticket dir");

    fs::write(
        ticket_dir.join("meta.toml"),
        concat!(
            "schema_version = 1\n",
            "id = \"TNDM-EMPTY01\"\n",
            "title = \"Empty\"\n",
            "type = \"task\"\n",
            "priority = \"p2\"\n",
            "depends_on = []\n",
            "tags = []\n",
            "\n",
            "[[documents]]\n",
            "name = \"content\"\n",
            "path = \"content.md\"\n",
        ),
    )
    .expect("write meta.toml");

    // Empty (0-byte) content.md
    fs::write(ticket_dir.join("content.md"), "").expect("write content.md");
    // Fingerprint for empty content
    let stored_fp = fingerprint_bytes(b"");

    fs::write(
        ticket_dir.join("state.toml"),
        format!(
            "schema_version = 1\nstatus = \"todo\"\nupdated_at = \"2026-03-03T12:34:56Z\"\nrevision = 1\n\n[document_fingerprints]\ncontent = \"{stored_fp}\"\n"
        ),
    )
    .expect("write state.toml");

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

    // Empty content should become a single newline
    let content = fs::read_to_string(ticket_dir.join("content.md")).expect("read content.md");
    assert_eq!(content, "\n");

    // Fingerprint should be updated for the newline
    let updated_fp = fingerprint_bytes(b"\n");
    let state_text = fs::read_to_string(ticket_dir.join("state.toml")).expect("read state.toml");
    assert!(
        state_text.contains(&updated_fp),
        "state.toml should contain updated fingerprint for single newline"
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn fmt_check_reports_drift_and_missing_newline_together() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let ticket_dir = repo_root
        .path()
        .join(".tndm")
        .join("tickets")
        .join("TNDM-DRIFT01");
    fs::create_dir_all(&ticket_dir).expect("create ticket dir");

    // Canonical meta.toml
    fs::write(
        ticket_dir.join("meta.toml"),
        concat!(
            "schema_version = 1\n",
            "id = \"TNDM-DRIFT01\"\n",
            "title = \"Drift check\"\n",
            "type = \"task\"\n",
            "priority = \"p2\"\n",
            "depends_on = []\n",
            "tags = []\n",
            "\n",
            "[[documents]]\n",
            "name = \"content\"\n",
            "path = \"content.md\"\n",
        ),
    )
    .expect("write meta.toml");

    // content.md WITHOUT trailing newline (non-canonical)
    let body = "stale content";
    fs::write(ticket_dir.join("content.md"), body).expect("write content.md");

    // Stored fingerprint does NOT match (simulates external edit)
    let stale_fp = fingerprint_bytes(b"mismatched fingerprint");
    fs::write(
        ticket_dir.join("state.toml"),
        format!(
            "schema_version = 1\nstatus = \"todo\"\nupdated_at = \"2026-03-03T12:34:56Z\"\nrevision = 1\n\n[document_fingerprints]\ncontent = \"{stale_fp}\"\n"
        ),
    )
    .expect("write state.toml");

    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .arg("fmt")
        .arg("--check")
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm fmt --check");

    // Should fail with both file and drift reported
    assert!(
        !output.status.success(),
        "fmt --check should fail with both non-canonical content.md and stale fingerprint"
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert!(
        stdout.contains("content.md"),
        "stdout should mention content.md, was: {stdout:?}"
    );
    assert!(
        stdout.contains("state.toml"),
        "stdout should mention state.toml (fingerprint would change), was: {stdout:?}"
    );
    assert!(
        stdout.contains("stale fingerprint"),
        "stdout should mention stale fingerprint, was: {stdout:?}"
    );

    // File should NOT have been modified in --check mode
    let content = fs::read_to_string(ticket_dir.join("content.md")).expect("read content.md");
    assert_eq!(content, "stale content");
}

#[test]
#[allow(clippy::disallowed_methods)]
fn fmt_fixes_structured_files_and_content_together() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let ticket_dir = repo_root
        .path()
        .join(".tndm")
        .join("tickets")
        .join("TNDM-BOTH01");
    fs::create_dir_all(&ticket_dir).expect("create ticket dir");

    // meta.toml with non-canonical key ordering
    fs::write(
        ticket_dir.join("meta.toml"),
        concat!(
            "title = \"Both fix\"\n",
            "id = \"TNDM-BOTH01\"\n",
            "schema_version = 1\n",
            "priority = \"p2\"\n",
            "type = \"task\"\n",
            "tags = []\n",
            "depends_on = []\n",
        ),
    )
    .expect("write meta.toml");

    // content.md WITHOUT trailing newline
    let body = "needs trailing newline";
    fs::write(ticket_dir.join("content.md"), body).expect("write content.md");
    let fp = fingerprint_bytes(body.as_bytes());

    // state.toml with canonical formatting (but fingerprints match original content)
    fs::write(
        ticket_dir.join("state.toml"),
        format!(
            "schema_version = 1\nstatus = \"todo\"\nupdated_at = \"2026-03-03T12:34:56Z\"\nrevision = 1\n\n[document_fingerprints]\ncontent = \"{fp}\"\n"
        ),
    )
    .expect("write state.toml");

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

    // meta.toml should now be canonical (keys reordered)
    let meta = fs::read_to_string(ticket_dir.join("meta.toml")).expect("read meta.toml");
    assert!(
        meta.starts_with("schema_version = 1\nid = \"TNDM-BOTH01\"\n"),
        "meta.toml should be canonical, got: {meta}"
    );

    // content.md should end with trailing newline
    let content = fs::read_to_string(ticket_dir.join("content.md")).expect("read content.md");
    assert_eq!(content, "needs trailing newline\n");

    // state.toml fingerprint should be updated
    let updated_fp = fingerprint_bytes(b"needs trailing newline\n");
    let state_text = fs::read_to_string(ticket_dir.join("state.toml")).expect("read state.toml");
    assert!(
        state_text.contains(&updated_fp),
        "state.toml should contain updated fingerprint"
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn fmt_preserves_windows_line_endings() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let ticket_dir = repo_root
        .path()
        .join(".tndm")
        .join("tickets")
        .join("TNDM-CRLF01");
    fs::create_dir_all(&ticket_dir).expect("create ticket dir");

    fs::write(
        ticket_dir.join("meta.toml"),
        concat!(
            "schema_version = 1\n",
            "id = \"TNDM-CRLF01\"\n",
            "title = \"CRLF test\"\n",
            "type = \"task\"\n",
            "priority = \"p2\"\n",
            "depends_on = []\n",
            "tags = []\n",
            "\n",
            "[[documents]]\n",
            "name = \"content\"\n",
            "path = \"content.md\"\n",
        ),
    )
    .expect("write meta.toml");

    // content.md with Windows \r\n line ending (already ends with a line terminator)
    let body = "hello\r\n";
    fs::write(ticket_dir.join("content.md"), body).expect("write content.md");
    let fp = fingerprint_bytes(body.as_bytes());

    fs::write(
        ticket_dir.join("state.toml"),
        format!(
            "schema_version = 1\nstatus = \"todo\"\nupdated_at = \"2026-03-03T12:34:56Z\"\nrevision = 1\n\n[document_fingerprints]\ncontent = \"{fp}\"\n"
        ),
    )
    .expect("write state.toml");

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

    // \r\n ending should be preserved unchanged
    let content = fs::read_to_string(ticket_dir.join("content.md")).expect("read content.md");
    assert_eq!(content, "hello\r\n");

    // Fingerprint should remain unchanged
    let state_text = fs::read_to_string(ticket_dir.join("state.toml")).expect("read state.toml");
    assert!(state_text.contains(&fp));
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
