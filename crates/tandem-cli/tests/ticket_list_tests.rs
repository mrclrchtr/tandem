#![allow(clippy::disallowed_types, unused_imports)]

mod common;

use common::*;

#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_list_prints_sorted_rows() {
    let repo = TestRepo::new();

    repo.create_ticket(Some("TNDM-2"), "Second ticket");
    repo.create_ticket(Some("TNDM-1"), "First ticket");

    let stdout = repo.run_assert(&["ticket", "list"]);

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3, "unexpected list output: {stdout}");
    assert!(lines[0].contains("ID"));
    assert!(lines[0].contains("STATUS"));
    assert!(lines[0].contains("TITLE"));
    assert!(lines[1].contains("TNDM-1"));
    assert!(lines[1].contains("First ticket"));
    assert!(lines[2].contains("TNDM-2"));
    assert!(lines[2].contains("Second ticket"));
}

#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_list_hides_done_tickets_by_default() {
    let repo = TestRepo::new();

    // Create two tickets
    for (id, title) in [("TNDM-1", "Open ticket"), ("TNDM-2", "Done ticket")] {
        repo.create_ticket(Some(id), title);
    }

    // Mark TNDM-2 as done
    repo.run_assert(&["ticket", "update", "TNDM-2", "--status", "done"]);

    // Default list should hide done tickets
    let stdout = repo.run_assert(&["ticket", "list"]);
    assert!(stdout.contains("TNDM-1"), "open ticket should appear");
    assert!(!stdout.contains("TNDM-2"), "done ticket should be hidden");

    // --all should include done tickets
    let stdout = repo.run_assert(&["ticket", "list", "--all"]);
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
    let repo = TestRepo::new();

    for (id, title) in [("TNDM-1", "Open"), ("TNDM-2", "Done")] {
        repo.create_ticket(Some(id), title);
    }

    repo.run_assert(&["ticket", "update", "TNDM-2", "--status", "done"]);

    // Default JSON list should hide done tickets
    let json = repo.run_json(&["ticket", "list"]);
    let tickets = json["tickets"].as_array().expect("tickets array");
    assert_eq!(tickets.len(), 1);
    assert_eq!(tickets[0]["id"], "TNDM-1");

    // --all --json should include done tickets
    let json = repo.run_json(&["ticket", "list", "--all"]);
    let tickets = json["tickets"].as_array().expect("tickets array");
    assert_eq!(tickets.len(), 2);
}

#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_list_json_outputs_schema_versioned_array() {
    let repo = TestRepo::new();

    repo.create_ticket(Some("TNDM-1"), "First");
    repo.create_ticket(Some("TNDM-2"), "Second");

    let json = repo.run_json(&["ticket", "list"]);
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
fn ticket_list_sorts_by_priority_then_id() {
    let repo = TestRepo::new();

    // Create tickets and set different priorities
    for (id, title, prio) in [
        ("TNDM-1", "Low prio", "p3"),
        ("TNDM-2", "High prio", "p0"),
        ("TNDM-3", "Also high prio", "p0"),
        ("TNDM-4", "Medium prio", "p2"),
    ] {
        repo.create_ticket(Some(id), title);
        repo.run_assert(&["ticket", "update", id, "--priority", prio]);
    }

    let stdout = repo.run_assert(&["ticket", "list"]);

    let ids: Vec<&str> = stdout
        .lines()
        .skip(1)
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            line.split_whitespace()
                .next()
                .expect("ticket row should start with id")
        })
        .collect();
    assert_eq!(ids, vec!["TNDM-2", "TNDM-3", "TNDM-4", "TNDM-1"]);
}

#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_list_json_empty_produces_empty_array() {
    let repo = TestRepo::new();

    let json = repo.run_json(&["ticket", "list"]);
    assert_eq!(json["schema_version"], 1);
    assert_eq!(json["tickets"], serde_json::json!([]));
}

#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_list_filters_by_definition_tags_in_plain_text() {
    let repo = TestRepo::new();

    for (id, title, tags) in [
        ("TNDM-READY", "Ready ticket", "definition:ready"),
        ("TNDM-QUES", "Questions ticket", "definition:questions"),
        ("TNDM-UNKW", "Unknown ticket", ""),
    ] {
        repo.create_ticket(Some(id), title);
        // Set tags via update (create_ticket with empty tags works via --tags param)
        repo.run_assert(&["ticket", "update", id, "--tags", tags]);
    }

    let ready_stdout = repo.run_assert(&["ticket", "list", "--definition", "ready"]);
    assert!(ready_stdout.contains("TNDM-READY"));
    assert!(!ready_stdout.contains("TNDM-QUES"));
    assert!(!ready_stdout.contains("TNDM-UNKW"));

    let questions_stdout = repo.run_assert(&["ticket", "list", "--definition", "questions"]);
    assert!(questions_stdout.contains("TNDM-QUES"));
    assert!(!questions_stdout.contains("TNDM-READY"));
    assert!(!questions_stdout.contains("TNDM-UNKW"));

    let unknown_stdout = repo.run_assert(&["ticket", "list", "--definition", "unknown"]);
    assert!(unknown_stdout.contains("TNDM-UNKW"));
    assert!(!unknown_stdout.contains("TNDM-READY"));
    assert!(!unknown_stdout.contains("TNDM-QUES"));
}

#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_list_filters_by_definition_tags_in_json() {
    let repo = TestRepo::new();

    for (id, title, tags) in [
        ("TNDM-READY", "Ready ticket", "definition:ready"),
        ("TNDM-QUES", "Questions ticket", "definition:questions"),
        ("TNDM-UNKW", "Unknown ticket", ""),
    ] {
        repo.create_ticket(Some(id), title);
        repo.run_assert(&["ticket", "update", id, "--tags", tags]);
    }

    let json = repo.run_json(&["ticket", "list", "--definition", "questions"]);
    let tickets = json["tickets"]
        .as_array()
        .expect("tickets should be an array");
    assert_eq!(tickets.len(), 1);
    assert_eq!(tickets[0]["id"], "TNDM-QUES");
    assert_eq!(
        tickets[0]["tags"],
        serde_json::json!(["definition:questions"])
    );
}
