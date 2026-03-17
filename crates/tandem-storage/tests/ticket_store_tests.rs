use std::fs;

use tandem_core::{
    ports::TicketStore,
    ticket::{NewTicket, TicketId, TicketMeta, TicketPriority, TicketStatus, TicketType},
};
use tandem_storage::{FileTicketStore, discover_repo_root};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

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
fn discover_repo_root_prefers_git_over_tndm_in_subdirectory() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let child = repo_root.path().join("sub");
    fs::create_dir_all(child.join(".tndm")).expect("create .tndm dir in child");

    let discovered = discover_repo_root(&child).expect("discover repo root");

    assert_eq!(discovered, repo_root.path());
}

#[test]
#[allow(clippy::disallowed_methods)]
fn create_ticket_writes_expected_files() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let store = FileTicketStore::new(repo_root.path().to_path_buf());

    let id = TicketId::parse("TNDM-123").expect("valid ticket id");
    let meta = TicketMeta::new(id.clone(), "Implement create_ticket").expect("valid ticket meta");
    let content = "## Description\n\nWrite ticket files.\n".to_string();

    let ticket = NewTicket {
        meta: meta.clone(),
        content: content.clone(),
    };

    let created = store.create_ticket(ticket).expect("create ticket");

    assert_eq!(created.meta, meta);
    assert_eq!(created.state.status, TicketStatus::Todo);
    assert_eq!(created.state.revision, 1);
    OffsetDateTime::parse(&created.state.updated_at, &Rfc3339)
        .expect("updated_at should parse as RFC3339");
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
    assert_eq!(written_state, created.state.to_canonical_toml());
    assert_eq!(written_content, content);
}

#[test]
#[allow(clippy::disallowed_methods)]
fn load_ticket_roundtrips_created_ticket() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let store = FileTicketStore::new(repo_root.path().to_path_buf());

    let id = TicketId::parse("TNDM-456").expect("valid ticket id");
    let depends_on_id = TicketId::parse("TNDM-123").expect("valid dependency ticket id");

    let mut meta = TicketMeta::new(id.clone(), "Roundtrip load_ticket").expect("valid ticket meta");
    meta.ticket_type = TicketType::Feature;
    meta.priority = TicketPriority::P1;
    meta.depends_on = vec![depends_on_id];
    meta.tags = vec!["backend".to_string(), "storage".to_string()];

    let content = "## Description\n\nRoundtrip ticket content.\n".to_string();

    let ticket = NewTicket {
        meta: meta.clone(),
        content: content.clone(),
    };

    let created = store.create_ticket(ticket).expect("create ticket");

    let loaded = store.load_ticket(&id).expect("load ticket");

    assert_eq!(loaded.meta, meta);
    assert_eq!(loaded.state, created.state);
    assert_eq!(loaded.content, content);
}

#[test]
#[allow(clippy::disallowed_methods)]
fn load_ticket_defaults_missing_type_and_priority_from_meta_toml() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let id = TicketId::parse("TNDM-457").expect("valid ticket id");
    let ticket_dir = repo_root
        .path()
        .join(".tndm")
        .join("tickets")
        .join(id.as_str());
    fs::create_dir_all(&ticket_dir).expect("create ticket dir");

    fs::write(
        ticket_dir.join("meta.toml"),
        concat!(
            "schema_version = 1\n",
            "id = \"TNDM-457\"\n",
            "title = \"Default stored meta\"\n",
            "\n",
            "depends_on = []\n",
            "tags = []\n",
        ),
    )
    .expect("write meta.toml");
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
    fs::write(ticket_dir.join("content.md"), "stored body\n").expect("write content.md");

    let store = FileTicketStore::new(repo_root.path().to_path_buf());
    let loaded = store.load_ticket(&id).expect("load ticket");

    assert_eq!(loaded.meta.id, id);
    assert_eq!(loaded.meta.title, "Default stored meta");
    assert_eq!(loaded.meta.ticket_type, TicketType::Task);
    assert_eq!(loaded.meta.priority, TicketPriority::P2);
    assert!(loaded.meta.depends_on.is_empty());
    assert!(loaded.meta.tags.is_empty());
}

#[test]
#[allow(clippy::disallowed_methods)]
fn load_ticket_parses_non_default_state_from_state_toml() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let id = TicketId::parse("TNDM-789").expect("valid ticket id");
    let ticket_dir = repo_root
        .path()
        .join(".tndm")
        .join("tickets")
        .join(id.as_str());
    fs::create_dir_all(&ticket_dir).expect("create ticket dir");

    fs::write(
        ticket_dir.join("meta.toml"),
        concat!(
            "schema_version = 1\n",
            "id = \"TNDM-789\"\n",
            "title = \"Parse stored state\"\n",
            "\n",
            "type = \"bug\"\n",
            "priority = \"p0\"\n",
            "\n",
            "depends_on = []\n",
            "tags = [\"urgent\"]\n",
        ),
    )
    .expect("write meta.toml");
    fs::write(
        ticket_dir.join("state.toml"),
        concat!(
            "schema_version = 1\n",
            "status = \"blocked\"\n",
            "updated_at = \"2026-03-03T12:34:56Z\"\n",
            "revision = 7\n",
        ),
    )
    .expect("write state.toml");
    fs::write(ticket_dir.join("content.md"), "stored body\n").expect("write content.md");

    let store = FileTicketStore::new(repo_root.path().to_path_buf());
    let loaded = store.load_ticket(&id).expect("load ticket");

    assert_eq!(loaded.state.status, TicketStatus::Blocked);
    assert_eq!(loaded.state.updated_at, "2026-03-03T12:34:56Z");
    assert_eq!(loaded.state.revision, 7);
}

#[test]
#[allow(clippy::disallowed_methods)]
fn list_ticket_ids_sorts_by_id() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let store = FileTicketStore::new(repo_root.path().to_path_buf());

    let id_b = TicketId::parse("TNDM-2").expect("valid ticket id");
    let id_a = TicketId::parse("TNDM-1").expect("valid ticket id");

    let meta_b = TicketMeta::new(id_b.clone(), "Second ticket").expect("valid ticket meta");
    let meta_a = TicketMeta::new(id_a.clone(), "First ticket").expect("valid ticket meta");

    store
        .create_ticket(NewTicket {
            meta: meta_b,
            content: "## Description\n\nSecond.\n".to_string(),
        })
        .expect("create second ticket");

    store
        .create_ticket(NewTicket {
            meta: meta_a,
            content: "## Description\n\nFirst.\n".to_string(),
        })
        .expect("create first ticket");

    let ids = store.list_ticket_ids().expect("list ticket ids");

    assert_eq!(ids, vec![id_a, id_b]);
}

#[test]
#[allow(clippy::disallowed_methods)]
fn update_ticket_persists_changed_fields() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let store = FileTicketStore::new(repo_root.path().to_path_buf());

    let id = TicketId::parse("TNDM-UPD1").expect("valid ticket id");
    let meta = TicketMeta::new(id.clone(), "Original title").expect("valid ticket meta");
    let content = "original content\n".to_string();

    let created = store
        .create_ticket(NewTicket { meta, content })
        .expect("create ticket");

    let mut ticket = created;
    ticket.state.status = TicketStatus::InProgress;
    ticket.meta.priority = TicketPriority::P0;
    ticket.meta.title = "Updated title".to_string();
    ticket.meta.tags = vec!["backend".to_string(), "urgent".to_string()];
    ticket.state.revision = 2;
    ticket.state.updated_at = OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .expect("format timestamp");

    store.update_ticket(&ticket).expect("update ticket");

    let loaded = store.load_ticket(&id).expect("load updated ticket");

    assert_eq!(loaded.state.status, TicketStatus::InProgress);
    assert_eq!(loaded.meta.priority, TicketPriority::P0);
    assert_eq!(loaded.meta.title, "Updated title");
    assert_eq!(
        loaded.meta.tags,
        vec!["backend".to_string(), "urgent".to_string()]
    );
    assert_eq!(loaded.state.revision, 2);
}

#[test]
#[allow(clippy::disallowed_methods)]
fn update_ticket_fails_for_nonexistent_ticket() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let store = FileTicketStore::new(repo_root.path().to_path_buf());

    let id = TicketId::parse("TNDM-NOPE").expect("valid ticket id");
    let meta = TicketMeta::new(id, "Ghost ticket").expect("valid ticket meta");
    let state = tandem_core::ticket::TicketState::new("2026-03-03T10:00:00Z", 1)
        .expect("valid ticket state");

    let ticket = tandem_core::ticket::Ticket {
        meta,
        state,
        content: "ghost\n".to_string(),
    };

    let error = store
        .update_ticket(&ticket)
        .expect_err("update should fail for nonexistent ticket");

    assert!(
        error.to_string().contains("does not exist"),
        "error was: {error}"
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn update_ticket_cleans_up_stale_staging_dirs() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let store = FileTicketStore::new(repo_root.path().to_path_buf());

    let id = TicketId::parse("TNDM-STALE").expect("valid ticket id");
    let meta = TicketMeta::new(id.clone(), "Stale staging").expect("valid ticket meta");

    let created = store
        .create_ticket(NewTicket {
            meta,
            content: "body\n".to_string(),
        })
        .expect("create ticket");

    // Create stale staging dirs
    let tickets_path = repo_root.path().join(".tndm").join("tickets");
    let stale_staging = tickets_path.join(".TNDM-STALE.update.tmp");
    let stale_old = tickets_path.join(".TNDM-STALE.old.tmp");
    fs::create_dir_all(&stale_staging).expect("create stale staging dir");
    fs::create_dir_all(&stale_old).expect("create stale old dir");

    let mut ticket = created;
    ticket.state.status = TicketStatus::Done;
    ticket.state.revision = 2;
    ticket.state.updated_at = OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .expect("format timestamp");

    store
        .update_ticket(&ticket)
        .expect("update should succeed despite stale dirs");

    assert!(!stale_staging.exists());
    assert!(!stale_old.exists());

    let loaded = store.load_ticket(&id).expect("load updated ticket");
    assert_eq!(loaded.state.status, TicketStatus::Done);
}
