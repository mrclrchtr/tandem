#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use std::{fs, path::Path, process::Command};

use tandem_core::{
    ports::TicketStore,
    ticket::{NewTicket, TicketId, TicketMeta},
};
use tandem_repo::GitAwarenessProvider;
use tandem_storage::{FileTicketStore, load_ticket_snapshot};

#[test]
fn materialize_ref_snapshot_writes_committed_ticket_files() {
    let repo = TestRepo::new();
    repo.create_ticket("TNDM-1", "Committed ticket");
    repo.commit_all("base");

    let committed_meta = fs::read_to_string(repo.root().join(".tndm/tickets/TNDM-1/meta.toml"))
        .expect("read committed meta");
    let committed_state = fs::read_to_string(repo.root().join(".tndm/tickets/TNDM-1/state.toml"))
        .expect("read committed state");
    let committed_content = fs::read_to_string(repo.root().join(".tndm/tickets/TNDM-1/content.md"))
        .expect("read committed content");

    repo.write_ticket_file("TNDM-1", "content.md", "working tree body\n");

    let snapshot = repo
        .provider()
        .materialize_ref_snapshot("HEAD")
        .expect("materialize ref snapshot")
        .expect("snapshot root should exist");

    assert_eq!(
        fs::read_to_string(snapshot.path().join(".tndm/tickets/TNDM-1/meta.toml"))
            .expect("read materialized meta"),
        committed_meta
    );
    assert_eq!(
        fs::read_to_string(snapshot.path().join(".tndm/tickets/TNDM-1/state.toml"))
            .expect("read materialized state"),
        committed_state
    );
    assert_eq!(
        fs::read_to_string(snapshot.path().join(".tndm/tickets/TNDM-1/content.md"))
            .expect("read materialized content"),
        committed_content
    );
}

#[test]
fn materialize_ref_snapshot_returns_empty_when_ref_has_no_tickets() {
    let repo = TestRepo::new();
    fs::write(repo.root().join("README.md"), "repo\n").expect("write readme");
    repo.commit_paths("base", &["README.md"]);

    let snapshot = repo
        .provider()
        .materialize_ref_snapshot("HEAD")
        .expect("materialize ref snapshot");

    assert!(snapshot.is_none());
}

#[test]
fn materialize_ref_snapshot_errors_for_unknown_ref() {
    let repo = TestRepo::new();

    let error = repo
        .provider()
        .materialize_ref_snapshot("does-not-exist")
        .expect_err("unknown ref should fail");

    let message = error.to_string();
    assert!(message.contains("git rev-parse --verify does-not-exist^{commit}"));
    assert!(message.contains("does-not-exist"));
    assert!(!message.contains(&repo.root().display().to_string()));
}

#[test]
fn materialize_ref_snapshot_sanitizes_temp_paths_for_invalid_committed_ticket_data() {
    let repo = TestRepo::new();
    repo.create_ticket("TNDM-1", "Broken ticket");
    repo.write_ticket_file("TNDM-1", "meta.toml", "not toml\n");
    repo.commit_all("broken snapshot");

    let snapshot = repo
        .provider()
        .materialize_ref_snapshot("HEAD")
        .expect("materialize ref snapshot")
        .expect("snapshot root should exist");

    let error = load_ticket_snapshot(snapshot.path())
        .expect_err("invalid materialized snapshot should fail")
        .to_string();
    let sanitized = snapshot.sanitize_error_text(&error);

    assert!(sanitized.contains("<ref-snapshot>/.tndm/tickets/TNDM-1/meta.toml"));
    assert!(!sanitized.contains(&snapshot.path().display().to_string()));
    assert!(!sanitized.contains("/tmp/"));
    assert!(!sanitized.contains("/private/tmp/"));
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

    fn write_ticket_file(&self, id: &str, file_name: &str, contents: &str) {
        let path = self
            .root()
            .join(".tndm")
            .join("tickets")
            .join(id)
            .join(file_name);
        fs::write(path, contents).expect("write ticket file");
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
