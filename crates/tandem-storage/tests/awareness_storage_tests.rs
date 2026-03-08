use std::fs;

use tandem_core::{
    ports::TicketStore,
    ticket::{NewTicket, TicketId, TicketMeta},
};
use tandem_storage::{FileTicketStore, load_ticket_snapshot};

#[allow(clippy::disallowed_methods)]
#[test]
fn load_ticket_snapshot_returns_sorted_tickets() {
    let repo_root = tempfile::tempdir().unwrap();
    fs::create_dir_all(repo_root.path().join(".git")).unwrap();
    let store = FileTicketStore::new(repo_root.path().to_path_buf());

    for (id, title) in [("TNDM-2", "Second"), ("TNDM-1", "First")] {
        let id = TicketId::parse(id).unwrap();
        let meta = TicketMeta::new(id, title).unwrap();
        store
            .create_ticket(NewTicket {
                meta,
                content: "body\n".to_string(),
            })
            .unwrap();
    }

    let snapshot = load_ticket_snapshot(repo_root.path()).unwrap();
    let ids = snapshot
        .tickets
        .keys()
        .map(tandem_core::ticket::TicketId::as_str)
        .map(str::to_string)
        .collect::<Vec<_>>();

    assert_eq!(ids, vec!["TNDM-1", "TNDM-2"]);
}

#[allow(clippy::disallowed_methods)]
#[test]
fn load_ticket_snapshot_returns_empty_when_tickets_dir_is_missing() {
    let repo_root = tempfile::tempdir().unwrap();
    fs::create_dir_all(repo_root.path().join(".git")).unwrap();

    let snapshot = load_ticket_snapshot(repo_root.path()).unwrap();

    assert!(snapshot.tickets.is_empty());
}
