use std::fs;

use tandem_core::{
    ports::TicketStore,
    ticket::{NewTicket, TicketId, TicketMeta, TicketState, TicketStatus},
};
use tandem_storage::{FileTicketStore, discover_repo_root};

#[test]
#[allow(clippy::disallowed_methods)]
fn discover_repo_root_finds_git_dir() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let nested = repo_root.path().join("crates").join("tandem-storage");
    fs::create_dir_all(&nested).expect("create nested dir");

    let discovered = discover_repo_root(&nested).expect("discover repo root");

    assert_eq!(discovered, repo_root.path());
}

#[test]
#[allow(clippy::disallowed_methods)]
fn discover_repo_root_errors_when_no_repo_markers() {
    let start = tempfile::tempdir().expect("tempdir");

    let error = discover_repo_root(start.path()).expect_err("discover should fail");

    assert_eq!(
        error.to_string(),
        "no repository markers found (.tndm or .git)"
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn create_ticket_writes_expected_files() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let store = FileTicketStore::new(repo_root.path().to_path_buf());

    let id = TicketId::parse("TNDM-123").expect("valid ticket id");
    let meta = TicketMeta::new(id.clone(), "Implement create_ticket").expect("valid ticket meta");
    let mut state = TicketState::new("2026-03-03T12:00:00Z", 7).expect("valid ticket state");
    state.status = TicketStatus::InProgress;
    let content = "## Description\n\nWrite ticket files.\n".to_string();

    let ticket = NewTicket {
        meta: meta.clone(),
        state: state.clone(),
        content: content.clone(),
    };

    let created = store.create_ticket(ticket).expect("create ticket");

    assert_eq!(created.meta, meta);
    assert_eq!(created.state, state);
    assert_eq!(created.content, content);

    let ticket_dir = repo_root
        .path()
        .join(".tndm")
        .join("tickets")
        .join(id.as_str());

    let written_meta = fs::read_to_string(ticket_dir.join("meta.toml")).expect("read meta.toml");
    let written_state = fs::read_to_string(ticket_dir.join("state.toml")).expect("read state.toml");
    let written_content =
        fs::read_to_string(ticket_dir.join("content.md")).expect("read content.md");

    assert_eq!(written_meta, meta.to_canonical_toml());
    assert_eq!(written_state, state.to_canonical_toml());
    assert_eq!(written_content, content);
}
