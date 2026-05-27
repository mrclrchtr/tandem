#![allow(clippy::disallowed_types, unused_imports)]

mod common;

use common::*;
use std::fs;

#[test]
#[allow(clippy::disallowed_methods)]
fn task_add_creates_task_with_auto_number() {
    let repo = TestRepo::new();
    repo.create_ticket(Some("TNDM-ADDN1"), "Add task test");

    repo.run_assert(&[
        "ticket",
        "task",
        "add",
        "TNDM-ADDN1",
        "--title",
        "First task",
    ]);

    // List tasks and verify via separate call
    let tasks = repo.run_json(&["ticket", "task", "list", "TNDM-ADDN1"]);
    let tasks = tasks.as_array().expect("task list should be an array");
    assert_eq!(tasks[0]["number"], 1);
    assert_eq!(tasks[0]["title"], "First task");
    assert_eq!(tasks[0]["status"], "todo");
}

#[test]
#[allow(clippy::disallowed_methods)]
fn task_add_increments_number() {
    let repo = TestRepo::new();
    repo.create_ticket(Some("TNDM-ADDN2"), "Add two tasks");

    // Add first task
    repo.run_assert(&["ticket", "task", "add", "TNDM-ADDN2", "--title", "Task A"]);

    // Add second task
    repo.run_assert(&["ticket", "task", "add", "TNDM-ADDN2", "--title", "Task B"]);

    // List tasks with JSON
    let tasks = repo.run_json(&["ticket", "task", "list", "TNDM-ADDN2"]);
    let tasks = tasks.as_array().expect("task list should be an array");
    assert_eq!(tasks.len(), 2);
    assert_eq!(tasks[0]["number"], 1);
    assert_eq!(tasks[0]["title"], "Task A");
    assert_eq!(tasks[1]["number"], 2);
    assert_eq!(tasks[1]["title"], "Task B");
}

#[test]
#[allow(clippy::disallowed_methods)]
fn task_list_json_output() {
    let repo = TestRepo::new();
    repo.create_ticket(Some("TNDM-LST1"), "Task list test");

    // Add a task
    repo.run_assert(&["ticket", "task", "add", "TNDM-LST1", "--title", "List me"]);

    // List --json
    let tasks = repo.run_json(&["ticket", "task", "list", "TNDM-LST1"]);
    let tasks = tasks.as_array().expect("task list should be an array");
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0]["number"], 1);
    assert_eq!(tasks[0]["title"], "List me");
    assert_eq!(tasks[0]["status"], "todo");
}

#[test]
#[allow(clippy::disallowed_methods)]
fn task_complete_marks_task_done() {
    let repo = TestRepo::new();
    repo.create_ticket(Some("TNDM-CMP1"), "Complete test");

    repo.run_assert(&["ticket", "task", "add", "TNDM-CMP1", "--title", "Finish me"]);

    // Complete task 1
    repo.run_assert(&["ticket", "task", "complete", "TNDM-CMP1", "1"]);

    // List tasks and verify
    let tasks = repo.run_json(&["ticket", "task", "list", "TNDM-CMP1"]);
    let tasks = tasks.as_array().expect("task list should be an array");
    assert_eq!(tasks[0]["status"], "done");
}

#[test]
#[allow(clippy::disallowed_methods)]
fn task_complete_twice_is_idempotent() {
    let repo = TestRepo::new();
    repo.create_ticket(Some("TNDM-CMP2X"), "Complete twice test");

    repo.run_assert(&[
        "ticket",
        "task",
        "add",
        "TNDM-CMP2X",
        "--title",
        "Do me twice",
    ]);

    // First complete - should succeed
    repo.run_assert(&["ticket", "task", "complete", "TNDM-CMP2X", "1"]);

    // Second complete - should also succeed (idempotent)
    repo.run_assert(&["ticket", "task", "complete", "TNDM-CMP2X", "1"]);

    // Verify task is still done after second complete
    let tasks = repo.run_json(&["ticket", "task", "list", "TNDM-CMP2X"]);
    let tasks = tasks.as_array().expect("task list should be an array");
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0]["status"], "done");
}

#[test]
#[allow(clippy::disallowed_methods)]
fn task_complete_nonexistent_fails() {
    let repo = TestRepo::new();
    repo.create_ticket(Some("TNDM-CMPNF"), "No tasks");

    let output = repo.run(&["ticket", "task", "complete", "TNDM-CMPNF", "99"]);

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("task 99 not found"), "stderr was: {stderr}");
}

#[test]
#[allow(clippy::disallowed_methods)]
fn task_remove_deletes_task() {
    let repo = TestRepo::new();
    repo.create_ticket(Some("TNDM-RMV1"), "Remove task test");

    repo.run_assert(&["ticket", "task", "add", "TNDM-RMV1", "--title", "Task 1"]);
    repo.run_assert(&["ticket", "task", "add", "TNDM-RMV1", "--title", "Task 2"]);

    // Remove task 1
    repo.run_assert(&["ticket", "task", "remove", "TNDM-RMV1", "1"]);

    // List remaining, only task 2 should exist
    let tasks = repo.run_json(&["ticket", "task", "list", "TNDM-RMV1"]);
    let tasks = tasks.as_array().expect("task list should be an array");
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0]["number"], 2);
    assert_eq!(tasks[0]["title"], "Task 2");
}

#[test]
#[allow(clippy::disallowed_methods)]
fn task_edit_updates_fields() {
    let repo = TestRepo::new();
    repo.create_ticket(Some("TNDM-EDT1"), "Edit task test");

    repo.run_assert(&["ticket", "task", "add", "TNDM-EDT1", "--title", "Old title"]);

    // Edit task 1 — change title
    repo.run_assert(&[
        "ticket",
        "task",
        "edit",
        "TNDM-EDT1",
        "1",
        "--title",
        "New title",
    ]);

    // List tasks and verify
    let tasks = repo.run_json(&["ticket", "task", "list", "TNDM-EDT1"]);
    let tasks = tasks.as_array().expect("task list should be an array");
    assert_eq!(tasks[0]["title"], "New title");
}

#[test]
#[allow(clippy::disallowed_methods)]
fn task_set_bulk_replace() {
    let repo = TestRepo::new();
    repo.create_ticket(Some("TNDM-SET1"), "Bulk set test");

    // Add a legacy task
    repo.run_assert(&[
        "ticket",
        "task",
        "add",
        "TNDM-SET1",
        "--title",
        "Will be replaced",
    ]);

    // Bulk replace with set
    let tasks_json = r#"[{"number":10,"title":"New A","status":"todo","files":["src/lib.rs","tests/lib.rs"],"verification":"cargo test","notes":"Covers core path"},{"number":20,"title":"New B","status":"done"}]"#;
    repo.run_assert(&["ticket", "task", "set", "TNDM-SET1", "--tasks", tasks_json]);

    // List tasks
    let tasks = repo.run_json(&["ticket", "task", "list", "TNDM-SET1"]);
    let tasks = tasks.as_array().expect("task list should be an array");
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
    let repo = TestRepo::new();
    repo.create_ticket(Some("TNDM-SETCLR"), "Clear tasks");

    repo.run_assert(&[
        "ticket",
        "task",
        "add",
        "TNDM-SETCLR",
        "--title",
        "To clear",
    ]);

    // Set empty
    repo.run_assert(&["ticket", "task", "set", "TNDM-SETCLR", "--tasks", "[]"]);

    // Verify empty
    let tasks = repo.run_json(&["ticket", "task", "list", "TNDM-SETCLR"]);
    let tasks = tasks.as_array().expect("task list should be an array");
    assert!(tasks.is_empty(), "tasks should be empty after clearing");
}

#[test]
#[allow(clippy::disallowed_methods)]
fn task_set_duplicate_numbers_fails() {
    let repo = TestRepo::new();
    repo.create_ticket(Some("TNDM-SETDUP"), "Dup task numbers");

    let tasks_json = r#"[{"number":1,"title":"First","status":"todo"},{"number":1,"title":"Duplicate","status":"todo"}]"#;
    let output = repo.run(&[
        "ticket",
        "task",
        "set",
        "TNDM-SETDUP",
        "--tasks",
        tasks_json,
    ]);

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
    let repo = TestRepo::new();
    repo.create_ticket(Some("TNDM-EMPTY"), "Empty title test");

    let output = repo.run(&["ticket", "task", "add", "TNDM-EMPTY", "--title", ""]);

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
    let repo = TestRepo::new();
    repo.create_ticket(Some("TNDM-DETSET"), "Task set detail doc auto-create");

    let tasks_json = r#"[{"number":1,"title":"Task with auto doc","status":"todo"}]"#;
    repo.run_assert(&[
        "ticket",
        "task",
        "set",
        "TNDM-DETSET",
        "--tasks",
        tasks_json,
    ]);

    let ticket_dir = repo
        .path()
        .join(".tndm")
        .join("tickets")
        .join("TNDM-DETSET");
    assert!(
        ticket_dir.join("tasks").join("task-01.md").is_file(),
        "task_set should auto-create the canonical task detail doc"
    );

    let meta_text = repo.read_ticket_file("TNDM-DETSET", "meta.toml");
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
    let repo = TestRepo::new();
    repo.create_ticket(Some("TNDM-EDCLR"), "Edit clear test");

    repo.run_assert(&[
        "ticket",
        "task",
        "add",
        "TNDM-EDCLR",
        "--title",
        "Test task",
    ]);

    // Verify --file is rejected on edit
    let output = repo.run(&[
        "ticket",
        "task",
        "edit",
        "TNDM-EDCLR",
        "1",
        "--file",
        "src/main.rs",
    ]);

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
    let repo = TestRepo::new();
    repo.create_ticket(Some("TNDM-DETENS"), "Ensure detail doc test");

    repo.run_assert(&[
        "ticket",
        "task",
        "add",
        "TNDM-DETENS",
        "--title",
        "Detailed task",
    ]);

    repo.run_assert(&["ticket", "task", "detail", "ensure", "TNDM-DETENS", "1"]);

    let ticket_dir = repo
        .path()
        .join(".tndm")
        .join("tickets")
        .join("TNDM-DETENS");
    assert!(
        ticket_dir.join("tasks").join("task-01.md").is_file(),
        "canonical task detail doc should exist"
    );

    let meta_text = repo.read_ticket_file("TNDM-DETENS", "meta.toml");
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
    let repo = TestRepo::new();
    repo.create_ticket(Some("TNDM-DETIDEM"), "Idempotent detail doc test");

    repo.run_assert(&[
        "ticket",
        "task",
        "add",
        "TNDM-DETIDEM",
        "--title",
        "Detailed task",
    ]);

    for _ in 0..2 {
        repo.run_assert(&["ticket", "task", "detail", "ensure", "TNDM-DETIDEM", "1"]);
    }

    let meta_text = repo.read_ticket_file("TNDM-DETIDEM", "meta.toml");
    assert_eq!(meta_text.matches("name = \"task-01\"").count(), 1);
    assert_eq!(meta_text.matches("path = \"tasks/task-01.md\"").count(), 1);
}

#[test]
#[allow(clippy::disallowed_methods)]
fn task_remove_prunes_orphaned_canonical_detail_doc() {
    let repo = TestRepo::new();
    repo.create_ticket(Some("TNDM-DETREM"), "Remove detail doc test");

    repo.run_assert(&[
        "ticket",
        "task",
        "add",
        "TNDM-DETREM",
        "--title",
        "Old task",
    ]);
    repo.run_assert(&["ticket", "task", "detail", "ensure", "TNDM-DETREM", "1"]);

    let ticket_dir = repo
        .path()
        .join(".tndm")
        .join("tickets")
        .join("TNDM-DETREM");
    fs::write(
        ticket_dir.join("tasks").join("task-01.md"),
        "# Old task\n\nOld detail\n",
    )
    .expect("overwrite canonical task detail doc");
    repo.run_assert(&["ticket", "sync", "TNDM-DETREM"]);

    let remove = repo.run(&["ticket", "task", "remove", "TNDM-DETREM", "1"]);
    assert!(
        remove.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&remove.stderr)
    );

    assert!(
        !ticket_dir.join("tasks").join("task-01.md").exists(),
        "removing the task should prune the canonical task detail doc"
    );

    let meta_text = repo.read_ticket_file("TNDM-DETREM", "meta.toml");
    assert!(
        !meta_text.contains("name = \"task-01\""),
        "meta.toml should no longer register task-01 after removal: {meta_text}"
    );
    let state_text = repo.read_ticket_file("TNDM-DETREM", "state.toml");
    assert!(
        !state_text.contains("task-01 ="),
        "state.toml should no longer fingerprint task-01 after removal: {state_text}"
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn task_set_persists_detail_doc_on_same_number_reuse() {
    let repo = TestRepo::new();
    repo.create_ticket(Some("TNDM-DETSETP"), "Task set prune detail doc test");

    repo.run_assert(&[
        "ticket",
        "task",
        "add",
        "TNDM-DETSETP",
        "--title",
        "Old task",
    ]);
    repo.run_assert(&["ticket", "task", "detail", "ensure", "TNDM-DETSETP", "1"]);

    let ticket_dir = repo
        .path()
        .join(".tndm")
        .join("tickets")
        .join("TNDM-DETSETP");
    fs::write(
        ticket_dir.join("tasks").join("task-01.md"),
        "# Old task\n\nOld detail\n",
    )
    .expect("overwrite canonical task detail doc");
    repo.run_assert(&["ticket", "sync", "TNDM-DETSETP"]);

    let set = repo.run(&[
        "ticket",
        "task",
        "set",
        "TNDM-DETSETP",
        "--tasks",
        r#"[{"number":1,"title":"Replacement task","status":"todo"}]"#,
    ]);
    assert!(
        set.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&set.stderr)
    );

    assert!(
        ticket_dir.join("tasks").join("task-01.md").exists(),
        "existing detail doc should persist when same task number is reused"
    );

    let meta_text = repo.read_ticket_file("TNDM-DETSETP", "meta.toml");
    assert!(
        meta_text.contains("name = \"task-01\""),
        "meta.toml should re-register task-01 for replacement task: {meta_text}"
    );
    let state_text = repo.read_ticket_file("TNDM-DETSETP", "state.toml");
    assert!(
        state_text.contains("task-01 ="),
        "state.toml should fingerprint replacement task-01 doc: {state_text}"
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn task_edit_rejects_empty_title() {
    let repo = TestRepo::new();
    repo.create_ticket(Some("TNDM-EDEMP"), "Edit empty title test");

    repo.run_assert(&["ticket", "task", "add", "TNDM-EDEMP", "--title", "Valid"]);

    let output = repo.run(&["ticket", "task", "edit", "TNDM-EDEMP", "1", "--title", ""]);

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("task title must not be empty"),
        "stderr was: {stderr}"
    );
}
