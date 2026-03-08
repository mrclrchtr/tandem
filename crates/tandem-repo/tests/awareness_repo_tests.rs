#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use std::{fs, path::Path, process::Command};

use tandem_core::{
    awareness::TicketSnapshot,
    ports::{AwarenessSnapshotProvider, TicketStore},
    ticket::{NewTicket, TicketId, TicketMeta, TicketStatus},
};
use tandem_repo::GitAwarenessProvider;
use tandem_storage::FileTicketStore;

#[test]
fn load_current_snapshot_reads_working_tree_tickets() {
    let repo = TestRepo::new();
    repo.create_ticket("TNDM-1", "Working tree ticket");
    repo.set_ticket_status("TNDM-1", TicketStatus::InProgress);

    let snapshot = repo
        .provider()
        .load_current_snapshot()
        .expect("load snapshot");

    assert_eq!(snapshot.tickets.len(), 1);
    assert_eq!(ticket_status(&snapshot, "TNDM-1"), TicketStatus::InProgress);
}

#[test]
fn load_snapshot_for_ref_reads_committed_ticket_files() {
    let repo = TestRepo::new();
    repo.create_ticket("TNDM-1", "Committed ticket");
    repo.commit_all("base");
    repo.set_ticket_status("TNDM-1", TicketStatus::InProgress);

    let provider = repo.provider();
    let current = provider.load_current_snapshot().expect("load snapshot");
    let head = provider
        .load_snapshot_for_ref("HEAD")
        .expect("load snapshot for HEAD");

    assert_eq!(ticket_status(&current, "TNDM-1"), TicketStatus::InProgress);
    assert_eq!(ticket_status(&head, "TNDM-1"), TicketStatus::Todo);
}

#[test]
fn load_snapshot_for_ref_returns_empty_when_ref_has_no_tickets() {
    let repo = TestRepo::new();
    fs::write(repo.root().join("README.md"), "repo\n").expect("write readme");
    repo.commit_paths("base", &["README.md"]);

    let snapshot = repo
        .provider()
        .load_snapshot_for_ref("HEAD")
        .expect("load snapshot for HEAD");

    assert!(snapshot.tickets.is_empty());
}

#[test]
fn load_snapshot_for_ref_errors_for_unknown_ref() {
    let repo = TestRepo::new();

    let error = repo
        .provider()
        .load_snapshot_for_ref("does-not-exist")
        .expect_err("unknown ref should fail");

    let message = error.to_string();
    assert!(message.contains("git rev-parse --verify does-not-exist^{commit}"));
    assert!(message.contains("does-not-exist"));
    assert!(!message.contains(&repo.root().display().to_string()));
}

struct TestRepo {
    root: tempfile::TempDir,
}

impl TestRepo {
    fn new() -> Self {
        let root = tempfile::tempdir().expect("tempdir");
        run_git(root.path(), &["init", "-b", "main"]);
        run_git(root.path(), &["config", "user.name", "Test User"]);
        run_git(root.path(), &["config", "user.email", "test@example.com"]);
        run_git(root.path(), &["config", "commit.gpgsign", "false"]);
        Self { root }
    }

    fn root(&self) -> &Path {
        self.root.path()
    }

    fn provider(&self) -> GitAwarenessProvider {
        GitAwarenessProvider::new(self.root().to_path_buf())
    }

    fn create_ticket(&self, id: &str, title: &str) {
        let store = FileTicketStore::new(self.root().to_path_buf());
        let id = TicketId::parse(id).expect("valid ticket id");
        let meta = TicketMeta::new(id, title).expect("valid ticket meta");
        store
            .create_ticket(NewTicket {
                meta,
                content: format!("## Description\n\n{title}\n"),
            })
            .expect("create ticket");
    }

    fn set_ticket_status(&self, id: &str, status: TicketStatus) {
        let ticket_dir = self.root().join(".tndm").join("tickets").join(id);
        let state_path = ticket_dir.join("state.toml");
        let state = fs::read_to_string(&state_path).expect("read state");
        let next = state.replace(
            "status = \"todo\"",
            &format!("status = \"{}\"", status.as_str()),
        );
        fs::write(state_path, next).expect("write state");
    }

    fn commit_all(&self, message: &str) {
        run_git(self.root(), &["add", "."]);
        run_git(self.root(), &["commit", "-m", message]);
    }

    fn commit_paths(&self, message: &str, paths: &[&str]) {
        let mut args = vec!["add"];
        args.extend_from_slice(paths);
        run_git(self.root(), &args);
        run_git(self.root(), &["commit", "-m", message]);
    }
}

fn ticket_status(snapshot: &TicketSnapshot, id: &str) -> TicketStatus {
    snapshot
        .tickets
        .get(&TicketId::parse(id).expect("valid ticket id"))
        .expect("ticket in snapshot")
        .state
        .status
}

fn run_git(repo_root: &Path, args: &[&str]) {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo_root)
        .output()
        .expect("run git");

    assert!(
        output.status.success(),
        "git {:?} failed\nstdout:\n{}\nstderr:\n{}",
        args,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
