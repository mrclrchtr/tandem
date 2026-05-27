#![allow(clippy::disallowed_types, unused_imports)]

mod common;

use common::*;
use std::{fs, process::Command};

use regex::Regex;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

#[test]
#[allow(clippy::disallowed_methods)]
fn task_add_creates_task_with_auto_number() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");
    create_test_ticket(repo_root.path(), "TNDM-ADDN1", "Add task test");

    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args([
            "ticket",
            "task",
            "add",
            "TNDM-ADDN1",
            "--title",
            "First task",
            "--json",
        ])
        .current_dir(repo_root.path())
        .output()
        .expect("add task");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // List tasks and verify
    let list = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "task", "list", "TNDM-ADDN1", "--json"])
        .current_dir(repo_root.path())
        .output()
        .expect("list tasks");
    assert!(list.status.success());
    let tasks: Vec<serde_json::Value> =
        serde_json::from_str(&String::from_utf8(list.stdout).unwrap()).unwrap();
    assert_eq!(tasks[0]["number"], 1);
    assert_eq!(tasks[0]["title"], "First task");
    assert_eq!(tasks[0]["status"], "todo");
}

#[test]
#[allow(clippy::disallowed_methods)]
fn task_add_increments_number() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");
    create_test_ticket(repo_root.path(), "TNDM-ADDN2", "Add two tasks");

    // Add first task
    let _ = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "task", "add", "TNDM-ADDN2", "--title", "Task A"])
        .current_dir(repo_root.path())
        .output()
        .expect("add task A");

    // Add second task
    let _ = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "task", "add", "TNDM-ADDN2", "--title", "Task B"])
        .current_dir(repo_root.path())
        .output()
        .expect("add task B");

    // List tasks with JSON
    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "task", "list", "TNDM-ADDN2", "--json"])
        .current_dir(repo_root.path())
        .output()
        .expect("list tasks");
    assert!(output.status.success());
    let tasks: Vec<serde_json::Value> =
        serde_json::from_str(&String::from_utf8(output.stdout).unwrap()).unwrap();
    assert_eq!(tasks.len(), 2);
    assert_eq!(tasks[0]["number"], 1);
    assert_eq!(tasks[0]["title"], "Task A");
    assert_eq!(tasks[1]["number"], 2);
    assert_eq!(tasks[1]["title"], "Task B");
}

#[test]
#[allow(clippy::disallowed_methods)]
fn task_list_json_output() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");
    create_test_ticket(repo_root.path(), "TNDM-LST1", "Task list test");

    // Add a task
    Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "task", "add", "TNDM-LST1", "--title", "List me"])
        .current_dir(repo_root.path())
        .output()
        .expect("add task");

    // List --json
    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "task", "list", "TNDM-LST1", "--json"])
        .current_dir(repo_root.path())
        .output()
        .expect("list tasks json");
    assert!(output.status.success());
    let tasks: Vec<serde_json::Value> =
        serde_json::from_str(&String::from_utf8(output.stdout).unwrap()).unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0]["number"], 1);
    assert_eq!(tasks[0]["title"], "List me");
    assert_eq!(tasks[0]["status"], "todo");
}

#[test]
#[allow(clippy::disallowed_methods)]
fn task_complete_marks_task_done() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");
    create_test_ticket(repo_root.path(), "TNDM-CMP1", "Complete test");

    Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "task", "add", "TNDM-CMP1", "--title", "Finish me"])
        .current_dir(repo_root.path())
        .output()
        .expect("add task");

    // Complete task 1
    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "task", "complete", "TNDM-CMP1", "1", "--json"])
        .current_dir(repo_root.path())
        .output()
        .expect("complete task");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // List tasks and verify
    let list = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "task", "list", "TNDM-CMP1", "--json"])
        .current_dir(repo_root.path())
        .output()
        .expect("list tasks");
    let tasks: Vec<serde_json::Value> =
        serde_json::from_str(&String::from_utf8(list.stdout).unwrap()).unwrap();
    assert_eq!(tasks[0]["status"], "done");
}

#[test]
#[allow(clippy::disallowed_methods)]
fn task_complete_twice_is_idempotent() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");
    create_test_ticket(repo_root.path(), "TNDM-CMP2X", "Complete twice test");

    Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args([
            "ticket",
            "task",
            "add",
            "TNDM-CMP2X",
            "--title",
            "Do me twice",
        ])
        .current_dir(repo_root.path())
        .output()
        .expect("add task");

    // First complete - should succeed
    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "task", "complete", "TNDM-CMP2X", "1"])
        .current_dir(repo_root.path())
        .output()
        .expect("first complete");
    assert!(
        output.status.success(),
        "first complete failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Second complete - should also succeed (idempotent)
    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "task", "complete", "TNDM-CMP2X", "1"])
        .current_dir(repo_root.path())
        .output()
        .expect("second complete");
    assert!(
        output.status.success(),
        "second complete failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify task is still done after second complete
    let list = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "task", "list", "TNDM-CMP2X", "--json"])
        .current_dir(repo_root.path())
        .output()
        .expect("list tasks");
    let tasks: Vec<serde_json::Value> =
        serde_json::from_str(&String::from_utf8(list.stdout).unwrap()).unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0]["status"], "done");
}

#[test]
#[allow(clippy::disallowed_methods)]
fn task_complete_nonexistent_fails() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");
    create_test_ticket(repo_root.path(), "TNDM-CMPNF", "No tasks");

    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "task", "complete", "TNDM-CMPNF", "99"])
        .current_dir(repo_root.path())
        .output()
        .expect("complete nonexistent");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("task 99 not found"), "stderr was: {stderr}");
}

#[test]
#[allow(clippy::disallowed_methods)]
fn task_remove_deletes_task() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");
    create_test_ticket(repo_root.path(), "TNDM-RMV1", "Remove task test");

    Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "task", "add", "TNDM-RMV1", "--title", "Task 1"])
        .current_dir(repo_root.path())
        .output()
        .expect("add task 1");
    Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "task", "add", "TNDM-RMV1", "--title", "Task 2"])
        .current_dir(repo_root.path())
        .output()
        .expect("add task 2");

    // Remove task 1
    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "task", "remove", "TNDM-RMV1", "1", "--json"])
        .current_dir(repo_root.path())
        .output()
        .expect("remove task 1");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // List remaining, only task 2 should exist
    let list = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "task", "list", "TNDM-RMV1", "--json"])
        .current_dir(repo_root.path())
        .output()
        .expect("list tasks");
    let tasks: Vec<serde_json::Value> =
        serde_json::from_str(&String::from_utf8(list.stdout).unwrap()).unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0]["number"], 2);
    assert_eq!(tasks[0]["title"], "Task 2");
}

#[test]
#[allow(clippy::disallowed_methods)]
fn task_edit_updates_fields() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");
    create_test_ticket(repo_root.path(), "TNDM-EDT1", "Edit task test");

    Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "task", "add", "TNDM-EDT1", "--title", "Old title"])
        .current_dir(repo_root.path())
        .output()
        .expect("add task");

    // Edit task 1 — change title
    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args([
            "ticket",
            "task",
            "edit",
            "TNDM-EDT1",
            "1",
            "--title",
            "New title",
            "--json",
        ])
        .current_dir(repo_root.path())
        .output()
        .expect("edit task");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // List tasks and verify
    let list = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "task", "list", "TNDM-EDT1", "--json"])
        .current_dir(repo_root.path())
        .output()
        .expect("list tasks");
    let tasks: Vec<serde_json::Value> =
        serde_json::from_str(&String::from_utf8(list.stdout).unwrap()).unwrap();
    assert_eq!(tasks[0]["title"], "New title");
}

#[test]
#[allow(clippy::disallowed_methods)]
fn task_set_bulk_replace() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");
    create_test_ticket(repo_root.path(), "TNDM-SET1", "Bulk set test");

    // Add a legacy task
    Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args([
            "ticket",
            "task",
            "add",
            "TNDM-SET1",
            "--title",
            "Will be replaced",
        ])
        .current_dir(repo_root.path())
        .output()
        .expect("add task");

    // Bulk replace with set
    let tasks_json = r#"[{"number":10,"title":"New A","status":"todo","files":["src/lib.rs","tests/lib.rs"],"verification":"cargo test","notes":"Covers core path"},{"number":20,"title":"New B","status":"done"}]"#;
    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args([
            "ticket",
            "task",
            "set",
            "TNDM-SET1",
            "--tasks",
            tasks_json,
            "--json",
        ])
        .current_dir(repo_root.path())
        .output()
        .expect("set tasks");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // List tasks
    let list = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "task", "list", "TNDM-SET1", "--json"])
        .current_dir(repo_root.path())
        .output()
        .expect("list tasks");
    let tasks: Vec<serde_json::Value> =
        serde_json::from_str(&String::from_utf8(list.stdout).unwrap()).unwrap();
    assert_eq!(tasks.len(), 2);
    assert_eq!(tasks[0]["number"], 10);
    assert_eq!(tasks[0]["title"], "New A");
    assert_eq!(tasks[0]["status"], "todo");
    assert_eq!(tasks[1]["number"], 20);
    assert_eq!(tasks[1]["title"], "New B");
    assert_eq!(tasks[1]["status"], "done");
}

#[test]
#[allow(clippy::disallowed_methods)]
fn task_set_empty_clears() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");
    create_test_ticket(repo_root.path(), "TNDM-SETCLR", "Clear tasks");

    Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args([
            "ticket",
            "task",
            "add",
            "TNDM-SETCLR",
            "--title",
            "To clear",
        ])
        .current_dir(repo_root.path())
        .output()
        .expect("add task");

    // Set empty
    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args([
            "ticket",
            "task",
            "set",
            "TNDM-SETCLR",
            "--tasks",
            "[]",
            "--json",
        ])
        .current_dir(repo_root.path())
        .output()
        .expect("clear tasks via set");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify empty
    let list = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "task", "list", "TNDM-SETCLR", "--json"])
        .current_dir(repo_root.path())
        .output()
        .expect("list tasks");
    let tasks: Vec<serde_json::Value> =
        serde_json::from_str(&String::from_utf8(list.stdout).unwrap()).unwrap();
    assert!(tasks.is_empty(), "tasks should be empty after clearing");
}

#[test]
#[allow(clippy::disallowed_methods)]
fn task_set_duplicate_numbers_fails() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");
    create_test_ticket(repo_root.path(), "TNDM-SETDUP", "Dup task numbers");

    let tasks_json = r#"[{"number":1,"title":"First","status":"todo"},{"number":1,"title":"Duplicate","status":"todo"}]"#;
    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args([
            "ticket",
            "task",
            "set",
            "TNDM-SETDUP",
            "--tasks",
            tasks_json,
        ])
        .current_dir(repo_root.path())
        .output()
        .expect("set duplicate tasks");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("duplicate task number"),
        "stderr was: {stderr}"
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn task_add_rejects_empty_title() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");
    create_test_ticket(repo_root.path(), "TNDM-EMPTY", "Empty title test");

    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "task", "add", "TNDM-EMPTY", "--title", ""])
        .current_dir(repo_root.path())
        .output()
        .expect("add task with empty title");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("task title must not be empty"),
        "stderr was: {stderr}"
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn task_set_auto_creates_detail_docs() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");
    create_test_ticket(
        repo_root.path(),
        "TNDM-DETSET",
        "Task set detail doc auto-create",
    );

    let tasks_json = r#"[{"number":1,"title":"Task with auto doc","status":"todo"}]"#;
    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args([
            "ticket",
            "task",
            "set",
            "TNDM-DETSET",
            "--tasks",
            tasks_json,
        ])
        .current_dir(repo_root.path())
        .output()
        .expect("set tasks without explicit detail path");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let ticket_dir = repo_root
        .path()
        .join(".tndm")
        .join("tickets")
        .join("TNDM-DETSET");
    assert!(
        ticket_dir.join("tasks").join("task-01.md").is_file(),
        "task_set should auto-create the canonical task detail doc"
    );

    let meta_text = fs::read_to_string(ticket_dir.join("meta.toml")).expect("read meta.toml");
    assert!(
        meta_text.contains("name = \"task-01\""),
        "meta.toml should register the task-01 doc: {meta_text}"
    );
    assert!(
        meta_text.contains("path = \"tasks/task-01.md\""),
        "meta.toml should register the task detail doc path: {meta_text}"
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn task_edit_rejects_replaced_args() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");
    create_test_ticket(repo_root.path(), "TNDM-EDCLR", "Edit clear test");

    Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args([
            "ticket",
            "task",
            "add",
            "TNDM-EDCLR",
            "--title",
            "Test task",
        ])
        .current_dir(repo_root.path())
        .output()
        .expect("add task");

    // Verify --file is rejected on edit
    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args([
            "ticket",
            "task",
            "edit",
            "TNDM-EDCLR",
            "1",
            "--file",
            "src/main.rs",
        ])
        .current_dir(repo_root.path())
        .output()
        .expect("edit with removed --file flag");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("unexpected argument") || stderr.contains("error"),
        "stderr was: {stderr}"
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn task_detail_ensure_creates_and_links_canonical_doc() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");
    create_test_ticket(repo_root.path(), "TNDM-DETENS", "Ensure detail doc test");

    Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args([
            "ticket",
            "task",
            "add",
            "TNDM-DETENS",
            "--title",
            "Detailed task",
        ])
        .current_dir(repo_root.path())
        .output()
        .expect("add task");

    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args([
            "ticket",
            "task",
            "detail",
            "ensure",
            "TNDM-DETENS",
            "1",
            "--json",
        ])
        .current_dir(repo_root.path())
        .output()
        .expect("ensure task detail");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let ticket_dir = repo_root
        .path()
        .join(".tndm")
        .join("tickets")
        .join("TNDM-DETENS");
    assert!(
        ticket_dir.join("tasks").join("task-01.md").is_file(),
        "canonical task detail doc should exist"
    );

    let meta_text = fs::read_to_string(ticket_dir.join("meta.toml")).expect("read meta.toml");
    assert!(
        meta_text.contains("name = \"task-01\""),
        "meta.toml should register task-01: {meta_text}"
    );
    assert!(
        meta_text.contains("path = \"tasks/task-01.md\""),
        "meta.toml should register the canonical task detail path: {meta_text}"
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn task_detail_ensure_is_idempotent() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");
    create_test_ticket(
        repo_root.path(),
        "TNDM-DETIDEM",
        "Idempotent detail doc test",
    );

    Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args([
            "ticket",
            "task",
            "add",
            "TNDM-DETIDEM",
            "--title",
            "Detailed task",
        ])
        .current_dir(repo_root.path())
        .output()
        .expect("add task");

    for _ in 0..2 {
        let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
            .args([
                "ticket",
                "task",
                "detail",
                "ensure",
                "TNDM-DETIDEM",
                "1",
                "--json",
            ])
            .current_dir(repo_root.path())
            .output()
            .expect("ensure task detail");
        assert!(
            output.status.success(),
            "stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let meta_text = fs::read_to_string(
        repo_root
            .path()
            .join(".tndm")
            .join("tickets")
            .join("TNDM-DETIDEM")
            .join("meta.toml"),
    )
    .expect("read meta.toml");
    assert_eq!(meta_text.matches("name = \"task-01\"").count(), 1);
    assert_eq!(meta_text.matches("path = \"tasks/task-01.md\"").count(), 1);
}

#[test]
#[allow(clippy::disallowed_methods)]
fn task_remove_prunes_orphaned_canonical_detail_doc() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");
    create_test_ticket(repo_root.path(), "TNDM-DETREM", "Remove detail doc test");

    Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args([
            "ticket",
            "task",
            "add",
            "TNDM-DETREM",
            "--title",
            "Old task",
        ])
        .current_dir(repo_root.path())
        .output()
        .expect("add task");
    Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "task", "detail", "ensure", "TNDM-DETREM", "1"])
        .current_dir(repo_root.path())
        .output()
        .expect("ensure task detail");

    let ticket_dir = repo_root
        .path()
        .join(".tndm")
        .join("tickets")
        .join("TNDM-DETREM");
    fs::write(
        ticket_dir.join("tasks").join("task-01.md"),
        "# Old task\n\nOld detail\n",
    )
    .expect("overwrite canonical task detail doc");
    Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "sync", "TNDM-DETREM"])
        .current_dir(repo_root.path())
        .output()
        .expect("sync task detail doc");

    let remove = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "task", "remove", "TNDM-DETREM", "1"])
        .current_dir(repo_root.path())
        .output()
        .expect("remove task");
    assert!(
        remove.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&remove.stderr)
    );

    assert!(
        !ticket_dir.join("tasks").join("task-01.md").exists(),
        "removing the task should prune the canonical task detail doc"
    );

    let meta_text = fs::read_to_string(ticket_dir.join("meta.toml")).expect("read meta.toml");
    assert!(
        !meta_text.contains("name = \"task-01\""),
        "meta.toml should no longer register task-01 after removal: {meta_text}"
    );
    let state_text = fs::read_to_string(ticket_dir.join("state.toml")).expect("read state.toml");
    assert!(
        !state_text.contains("task-01 ="),
        "state.toml should no longer fingerprint task-01 after removal: {state_text}"
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn task_set_persists_detail_doc_on_same_number_reuse() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");
    create_test_ticket(
        repo_root.path(),
        "TNDM-DETSETP",
        "Task set prune detail doc test",
    );

    Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args([
            "ticket",
            "task",
            "add",
            "TNDM-DETSETP",
            "--title",
            "Old task",
        ])
        .current_dir(repo_root.path())
        .output()
        .expect("add task");
    Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "task", "detail", "ensure", "TNDM-DETSETP", "1"])
        .current_dir(repo_root.path())
        .output()
        .expect("ensure task detail");

    let ticket_dir = repo_root
        .path()
        .join(".tndm")
        .join("tickets")
        .join("TNDM-DETSETP");
    fs::write(
        ticket_dir.join("tasks").join("task-01.md"),
        "# Old task\n\nOld detail\n",
    )
    .expect("overwrite canonical task detail doc");
    Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "sync", "TNDM-DETSETP"])
        .current_dir(repo_root.path())
        .output()
        .expect("sync task detail doc");

    let set = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args([
            "ticket",
            "task",
            "set",
            "TNDM-DETSETP",
            "--tasks",
            r#"[{"number":1,"title":"Replacement task","status":"todo"}]"#,
        ])
        .current_dir(repo_root.path())
        .output()
        .expect("replace tasks");
    assert!(
        set.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&set.stderr)
    );

    assert!(
        ticket_dir.join("tasks").join("task-01.md").exists(),
        "existing detail doc should persist when same task number is reused"
    );

    let meta_text = fs::read_to_string(ticket_dir.join("meta.toml")).expect("read meta.toml");
    assert!(
        meta_text.contains("name = \"task-01\""),
        "meta.toml should re-register task-01 for replacement task: {meta_text}"
    );
    let state_text = fs::read_to_string(ticket_dir.join("state.toml")).expect("read state.toml");
    assert!(
        state_text.contains("task-01 ="),
        "state.toml should fingerprint replacement task-01 doc: {state_text}"
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn task_edit_rejects_empty_title() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");
    create_test_ticket(repo_root.path(), "TNDM-EDEMP", "Edit empty title test");

    Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "task", "add", "TNDM-EDEMP", "--title", "Valid"])
        .current_dir(repo_root.path())
        .output()
        .expect("add task");

    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "task", "edit", "TNDM-EDEMP", "1", "--title", ""])
        .current_dir(repo_root.path())
        .output()
        .expect("edit with empty title");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("task title must not be empty"),
        "stderr was: {stderr}"
    );
}
