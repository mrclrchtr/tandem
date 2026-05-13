use std::fs;

use tandem_core::{
    ports::TicketStore,
    ticket::{NewTicket, TicketId, TicketMeta},
};
use tandem_storage::{FileTicketStore, ticket_dir};

#[test]
#[allow(clippy::disallowed_methods)]
fn create_ticket_keeps_existing_directory_unchanged_when_finalize_fails() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let ticket_id = TicketId::parse("TNDM-LOCKED").expect("valid ticket id");
    let final_dir = repo_root
        .path()
        .join(".tndm")
        .join("tickets")
        .join(ticket_id.as_str());
    fs::create_dir_all(&final_dir).expect("create existing ticket dir");
    fs::write(final_dir.join("sentinel.txt"), "keep me\n").expect("write sentinel");

    let store = FileTicketStore::new(repo_root.path().to_path_buf());
    let meta = TicketMeta::new(ticket_id.clone(), "Locked ticket").expect("valid meta");

    let error = store
        .create_ticket(NewTicket {
            meta,
            content: "body\n".to_string(),
        })
        .expect_err("create should fail when final dir already exists");

    assert!(
        error.to_string().contains("failed to finalize"),
        "error was: {}",
        error
    );
    assert_eq!(
        fs::read_to_string(final_dir.join("sentinel.txt")).expect("read sentinel"),
        "keep me\n"
    );

    let tickets_dir = repo_root.path().join(".tndm").join("tickets");
    let entries = fs::read_dir(&tickets_dir)
        .expect("read tickets dir")
        .map(|entry| {
            entry
                .expect("dir entry")
                .file_name()
                .into_string()
                .expect("utf8 name")
        })
        .collect::<Vec<_>>();
    assert_eq!(entries, vec!["TNDM-LOCKED".to_string()]);
}

#[test]
#[allow(clippy::disallowed_methods)]
fn create_ticket_recovers_from_stale_per_ticket_staging_directory() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let ticket_id = TicketId::parse("TNDM-STALE").expect("valid ticket id");
    let staging_dir = repo_root
        .path()
        .join(".tndm")
        .join("tickets")
        .join(format!(".{}.tmp", ticket_id.as_str()));
    fs::create_dir_all(&staging_dir).expect("create stale staging dir");
    fs::write(staging_dir.join("meta.toml"), "partial\n").expect("write stale meta");

    let store = FileTicketStore::new(repo_root.path().to_path_buf());
    let meta = TicketMeta::new(ticket_id.clone(), "Recovered ticket").expect("valid meta");

    let ticket = store
        .create_ticket(NewTicket {
            meta,
            content: "body\n".to_string(),
        })
        .expect("create should replace stale staging dir");

    assert_eq!(ticket.meta.id, ticket_id);
    assert_eq!(ticket.content, "body\n");

    let final_dir = ticket_dir(repo_root.path(), &ticket_id);
    assert!(final_dir.is_dir(), "final ticket dir should exist");
    assert!(
        !staging_dir.exists(),
        "stale staging dir should be removed after successful create"
    );
    assert_eq!(
        fs::read_to_string(final_dir.join("content.md")).expect("read content"),
        "body\n"
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn list_ticket_ids_ignores_stale_staging_directories() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let store = FileTicketStore::new(repo_root.path().to_path_buf());

    let ticket_id = TicketId::parse("TNDM-123").expect("valid ticket id");
    let ticket_path = ticket_dir(repo_root.path(), &ticket_id);
    fs::create_dir_all(&ticket_path).expect("create ticket dir");
    fs::write(
        ticket_path.join("meta.toml"),
        concat!(
            "schema_version = 1\n",
            "id = \"TNDM-123\"\n",
            "title = \"Real ticket\"\n",
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
        ticket_path.join("state.toml"),
        concat!(
            "schema_version = 1\n",
            "status = \"todo\"\n",
            "updated_at = \"2026-03-03T12:34:56Z\"\n",
            "revision = 1\n",
        ),
    )
    .expect("write state.toml");
    fs::write(ticket_path.join("content.md"), "body\n").expect("write content.md");

    let staging_dir = repo_root
        .path()
        .join(".tndm")
        .join("tickets")
        .join(".TNDM-999.tmp");
    fs::create_dir_all(&staging_dir).expect("create stale staging dir");
    fs::write(staging_dir.join("meta.toml"), "partial\n").expect("write stale meta");

    let ids = store.list_ticket_ids().expect("list ticket ids");

    assert_eq!(ids, vec![ticket_id]);
}
