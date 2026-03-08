#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use std::{fs, path::Path, process::Command};

use tandem_core::{
    ports::TicketStore,
    ticket::{NewTicket, TicketId, TicketMeta},
};
use tandem_storage::FileTicketStore;

#[test]
fn awareness_prints_empty_json_when_snapshots_match() {
    let repo = TestRepo::new();
    repo.create_ticket("TNDM-1", "Same ticket");
    repo.commit_all("base");

    let output = repo.run_awareness("HEAD");

    assert!(
        output.status.success(),
        "expected success, stderr was: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stdout).expect("stdout should be UTF-8"),
        concat!(
            "{\n",
            "  \"schema_version\": 1,\n",
            "  \"against\": \"HEAD\",\n",
            "  \"tickets\": []\n",
            "}\n"
        )
    );
}

#[test]
fn awareness_reports_added_current_added_against_and_diverged_sorted() {
    let repo = TestRepo::new();
    repo.create_ticket("TNDM-2", "Diverged ticket");
    repo.set_ticket_priority("TNDM-2", "p3");
    repo.set_ticket_depends_on("TNDM-2", &["TNDM-9"]);
    repo.create_ticket("TNDM-3", "Against only ticket");
    repo.commit_all("base");

    repo.set_ticket_status("TNDM-2", "in_progress");
    repo.set_ticket_priority("TNDM-2", "p1");
    repo.set_ticket_depends_on("TNDM-2", &["TNDM-1"]);
    repo.remove_ticket("TNDM-3");
    repo.create_ticket("TNDM-4", "Current only ticket");

    let output = repo.run_awareness("HEAD");

    assert!(
        output.status.success(),
        "expected success, stderr was: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stdout).expect("stdout should be UTF-8"),
        concat!(
            "{\n",
            "  \"schema_version\": 1,\n",
            "  \"against\": \"HEAD\",\n",
            "  \"tickets\": [\n",
            "    {\n",
            "      \"id\": \"TNDM-2\",\n",
            "      \"change\": \"diverged\",\n",
            "      \"fields\": {\n",
            "        \"status\": {\n",
            "          \"current\": \"in_progress\",\n",
            "          \"against\": \"todo\"\n",
            "        },\n",
            "        \"priority\": {\n",
            "          \"current\": \"p1\",\n",
            "          \"against\": \"p3\"\n",
            "        },\n",
            "        \"depends_on\": {\n",
            "          \"current\": [\n",
            "            \"TNDM-1\"\n",
            "          ],\n",
            "          \"against\": [\n",
            "            \"TNDM-9\"\n",
            "          ]\n",
            "        }\n",
            "      }\n",
            "    },\n",
            "    {\n",
            "      \"id\": \"TNDM-3\",\n",
            "      \"change\": \"added_against\"\n",
            "    },\n",
            "    {\n",
            "      \"id\": \"TNDM-4\",\n",
            "      \"change\": \"added_current\"\n",
            "    }\n",
            "  ]\n",
            "}\n"
        )
    );
}

#[test]
fn awareness_errors_for_invalid_ref() {
    let repo = TestRepo::new();

    let output = repo.run_awareness("does-not-exist");

    assert!(!output.status.success(), "invalid ref should fail");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be UTF-8");
    assert!(
        stderr.contains("git rev-parse --verify does-not-exist^{commit}"),
        "stderr was: {stderr:?}"
    );
    assert!(stderr.contains("does-not-exist"), "stderr was: {stderr:?}");
}

#[test]
fn awareness_errors_for_invalid_committed_ticket_data_without_temp_path_leakage() {
    let repo = TestRepo::new();
    repo.create_ticket("TNDM-1", "Broken committed ticket");
    repo.write_ticket_file("TNDM-1", "meta.toml", "not toml\n");
    repo.commit_all("broken snapshot");
    fs::remove_dir_all(repo.root().join(".tndm")).expect("remove current .tndm directory");

    let output = repo.run_awareness("HEAD");

    assert!(
        !output.status.success(),
        "invalid committed data should fail"
    );
    let stderr = String::from_utf8(output.stderr).expect("stderr should be UTF-8");
    assert!(
        stderr.contains("failed to load materialized snapshot for ref `HEAD`"),
        "stderr was: {stderr:?}"
    );
    assert!(
        stderr.contains("<ref-snapshot>/.tndm/tickets/TNDM-1/meta.toml"),
        "stderr was: {stderr:?}"
    );
    assert!(!stderr.contains("/tmp/"), "stderr was: {stderr:?}");
    assert!(!stderr.contains("/private/tmp/"), "stderr was: {stderr:?}");
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

    fn run_awareness(&self, against: &str) -> std::process::Output {
        Command::new(env!("CARGO_BIN_EXE_tndm"))
            .arg("awareness")
            .arg("--against")
            .arg(against)
            .current_dir(self.root())
            .output()
            .expect("run tndm awareness")
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

    fn set_ticket_status(&self, id: &str, status: &str) {
        let path = self
            .root()
            .join(".tndm/tickets")
            .join(id)
            .join("state.toml");
        let contents = fs::read_to_string(&path).expect("read state.toml");
        let updated = contents.replace("status = \"todo\"", &format!("status = \"{status}\""));
        fs::write(path, updated).expect("write state.toml");
    }

    fn set_ticket_priority(&self, id: &str, priority: &str) {
        self.rewrite_meta_line(id, "priority = ", &format!("priority = \"{priority}\""));
    }

    fn set_ticket_depends_on(&self, id: &str, depends_on: &[&str]) {
        let depends_on = format!(
            "depends_on = [{}]",
            depends_on
                .iter()
                .map(|dependency| format!("\"{dependency}\""))
                .collect::<Vec<_>>()
                .join(", ")
        );
        self.rewrite_meta_line(id, "depends_on = ", &depends_on);
    }

    fn rewrite_meta_line(&self, id: &str, prefix: &str, replacement: &str) {
        let path = self.root().join(".tndm/tickets").join(id).join("meta.toml");
        let contents = fs::read_to_string(&path).expect("read meta.toml");
        let updated = contents
            .lines()
            .map(|line| {
                if line.starts_with(prefix) {
                    replacement.to_string()
                } else {
                    line.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
            + "\n";
        fs::write(path, updated).expect("write meta.toml");
    }

    fn write_ticket_file(&self, id: &str, file_name: &str, contents: &str) {
        let path = self.root().join(".tndm/tickets").join(id).join(file_name);
        fs::write(path, contents).expect("write ticket file");
    }

    fn remove_ticket(&self, id: &str) {
        fs::remove_dir_all(self.root().join(".tndm/tickets").join(id)).expect("remove ticket dir");
    }

    fn commit_all(&self, message: &str) {
        run_git(self.root(), &["add", "."]);
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
