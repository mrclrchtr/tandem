#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use std::{
    fmt, fs,
    path::{Component, Path, PathBuf},
    process::Command,
};

use tandem_core::ports::{AwarenessRefMaterializer, MaterializedRefSnapshot, RepoContext};

#[derive(Debug, Default, Clone, Copy)]
pub struct GitRepoContext;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitAwarenessProvider {
    repo_root: PathBuf,
}

impl GitAwarenessProvider {
    pub fn new(repo_root: PathBuf) -> Self {
        Self { repo_root }
    }
}

#[derive(Debug)]
pub struct GitMaterializedRefSnapshot {
    tempdir: tempfile::TempDir,
}

impl GitMaterializedRefSnapshot {
    pub fn sanitize_error_text(&self, text: &str) -> String {
        sanitize_snapshot_path(text, self.path())
    }
}

impl MaterializedRefSnapshot for GitMaterializedRefSnapshot {
    fn path(&self) -> &Path {
        self.tempdir.path()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoError {
    message: String,
}

impl RepoError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    fn not_implemented(operation: &str) -> Self {
        Self::new(format!("repo operation `{operation}` is not implemented"))
    }

    fn git_command_failed(args: &[&str], stderr: &[u8]) -> Self {
        let command = format!("git {}", args.join(" "));
        let stderr = String::from_utf8_lossy(stderr);
        let details = match stderr.trim() {
            "" => "git command exited unsuccessfully".to_string(),
            value => value.to_string(),
        };

        Self::new(format!("{command} failed: {details}"))
    }

    fn unsafe_ticket_path(reference: &str, ticket_path: &Path) -> Self {
        Self::new(format!(
            "unsafe ticket path `{}` for ref `{reference}`",
            ticket_path.display()
        ))
    }
}

impl fmt::Display for RepoError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.message)
    }
}

impl std::error::Error for RepoError {}

impl RepoContext for GitRepoContext {
    type Error = RepoError;

    fn current_branch(&self) -> Result<String, Self::Error> {
        Err(RepoError::not_implemented("current_branch"))
    }

    fn list_worktrees(&self) -> Result<Vec<String>, Self::Error> {
        Err(RepoError::not_implemented("list_worktrees"))
    }
}

impl AwarenessRefMaterializer for GitAwarenessProvider {
    type Error = RepoError;
    type Snapshot = GitMaterializedRefSnapshot;

    fn materialize_ref_snapshot(
        &self,
        reference: &str,
    ) -> Result<Option<Self::Snapshot>, Self::Error> {
        let resolved_commit = resolve_ref_commit(&self.repo_root, reference)?;
        let ticket_paths = list_ref_ticket_paths(&self.repo_root, &resolved_commit)?;
        if ticket_paths.is_empty() {
            return Ok(None);
        }

        let tempdir = tempfile::tempdir().map_err(|error| {
            RepoError::new(format!(
                "failed to create temp snapshot root for ref `{reference}`: {error}"
            ))
        })?;

        write_ref_ticket_tree(
            tempdir.path(),
            &self.repo_root,
            reference,
            &resolved_commit,
            &ticket_paths,
        )?;
        Ok(Some(GitMaterializedRefSnapshot { tempdir }))
    }
}

fn run_git(repo_root: &Path, args: &[&str]) -> Result<Vec<u8>, RepoError> {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo_root)
        .output()
        .map_err(|error| {
            RepoError::new(format!("failed to run git {}: {error}", args.join(" ")))
        })?;

    if output.status.success() {
        Ok(output.stdout)
    } else {
        Err(RepoError::git_command_failed(args, &output.stderr))
    }
}

fn resolve_ref_commit(repo_root: &Path, reference: &str) -> Result<String, RepoError> {
    let resolved_ref = format!("{reference}^{{commit}}");
    let output = run_git(repo_root, &["rev-parse", "--verify", &resolved_ref])?;

    Ok(String::from_utf8_lossy(&output).trim().to_string())
}

fn list_ref_ticket_paths(
    repo_root: &Path,
    resolved_commit: &str,
) -> Result<Vec<PathBuf>, RepoError> {
    let output = run_git(
        repo_root,
        &[
            "ls-tree",
            "-r",
            "--name-only",
            resolved_commit,
            "--",
            ".tndm/tickets",
        ],
    )?;

    Ok(String::from_utf8_lossy(&output)
        .lines()
        .filter(|line| !line.is_empty())
        .map(PathBuf::from)
        .collect())
}

fn resolve_materialized_ticket_path(
    destination_root: &Path,
    ticket_path: &Path,
    reference: &str,
) -> Result<PathBuf, RepoError> {
    if ticket_path.is_absolute() {
        return Err(RepoError::unsafe_ticket_path(reference, ticket_path));
    }

    if ticket_path
        .components()
        .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(RepoError::unsafe_ticket_path(reference, ticket_path));
    }

    Ok(destination_root.join(ticket_path))
}

fn write_ref_ticket_tree(
    destination_root: &Path,
    repo_root: &Path,
    reference: &str,
    resolved_commit: &str,
    ticket_paths: &[PathBuf],
) -> Result<(), RepoError> {
    for ticket_path in ticket_paths {
        let ticket_path_string = ticket_path.to_string_lossy().to_string();
        let blob = run_git(
            repo_root,
            &["show", &format!("{resolved_commit}:{ticket_path_string}")],
        )?;

        let destination_path =
            resolve_materialized_ticket_path(destination_root, ticket_path, reference)?;
        if let Some(parent) = destination_path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                RepoError::new(format!(
                    "failed to create snapshot directory for ref `{reference}`: {error}"
                ))
            })?;
        }

        fs::write(&destination_path, blob).map_err(|error| {
            RepoError::new(format!(
                "failed to materialize snapshot file `{ticket_path_string}` for ref `{reference}`: {error}"
            ))
        })?;
    }

    Ok(())
}

fn sanitize_snapshot_path(text: &str, snapshot_root: &Path) -> String {
    let normalized_root = snapshot_root.to_string_lossy().replace('\\', "/");

    text.replace(snapshot_root.to_string_lossy().as_ref(), "<ref-snapshot>")
        .replace(&normalized_root, "<ref-snapshot>")
}

#[cfg(test)]
mod tests {
    use super::{list_ref_ticket_paths, resolve_materialized_ticket_path};
    use std::path::{Path, PathBuf};
    use std::process::Command;

    #[test]
    fn list_ref_ticket_paths_reads_from_resolved_commit() {
        let repo = TestRepo::new();
        repo.write_file(".tndm/tickets/TNDM-1/meta.toml", "base\n");
        repo.commit_all("base");
        let base_commit = repo.rev_parse("HEAD^{commit}");

        repo.write_file(".tndm/tickets/TNDM-2/meta.toml", "moved\n");
        repo.commit_all("move head");

        let paths = list_ref_ticket_paths(repo.root(), &base_commit).expect("list ticket paths");

        assert_eq!(paths, vec![PathBuf::from(".tndm/tickets/TNDM-1/meta.toml")]);
    }

    #[test]
    fn resolve_materialized_ticket_path_rejects_parent_components() {
        let root = Path::new("/snapshot");
        let error = resolve_materialized_ticket_path(
            root,
            Path::new(".tndm/tickets/../../escape.txt"),
            "stable",
        )
        .expect_err("unsafe path should fail");

        let message = error.to_string();
        assert!(message.contains("unsafe ticket path"));
        assert!(message.contains("stable"));
        assert!(!message.contains("/snapshot"));
    }

    #[test]
    fn resolve_materialized_ticket_path_rejects_absolute_paths() {
        let root = Path::new("/snapshot");
        let error = resolve_materialized_ticket_path(root, Path::new("/tmp/escape.txt"), "stable")
            .expect_err("absolute path should fail");

        let message = error.to_string();
        assert!(message.contains("unsafe ticket path"));
        assert!(message.contains("stable"));
        assert!(!message.contains("/snapshot"));
    }

    #[test]
    fn resolve_materialized_ticket_path_accepts_ticket_file_under_snapshot_root() {
        let root = Path::new("/snapshot");
        let path = resolve_materialized_ticket_path(
            root,
            Path::new(".tndm/tickets/TNDM-1/meta.toml"),
            "stable",
        )
        .expect("safe path should resolve");

        assert_eq!(
            path,
            PathBuf::from("/snapshot/.tndm/tickets/TNDM-1/meta.toml")
        );
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

        fn write_file(&self, relative_path: &str, contents: &str) {
            let path = self.root().join(relative_path);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).expect("create parent dirs");
            }
            std::fs::write(path, contents).expect("write file");
        }

        fn commit_all(&self, message: &str) {
            run_git(self.root(), &["add", "."]);
            run_git(self.root(), &["commit", "-m", message]);
        }

        fn rev_parse(&self, reference: &str) -> String {
            String::from_utf8(run_git_output(
                self.root(),
                &["rev-parse", "--verify", reference],
            ))
            .expect("utf8 rev-parse output")
            .trim()
            .to_string()
        }
    }

    fn run_git(repo_root: &Path, args: &[&str]) {
        let _ = run_git_output(repo_root, args);
    }

    fn run_git_output(repo_root: &Path, args: &[&str]) -> Vec<u8> {
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

        output.stdout
    }
}
