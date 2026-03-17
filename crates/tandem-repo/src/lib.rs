#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use std::{
    fmt, fs,
    path::{Path, PathBuf},
    process::Command,
};

use tandem_core::{
    awareness::TicketSnapshot,
    ports::{AwarenessRefMaterializer, AwarenessSnapshotProvider, RepoContext},
};
use tandem_storage::{StorageError, load_ticket_snapshot};

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

pub struct RefSnapshot {
    temp_dir: tempfile::TempDir,
    canonical_path: PathBuf,
}

impl fmt::Debug for RefSnapshot {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RefSnapshot")
            .field("path", &self.temp_dir.path())
            .finish()
    }
}

impl RefSnapshot {
    fn new(temp_dir: tempfile::TempDir) -> Self {
        let canonical_path = temp_dir
            .path()
            .canonicalize()
            .unwrap_or_else(|_| temp_dir.path().to_path_buf());
        Self {
            temp_dir,
            canonical_path,
        }
    }

    pub fn path(&self) -> &Path {
        self.temp_dir.path()
    }

    pub fn sanitize_error_text(&self, text: &str) -> String {
        let raw = self.temp_dir.path().to_string_lossy();
        let canonical = self.canonical_path.to_string_lossy();

        let mut result = text.replace(raw.as_ref(), "<ref-snapshot>");
        if canonical != raw {
            result = result.replace(canonical.as_ref(), "<ref-snapshot>");
        }

        // Normalize backslashes (Windows paths)
        let normalized_raw = raw.replace('\\', "/");
        if normalized_raw != raw.as_ref() {
            result = result.replace(&normalized_raw, "<ref-snapshot>");
        }

        result
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

    fn storage(error: StorageError) -> Self {
        Self::new(format!("storage error: {error}"))
    }

    fn ref_snapshot_storage(reference: &str, temp_root: &Path, error: StorageError) -> Self {
        let raw_message = error.to_string();
        let normalized_root = temp_root.to_string_lossy().replace('\\', "/");
        let sanitized = raw_message
            .replace(temp_root.to_string_lossy().as_ref(), "<ref-snapshot>")
            .replace(&normalized_root, "<ref-snapshot>");

        Self::new(format!(
            "failed to load materialized snapshot for ref `{reference}`: {sanitized}"
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

impl AwarenessSnapshotProvider for GitAwarenessProvider {
    type Error = RepoError;

    fn load_current_snapshot(&self) -> Result<TicketSnapshot, Self::Error> {
        load_ticket_snapshot(&self.repo_root).map_err(RepoError::storage)
    }

    fn load_snapshot_for_ref(&self, reference: &str) -> Result<TicketSnapshot, Self::Error> {
        match self.materialize_ref_snapshot(reference)? {
            None => Ok(TicketSnapshot::default()),
            Some(snapshot) => load_ticket_snapshot(snapshot.path()).map_err(|error| {
                RepoError::ref_snapshot_storage(reference, snapshot.path(), error)
            }),
        }
    }
}

impl AwarenessRefMaterializer for GitAwarenessProvider {
    type Error = RepoError;
    type Snapshot = RefSnapshot;

    fn materialize_ref_snapshot(&self, reference: &str) -> Result<Option<RefSnapshot>, RepoError> {
        let resolved_ref = format!("{reference}^{{commit}}");
        run_git(&self.repo_root, &["rev-parse", "--verify", &resolved_ref])?;

        let ticket_paths = list_ref_ticket_paths(&self.repo_root, reference)?;
        if ticket_paths.is_empty() {
            return Ok(None);
        }

        let temp_dir = tempfile::tempdir().map_err(|error| {
            RepoError::new(format!(
                "failed to create temp snapshot root for ref `{reference}`: {error}"
            ))
        })?;

        write_ref_ticket_tree(temp_dir.path(), &self.repo_root, reference, &ticket_paths)?;
        Ok(Some(RefSnapshot::new(temp_dir)))
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

fn list_ref_ticket_paths(repo_root: &Path, reference: &str) -> Result<Vec<PathBuf>, RepoError> {
    let output = run_git(
        repo_root,
        &[
            "ls-tree",
            "-r",
            "--name-only",
            reference,
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

fn write_ref_ticket_tree(
    destination_root: &Path,
    repo_root: &Path,
    reference: &str,
    ticket_paths: &[PathBuf],
) -> Result<(), RepoError> {
    for ticket_path in ticket_paths {
        let ticket_path_string = ticket_path.to_string_lossy().to_string();
        let blob = run_git(
            repo_root,
            &["show", &format!("{reference}:{ticket_path_string}")],
        )?;

        let destination_path = destination_root.join(ticket_path);
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
