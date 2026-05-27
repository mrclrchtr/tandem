use std::{fs, path::Path, process::Command};

/// Write a `.tndm/config.toml` with the given ID prefix.
#[allow(clippy::disallowed_methods, dead_code)]
pub fn write_prefix_config(repo_root: &Path, prefix: &str) {
    fs::create_dir_all(repo_root.join(".tndm")).expect("create .tndm dir");
    fs::write(
        repo_root.join(".tndm").join("config.toml"),
        format!("schema_version = 1\n\n[id]\nprefix = \"{prefix}\"\n"),
    )
    .expect("write config.toml");
}

// ─── TestRepo ─────────────────────────────────────────────────

/// A test repository for CLI integration tests.
///
/// Manages a temporary directory with a `.git` marker and provides
/// convenience methods for running the `tndm` CLI binary, creating
/// tickets, and issuing git commands.
#[allow(dead_code)]
pub struct TestRepo {
    root: tempfile::TempDir,
}

#[allow(dead_code)]
impl TestRepo {
    /// Creates a temp dir with `.git` marker.
    #[allow(clippy::disallowed_methods)]
    pub fn new() -> Self {
        let root = tempfile::tempdir().expect("tempdir");
        fs::create_dir_all(root.path().join(".git")).expect("create .git dir");
        Self { root }
    }

    /// Creates a temp dir with `.git` marker and writes `.tndm/config.toml`.
    #[allow(dead_code)]
    pub fn with_config(prefix: &str) -> Self {
        let repo = Self::new();
        write_prefix_config(repo.path(), prefix);
        repo
    }

    /// Returns the repo root path.
    pub fn path(&self) -> &Path {
        self.root.path()
    }

    /// Creates a ticket via the CLI binary. Asserts success.
    pub fn create_ticket(&self, id: Option<&str>, title: &str) {
        let mut args = vec!["ticket", "create", title];
        if let Some(value) = id {
            args.push("--id");
            args.push(value);
        }
        let output = self.run(&args);
        assert!(
            output.status.success(),
            "ticket create '{}' failed: {}",
            title,
            String::from_utf8_lossy(&output.stderr)
        );
    }

    /// Creates a ticket by writing meta.toml/state.toml/content.md directly.
    ///
    /// Bypasses the CLI; useful for testing against pre-existing or specific state.
    #[allow(clippy::disallowed_methods, dead_code)]
    pub fn write_ticket(
        &self,
        id: &str,
        title: &str,
        status: &str,
        priority: &str,
        depends_on: &[&str],
    ) {
        let ticket_dir = self.root.path().join(".tndm").join("tickets").join(id);
        fs::create_dir_all(&ticket_dir).expect("create ticket dir");

        let deps = depends_on
            .iter()
            .map(|dependency_id| format!("\"{dependency_id}\""))
            .collect::<Vec<_>>()
            .join(", ");

        fs::write(
            ticket_dir.join("meta.toml"),
            format!(
                "schema_version = 1\nid = \"{id}\"\ntitle = \"{title}\"\n\ntype = \"task\"\npriority = \"{priority}\"\n\ndepends_on = [{deps}]\ntags = []\n",
            ),
        )
        .expect("write meta.toml");
        fs::write(
            ticket_dir.join("state.toml"),
            format!(
                "schema_version = 1\nstatus = \"{status}\"\nupdated_at = \"2026-03-08T00:00:00Z\"\nrevision = 1\n",
            ),
        )
        .expect("write state.toml");
        fs::write(ticket_dir.join("content.md"), "body\n").expect("write content.md");
    }

    /// Runs the `tndm` CLI and returns the raw `Output` (does NOT assert success).
    pub fn run(&self, args: &[&str]) -> std::process::Output {
        Command::new(env!("CARGO_BIN_EXE_tndm"))
            .args(args)
            .current_dir(self.root.path())
            .output()
            .expect("run tndm command")
    }

    /// Runs the `tndm` CLI, asserts success, and returns stdout as a `String`.
    pub fn run_assert(&self, args: &[&str]) -> String {
        let output = self.run(args);
        assert!(
            output.status.success(),
            "command `tndm {:?}` failed\nstderr: {}",
            args,
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).expect("stdout should be UTF-8")
    }

    /// Runs the `tndm` CLI with `--json` appended, asserts success,
    /// and returns the parsed `serde_json::Value`.
    ///
    /// If `--json` is already present in `args`, no duplicate is added.
    pub fn run_json(&self, args: &[&str]) -> serde_json::Value {
        let has_json = args.contains(&"--json");
        let all_args: Vec<&str> = if has_json {
            args.to_vec()
        } else {
            [args, &["--json"]].concat()
        };
        let stdout = self.run_assert(&all_args);
        serde_json::from_str(&stdout).expect("stdout should be valid JSON")
    }

    /// Runs a git command in the repo root.
    ///
    /// Asserts success and suppresses GPG signing during commits.
    #[allow(dead_code)]
    pub fn run_git(&self, args: &[&str]) {
        let output = Command::new("git")
            .arg("-C")
            .arg(self.root.path())
            .args(args)
            .env("GIT_AUTHOR_NAME", "Test User")
            .env("GIT_AUTHOR_EMAIL", "test@example.com")
            .env("GIT_COMMITTER_NAME", "Test User")
            .env("GIT_COMMITTER_EMAIL", "test@example.com")
            .env("GIT_CONFIG_COUNT", "1")
            .env("GIT_CONFIG_KEY_0", "commit.gpgsign")
            .env("GIT_CONFIG_VALUE_0", "false")
            .output()
            .expect("run git command");
        assert!(
            output.status.success(),
            "git {:?} failed: {}",
            args,
            String::from_utf8_lossy(&output.stderr)
        );
    }

    /// Reads a file from `.tndm/tickets/{id}/{filename}` as a `String`.
    #[allow(clippy::disallowed_methods, dead_code)]
    pub fn read_ticket_file(&self, id: &str, filename: &str) -> String {
        let path = self
            .root
            .path()
            .join(".tndm")
            .join("tickets")
            .join(id)
            .join(filename);
        fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
    }
}
