use std::fs;

use tandem_core::{
    ports::TicketStore,
    ticket::{
        NewTicket, TicketEffort, TicketId, TicketMeta, TicketPriority, TicketStatus, TicketType,
    },
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

#[test]
#[allow(clippy::disallowed_methods)]
fn load_ticket_parses_effort_from_meta_toml() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let id = TicketId::parse("TNDM-EFF01").expect("valid ticket id");
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
            "id = \"TNDM-EFF01\"\n",
            "title = \"Effort ticket\"\n",
            "\n",
            "type = \"task\"\n",
            "priority = \"p2\"\n",
            "effort = \"m\"\n",
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
            "updated_at = \"2026-04-01T10:00:00Z\"\n",
            "revision = 1\n",
        ),
    )
    .expect("write state.toml");
    fs::write(ticket_dir.join("content.md"), "").expect("write content.md");

    let store = FileTicketStore::new(repo_root.path().to_path_buf());
    let ticket = store.load_ticket(&id).expect("load ticket");

    assert_eq!(ticket.meta.effort, Some(TicketEffort::M));
}

#[test]
#[allow(clippy::disallowed_methods)]
fn load_ticket_defaults_effort_to_none_when_absent() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let id = TicketId::parse("TNDM-EFF02").expect("valid ticket id");
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
            "id = \"TNDM-EFF02\"\n",
            "title = \"No effort ticket\"\n",
            "\n",
            "type = \"task\"\n",
            "priority = \"p2\"\n",
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
            "updated_at = \"2026-04-01T10:00:00Z\"\n",
            "revision = 1\n",
        ),
    )
    .expect("write state.toml");
    fs::write(ticket_dir.join("content.md"), "").expect("write content.md");

    let store = FileTicketStore::new(repo_root.path().to_path_buf());
    let ticket = store.load_ticket(&id).expect("load ticket");

    assert_eq!(ticket.meta.effort, None);
}

#[test]
#[allow(clippy::disallowed_methods)]
fn load_ticket_rejects_invalid_effort_in_meta_toml() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let id = TicketId::parse("TNDM-EFF03").expect("valid ticket id");
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
            "id = \"TNDM-EFF03\"\n",
            "title = \"Bad effort ticket\"\n",
            "\n",
            "type = \"task\"\n",
            "priority = \"p2\"\n",
            "effort = \"huge\"\n",
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
            "updated_at = \"2026-04-01T10:00:00Z\"\n",
            "revision = 1\n",
        ),
    )
    .expect("write state.toml");
    fs::write(ticket_dir.join("content.md"), "").expect("write content.md");

    let store = FileTicketStore::new(repo_root.path().to_path_buf());
    let error = store
        .load_ticket(&id)
        .expect_err("should fail with invalid effort");

    assert!(
        error.to_string().contains("invalid effort"),
        "error should mention invalid effort, got: {error}"
    );
}

// ─── Document registry and fingerprint tests ─────────────────

#[test]
#[allow(clippy::disallowed_methods)]
fn create_ticket_registers_content_document_with_fingerprint() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let store = FileTicketStore::new(repo_root.path().to_path_buf());
    let id = TicketId::parse("TNDM-DOCR01").expect("valid ticket id");
    let meta = TicketMeta::new(id.clone(), "Doc reg test").expect("valid ticket meta");
    let content = "# Hello\n\nThis is content.\n".to_string();

    let ticket = store
        .create_ticket(NewTicket { meta, content })
        .expect("create ticket");

    // The in-memory meta should have the content doc
    assert_eq!(ticket.meta.documents.len(), 1);
    assert_eq!(ticket.meta.documents[0].name, "content");
    assert_eq!(ticket.meta.documents[0].path, "content.md");

    // meta.toml should have [[documents]]
    let meta_path = repo_root
        .path()
        .join(".tndm")
        .join("tickets")
        .join(id.as_str())
        .join("meta.toml");
    let meta_text = fs::read_to_string(&meta_path).expect("read meta.toml");
    assert!(
        meta_text.contains("[[documents]]"),
        "meta.toml should contain [[documents]]: {meta_text}"
    );
    assert!(
        meta_text.contains(r#"name = "content""#),
        "meta.toml should have content doc: {meta_text}"
    );

    // state.toml should have a fingerprint for content
    let state_path = repo_root
        .path()
        .join(".tndm")
        .join("tickets")
        .join(id.as_str())
        .join("state.toml");
    let state_text = fs::read_to_string(&state_path).expect("read state.toml");
    assert!(
        state_text.contains("[document_fingerprints]"),
        "state.toml should contain [document_fingerprints]: {state_text}"
    );
    assert!(
        state_text.contains(r"content = "),
        "state.toml should have content fingerprint: {state_text}"
    );
    assert!(
        state_text.contains("sha256:"),
        "fingerprint should start with sha256:: {state_text}"
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn create_ticket_omits_fingerprints_section_when_no_documents() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let store = FileTicketStore::new(repo_root.path().to_path_buf());
    let id = TicketId::parse("TNDM-DOCR02").expect("valid ticket id");

    // Create a ticket with an empty documents vec to simulate no docs
    let mut meta = TicketMeta::new(id.clone(), "No docs").expect("valid ticket meta");
    meta.documents = Vec::new();

    let ticket = store
        .create_ticket(NewTicket {
            meta,
            content: "body".to_string(),
        })
        .expect("create ticket");

    assert!(
        ticket.state.document_fingerprints.is_empty(),
        "fingerprints should be empty when no documents"
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn load_legacy_ticket_infers_default_content_document() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let id = TicketId::parse("TNDM-LEG01").expect("valid ticket id");
    let ticket_dir = repo_root
        .path()
        .join(".tndm")
        .join("tickets")
        .join(id.as_str());
    fs::create_dir_all(&ticket_dir).expect("create ticket dir");

    // Legacy meta.toml without [[documents]]
    fs::write(
        ticket_dir.join("meta.toml"),
        concat!(
            "schema_version = 1\n",
            "id = \"TNDM-LEG01\"\n",
            "title = \"Legacy ticket\"\n",
            "\n",
            "type = \"task\"\n",
            "priority = \"p2\"\n",
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
            "updated_at = \"2026-03-03T10:00:00Z\"\n",
            "revision = 1\n",
        ),
    )
    .expect("write state.toml");
    fs::write(ticket_dir.join("content.md"), "legacy body\n").expect("write content.md");

    let store = FileTicketStore::new(repo_root.path().to_path_buf());
    let ticket = store.load_ticket(&id).expect("load legacy ticket");

    // Should have inferred the default content document
    assert_eq!(ticket.meta.documents.len(), 1);
    assert_eq!(ticket.meta.documents[0].name, "content");
    assert_eq!(ticket.meta.documents[0].path, "content.md");
    assert_eq!(ticket.content, "legacy body\n");
}

#[test]
#[allow(clippy::disallowed_methods)]
fn update_ticket_preserves_registered_documents() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let store = FileTicketStore::new(repo_root.path().to_path_buf());
    let id = TicketId::parse("TNDM-DOCR04").expect("valid ticket id");
    let meta = TicketMeta::new(id.clone(), "Multi doc").expect("valid ticket meta");

    let ticket = store
        .create_ticket(NewTicket {
            meta,
            content: "orig\n".to_string(),
        })
        .expect("create ticket");

    // Add a second document via the in-memory model
    let mut updated = ticket.clone();
    updated
        .meta
        .documents
        .push(tandem_core::ticket::TicketDocument {
            name: "extra".to_string(),
            path: "extra.md".to_string(),
        });
    updated.state.revision = 2;
    updated.state.updated_at = "2026-03-04T10:00:00Z".to_string();

    let result = store.update_ticket(&updated).expect("update ticket");

    // Should preserve both documents
    assert_eq!(result.meta.documents.len(), 2);
    let doc_names: Vec<&str> = result
        .meta
        .documents
        .iter()
        .map(|d| d.name.as_str())
        .collect();
    assert!(doc_names.contains(&"content"));
    assert!(doc_names.contains(&"extra"));
}

#[test]
#[allow(clippy::disallowed_methods)]
fn sync_after_content_edit_recomputes_fingerprint() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let store = FileTicketStore::new(repo_root.path().to_path_buf());
    let id = TicketId::parse("TNDM-SYNC1").expect("valid ticket id");
    let meta = TicketMeta::new(id.clone(), "Sync test").expect("valid ticket meta");

    let ticket = store
        .create_ticket(NewTicket {
            meta,
            content: "original content\n".to_string(),
        })
        .expect("create ticket");

    let original_fingerprint = ticket.state.document_fingerprints.get("content").cloned();
    assert!(
        original_fingerprint.is_some(),
        "should have fingerprint after create"
    );

    // Edit the content.md file directly
    let content_path = repo_root
        .path()
        .join(".tndm")
        .join("tickets")
        .join(id.as_str())
        .join("content.md");
    fs::write(&content_path, "edited content\n").expect("edit content.md");

    // Sync should recompute fingerprint
    let synced = store.sync_ticket_documents(&id).expect("sync documents");

    let new_fingerprint = synced.state.document_fingerprints.get("content").cloned();
    assert!(
        new_fingerprint.is_some(),
        "should have fingerprint after sync"
    );
    assert_ne!(
        new_fingerprint, original_fingerprint,
        "fingerprint should change after content edit"
    );
    assert!(
        synced.state.revision > ticket.state.revision,
        "revision should increase after sync"
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn sync_detects_stale_fingerprints() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let store = FileTicketStore::new(repo_root.path().to_path_buf());
    let id = TicketId::parse("TNDM-STL1").expect("valid ticket id");
    let meta = TicketMeta::new(id.clone(), "Stale test").expect("valid ticket meta");

    store
        .create_ticket(NewTicket {
            meta,
            content: "original\n".to_string(),
        })
        .expect("create ticket");

    // Edit content.md directly
    let content_path = repo_root
        .path()
        .join(".tndm")
        .join("tickets")
        .join(id.as_str())
        .join("content.md");
    fs::write(&content_path, "edited content\n").expect("edit content.md");

    // document_drift should detect fingerprints are stale
    let drift = store.document_drift(&id).expect("check drift");

    assert!(!drift.is_empty(), "drift should not be empty after edit");
    assert!(
        drift.iter().any(|(name, _)| name == "content"),
        "content should be in drift: {drift:?}"
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn sync_clears_drift() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let store = FileTicketStore::new(repo_root.path().to_path_buf());
    let id = TicketId::parse("TNDM-DRFT1").expect("valid ticket id");
    let meta = TicketMeta::new(id.clone(), "Drift clear").expect("valid ticket meta");

    store
        .create_ticket(NewTicket {
            meta,
            content: "original\n".to_string(),
        })
        .expect("create ticket");

    // Edit content.md directly
    let content_path = repo_root
        .path()
        .join(".tndm")
        .join("tickets")
        .join(id.as_str())
        .join("content.md");
    fs::write(&content_path, "edited content\n").expect("edit content.md");

    // Sync should clear drift
    store.sync_ticket_documents(&id).expect("sync documents");

    let drift = store.document_drift(&id).expect("check drift after sync");
    assert!(
        drift.is_empty(),
        "drift should be empty after sync, got: {drift:?}"
    );
}
