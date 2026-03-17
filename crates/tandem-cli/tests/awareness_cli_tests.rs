#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use std::{fs, path::Path, process::Command};

#[test]
fn awareness_prints_empty_json_when_snapshots_match() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    run_git(repo_root.path(), &["init", "-b", "main"]);
    run_git(repo_root.path(), &["config", "user.name", "Test User"]);
    run_git(
        repo_root.path(),
        &["config", "user.email", "test@example.com"],
    );

    write_ticket(repo_root.path(), "TNDM-1", "One", "todo", "p2", &[]);
    run_git(repo_root.path(), &["add", "."]);
    run_git(repo_root.path(), &["commit", "-m", "base"]);

    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["awareness", "--against", "HEAD", "--json"])
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm awareness");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stdout).expect("stdout should be UTF-8"),
        concat!(
            "{\n",
            "  \"schema_version\": 1,\n",
            "  \"against\": \"HEAD\",\n",
            "  \"tickets\": []\n",
            "}\n",
        )
    );
}

#[test]
fn awareness_reports_added_current_added_against_and_diverged_sorted() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    run_git(repo_root.path(), &["init", "-b", "main"]);
    run_git(repo_root.path(), &["config", "user.name", "Test User"]);
    run_git(
        repo_root.path(),
        &["config", "user.email", "test@example.com"],
    );

    write_ticket(
        repo_root.path(),
        "TNDM-1",
        "Against only",
        "todo",
        "p2",
        &[],
    );
    write_ticket(repo_root.path(), "TNDM-3", "Diverged", "todo", "p2", &[]);
    run_git(repo_root.path(), &["add", "."]);
    run_git(repo_root.path(), &["commit", "-m", "base"]);

    fs::remove_dir_all(
        repo_root
            .path()
            .join(".tndm")
            .join("tickets")
            .join("TNDM-1"),
    )
    .expect("remove TNDM-1 from working tree");
    write_ticket(
        repo_root.path(),
        "TNDM-2",
        "Current only",
        "todo",
        "p2",
        &[],
    );
    write_ticket(
        repo_root.path(),
        "TNDM-3",
        "Diverged",
        "in_progress",
        "p2",
        &["TNDM-1"],
    );

    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["awareness", "--against", "HEAD", "--json"])
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm awareness");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stdout).expect("stdout should be UTF-8"),
        concat!(
            "{\n",
            "  \"schema_version\": 1,\n",
            "  \"against\": \"HEAD\",\n",
            "  \"tickets\": [\n",
            "    {\n",
            "      \"id\": \"TNDM-1\",\n",
            "      \"change\": \"added_against\"\n",
            "    },\n",
            "    {\n",
            "      \"id\": \"TNDM-2\",\n",
            "      \"change\": \"added_current\"\n",
            "    },\n",
            "    {\n",
            "      \"id\": \"TNDM-3\",\n",
            "      \"change\": \"diverged\",\n",
            "      \"fields\": {\n",
            "        \"status\": {\n",
            "          \"current\": \"in_progress\",\n",
            "          \"against\": \"todo\"\n",
            "        },\n",
            "        \"depends_on\": {\n",
            "          \"current\": [\n",
            "            \"TNDM-1\"\n",
            "          ],\n",
            "          \"against\": []\n",
            "        }\n",
            "      }\n",
            "    }\n",
            "  ]\n",
            "}\n",
        )
    );
}

#[test]
fn awareness_errors_for_invalid_ref() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    run_git(repo_root.path(), &["init", "-b", "main"]);
    run_git(repo_root.path(), &["config", "user.name", "Test User"]);
    run_git(
        repo_root.path(),
        &["config", "user.email", "test@example.com"],
    );

    write_ticket(repo_root.path(), "TNDM-1", "One", "todo", "p2", &[]);
    run_git(repo_root.path(), &["add", "."]);
    run_git(repo_root.path(), &["commit", "-m", "base"]);

    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["awareness", "--against", "does-not-exist"])
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm awareness");

    assert!(
        !output.status.success(),
        "awareness should fail for an invalid ref"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be UTF-8");
    assert!(
        stderr.contains("git rev-parse --verify does-not-exist^{commit} failed"),
        "stderr was: {stderr:?}"
    );
}

#[test]
fn awareness_text_output_shows_human_readable_format() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    run_git(repo_root.path(), &["init", "-b", "main"]);
    run_git(repo_root.path(), &["config", "user.name", "Test User"]);
    run_git(
        repo_root.path(),
        &["config", "user.email", "test@example.com"],
    );

    write_ticket(
        repo_root.path(),
        "TNDM-1",
        "Against only",
        "todo",
        "p2",
        &[],
    );
    write_ticket(repo_root.path(), "TNDM-3", "Diverged", "todo", "p2", &[]);
    run_git(repo_root.path(), &["add", "."]);
    run_git(repo_root.path(), &["commit", "-m", "base"]);

    fs::remove_dir_all(
        repo_root
            .path()
            .join(".tndm")
            .join("tickets")
            .join("TNDM-1"),
    )
    .expect("remove TNDM-1");
    write_ticket(
        repo_root.path(),
        "TNDM-2",
        "Current only",
        "todo",
        "p2",
        &[],
    );
    write_ticket(
        repo_root.path(),
        "TNDM-3",
        "Diverged",
        "in_progress",
        "p1",
        &["TNDM-1"],
    );

    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["awareness", "--against", "HEAD"])
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm awareness");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert!(stdout.contains("Against: HEAD"), "stdout was: {stdout:?}");
    assert!(
        stdout.contains("TNDM-1") && stdout.contains("added (against)"),
        "stdout was: {stdout:?}"
    );
    assert!(
        stdout.contains("TNDM-2") && stdout.contains("added (current)"),
        "stdout was: {stdout:?}"
    );
    assert!(
        stdout.contains("TNDM-3") && stdout.contains("diverged"),
        "stdout was: {stdout:?}"
    );
    assert!(
        stdout.contains("status:") && stdout.contains("in_progress -> todo"),
        "stdout was: {stdout:?}"
    );
    assert!(
        stdout.contains("priority:") && stdout.contains("p1 -> p2"),
        "stdout was: {stdout:?}"
    );
}

#[test]
fn awareness_text_output_empty_shows_no_changes() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    run_git(repo_root.path(), &["init", "-b", "main"]);
    run_git(repo_root.path(), &["config", "user.name", "Test User"]);
    run_git(
        repo_root.path(),
        &["config", "user.email", "test@example.com"],
    );

    write_ticket(repo_root.path(), "TNDM-1", "One", "todo", "p2", &[]);
    run_git(repo_root.path(), &["add", "."]);
    run_git(repo_root.path(), &["commit", "-m", "base"]);

    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["awareness", "--against", "HEAD"])
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm awareness");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert!(stdout.contains("Against: HEAD"), "stdout was: {stdout:?}");
    assert!(stdout.contains("No changes."), "stdout was: {stdout:?}");
}

fn run_git(repo_root: &Path, args: &[&str]) {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo_root)
        .env("GIT_AUTHOR_NAME", "Test User")
        .env("GIT_AUTHOR_EMAIL", "test@example.com")
        .env("GIT_COMMITTER_NAME", "Test User")
        .env("GIT_COMMITTER_EMAIL", "test@example.com")
        .env("GIT_CONFIG_COUNT", "1")
        .env("GIT_CONFIG_KEY_0", "commit.gpgsign")
        .env("GIT_CONFIG_VALUE_0", "false")
        .output()
        .expect("run git command");

    assert!(
        output.status.success(),
        "git {:?} failed: {}",
        args,
        String::from_utf8_lossy(&output.stderr)
    );
}

fn write_ticket(
    repo_root: &Path,
    id: &str,
    title: &str,
    status: &str,
    priority: &str,
    depends_on: &[&str],
) {
    let ticket_dir = repo_root.join(".tndm").join("tickets").join(id);
    fs::create_dir_all(&ticket_dir).expect("create ticket dir");

    let depends_on = depends_on
        .iter()
        .map(|dependency_id| format!("\"{dependency_id}\""))
        .collect::<Vec<_>>()
        .join(", ");

    fs::write(
        ticket_dir.join("meta.toml"),
        format!(
            "schema_version = 1\nid = \"{id}\"\ntitle = \"{title}\"\n\ntype = \"task\"\npriority = \"{priority}\"\n\ndepends_on = [{depends_on}]\ntags = []\n"
        ),
    )
    .expect("write meta.toml");
    fs::write(
        ticket_dir.join("state.toml"),
        format!(
            "schema_version = 1\nstatus = \"{status}\"\nupdated_at = \"2026-03-08T00:00:00Z\"\nrevision = 1\n"
        ),
    )
    .expect("write state.toml");
    fs::write(ticket_dir.join("content.md"), "body\n").expect("write content.md");
}
