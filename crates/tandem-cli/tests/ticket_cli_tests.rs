use std::{fs, process::Command};

use regex::Regex;

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
fn ticket_show_prints_meta_state_and_content_sections() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let ticket_id = "TNDM-ABC123";
    let content_file = repo_root.path().join("ticket-content.md");
    fs::write(&content_file, "# Details\n\nshow output body").expect("write content file");

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
    assert!(stdout.contains("## meta.toml\n"));
    assert!(stdout.contains("id = \"TNDM-ABC123\"\n"));
    assert!(stdout.contains("title = \"Show ticket content\"\n"));

    assert!(stdout.contains("## state.toml\n"));
    assert!(stdout.contains("status = \"todo\"\n"));
    assert!(stdout.contains("revision = 1\n"));

    assert!(stdout.contains("## content.md\n"));
    assert!(stdout.contains("# Details\n"));
    assert!(stdout.contains("show output body"));
    assert!(stdout.ends_with('\n'));
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
        "TNDM-1\ttodo\tFirst ticket\nTNDM-2\ttodo\tSecond ticket\n"
    );
}
