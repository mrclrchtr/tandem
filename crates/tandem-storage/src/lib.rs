use std::{
    collections::BTreeMap,
    fmt, fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;
use sha2::{Digest, Sha256};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

use tandem_core::{
    awareness::TicketSnapshot,
    ports::TicketStore,
    ticket::{
        NewTicket, Ticket, TicketEffort, TicketId, TicketMeta, TicketPriority, TicketState,
        TicketStatus, TicketType,
    },
};

#[derive(Debug, Clone)]
pub struct FileTicketStore {
    repo_root: PathBuf,
}

/// Compute the SHA-256 fingerprint of a byte slice.
/// Returns `sha256:<hex>` string.
pub fn fingerprint_bytes(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!(
        "sha256:{}",
        hasher
            .finalize()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>()
    )
}

/// Compute the SHA-256 fingerprint of a file's contents.
/// Returns `sha256:<hex>` string.
#[allow(clippy::disallowed_methods)]
pub fn fingerprint_file(path: &Path) -> Result<String, StorageError> {
    let contents = fs::read(path).map_err(|error| {
        StorageError::new(format!(
            "failed to read {} for fingerprint: {error}",
            path.display()
        ))
    })?;
    Ok(fingerprint_bytes(&contents))
}

impl FileTicketStore {
    pub fn new(repo_root: PathBuf) -> Self {
        Self { repo_root }
    }

    /// Recompute fingerprints for all registered documents and update the ticket.
    /// Returns the updated ticket with fresh fingerprints and bumped revision.
    #[allow(clippy::disallowed_methods)]
    pub fn sync_ticket_documents(&self, id: &TicketId) -> Result<Ticket, StorageError> {
        let mut ticket = self.load_ticket(id)?;

        let ticket_path = ticket_dir(&self.repo_root, id);
        for doc in &ticket.meta.documents {
            let doc_path = ticket_path.join(&doc.path);
            if doc_path.is_file() {
                let fp = fingerprint_file(&doc_path)?;
                ticket
                    .state
                    .document_fingerprints
                    .insert(doc.name.clone(), fp);
            }
        }

        ticket.state.revision += 1;
        ticket.state.updated_at = OffsetDateTime::now_utc()
            .format(&Rfc3339)
            .map_err(|error| format!("failed to format timestamp: {error}"))
            .map_err(StorageError::new)?;

        // Persist the updated state
        let state_path = ticket_path.join("state.toml");
        fs::write(&state_path, ticket.state.to_canonical_toml()).map_err(|error| {
            StorageError::new(format!(
                "failed to write {} during sync: {error}",
                state_path.display()
            ))
        })?;

        Ok(ticket)
    }

    /// Check which registered documents have stale fingerprints (content edited without sync).
    /// Returns a list of (document_name, actual_fingerprint) pairs that differ from stored ones.
    #[allow(clippy::disallowed_methods)]
    pub fn document_drift(&self, id: &TicketId) -> Result<Vec<(String, String)>, StorageError> {
        let ticket = self.load_ticket(id)?;
        let ticket_path = ticket_dir(&self.repo_root, id);
        let mut drift = Vec::new();

        for doc in &ticket.meta.documents {
            let doc_path = ticket_path.join(&doc.path);
            if !doc_path.is_file() {
                drift.push((doc.name.clone(), "MISSING".to_string()));
                continue;
            }
            let actual_fp = fingerprint_file(&doc_path)?;
            let stored_fp = ticket.state.document_fingerprints.get(&doc.name);
            match stored_fp {
                Some(fp) if fp == &actual_fp => {}
                _ => drift.push((doc.name.clone(), actual_fp)),
            }
        }

        Ok(drift)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TandemConfig {
    pub id_prefix: String,
    pub content_template: String,
}

impl Default for TandemConfig {
    fn default() -> Self {
        Self {
            id_prefix: "TNDM".to_string(),
            content_template: tandem_core::ticket::DEFAULT_CONTENT_TEMPLATE.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageError {
    message: String,
}

impl StorageError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for StorageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.message)
    }
}

impl std::error::Error for StorageError {}

#[derive(Debug, Deserialize)]
struct RawConfig {
    schema_version: Option<u32>,
    id: Option<RawIdConfig>,
    templates: Option<RawTemplatesConfig>,
}

#[derive(Debug, Deserialize)]
struct RawIdConfig {
    prefix: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawTemplatesConfig {
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawTicketMeta {
    schema_version: Option<u32>,
    id: String,
    title: String,
    #[serde(rename = "type")]
    ticket_type: Option<String>,
    priority: Option<String>,
    effort: Option<String>,
    depends_on: Option<Vec<String>>,
    tags: Option<Vec<String>>,
    documents: Option<Vec<RawTicketDocument>>,
}

#[derive(Debug, Deserialize)]
struct RawTicketDocument {
    name: String,
    path: String,
}

#[derive(Debug, Deserialize)]
struct RawTicketState {
    schema_version: Option<u32>,
    status: String,
    updated_at: String,
    revision: u64,
    document_fingerprints: Option<BTreeMap<String, String>>,
}

pub fn tandem_dir(repo_root: &Path) -> PathBuf {
    repo_root.join(".tndm")
}

pub fn tickets_dir(repo_root: &Path) -> PathBuf {
    tandem_dir(repo_root).join("tickets")
}

pub fn ticket_dir(repo_root: &Path, id: &TicketId) -> PathBuf {
    tickets_dir(repo_root).join(id.as_str())
}

#[allow(clippy::disallowed_methods)]
pub fn discover_repo_root(start: &Path) -> Result<PathBuf, StorageError> {
    // First pass: .git is the authoritative repo root marker.
    let mut current = start;
    loop {
        let git_path = current.join(".git");
        if git_path.is_dir() || git_path.is_file() {
            return Ok(current.to_path_buf());
        }
        if let Some(parent) = current.parent() {
            current = parent;
        } else {
            break;
        }
    }

    // Fallback: accept a standalone .tndm directory.
    let mut current = start;
    loop {
        let tndm_dir = current.join(".tndm");
        if tndm_dir.is_dir() {
            return Ok(current.to_path_buf());
        }
        if let Some(parent) = current.parent() {
            current = parent;
        } else {
            return Err(StorageError::new(
                "no repository markers found (.tndm or .git)",
            ));
        }
    }
}

#[allow(clippy::disallowed_methods)]
pub fn load_config(repo_root: &Path) -> Result<TandemConfig, StorageError> {
    let config_path = tandem_dir(repo_root).join("config.toml");

    let config_text = match fs::read_to_string(&config_path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(TandemConfig::default());
        }
        Err(error) => {
            return Err(StorageError::new(format!(
                "failed to read {}: {error}",
                config_path.display()
            )));
        }
    };

    let raw: RawConfig = toml::from_str(&config_text).map_err(|error| {
        StorageError::new(format!(
            "failed to parse {}: {error}",
            config_path.display()
        ))
    })?;

    if let Some(version) = raw.schema_version
        && version != 1
    {
        return Err(StorageError::new(format!(
            "unsupported schema_version `{version}` in {}",
            config_path.display()
        )));
    }

    let mut config = TandemConfig::default();

    if let Some(prefix) = raw.id.and_then(|id| id.prefix) {
        config.id_prefix = prefix;
    }

    if let Some(content_template) = raw.templates.and_then(|templates| templates.content) {
        config.content_template = content_template;
    }

    Ok(config)
}

pub fn load_ticket_snapshot(repo_root: &Path) -> Result<TicketSnapshot, StorageError> {
    let store = FileTicketStore::new(repo_root.to_path_buf());
    let mut tickets = BTreeMap::new();

    for id in store.list_ticket_ids()? {
        let ticket = store.load_ticket(&id)?;
        tickets.insert(id, ticket);
    }

    Ok(TicketSnapshot { tickets })
}

impl TicketStore for FileTicketStore {
    type Error = StorageError;

    #[allow(clippy::disallowed_methods)]
    fn create_ticket(&self, ticket: NewTicket) -> Result<Ticket, Self::Error> {
        let updated_at = OffsetDateTime::now_utc()
            .format(&Rfc3339)
            .map_err(|error| {
                StorageError::new(format!("failed to format ticket timestamp: {error}"))
            })?;
        let mut state = TicketState::initial(updated_at).map_err(|error| {
            StorageError::new(format!("failed to build initial ticket state: {error}"))
        })?;

        // Compute fingerprints for all registered documents
        let mut fingerprints = BTreeMap::new();
        for doc in &ticket.meta.documents {
            // For content.md, fingerprint will be computed after writing
            // but we store the template for now
            if doc.name == "content" {
                fingerprints.insert(
                    doc.name.clone(),
                    fingerprint_bytes(ticket.content.as_bytes()),
                );
            }
            // Future documents fingerprint after creation
        }
        state.document_fingerprints = fingerprints;

        let tickets_path = tickets_dir(&self.repo_root);
        fs::create_dir_all(&tickets_path).map_err(|error| {
            StorageError::new(format!(
                "failed to create tickets directory {}: {error}",
                tickets_path.display()
            ))
        })?;

        let ticket_path = ticket_dir(&self.repo_root, &ticket.meta.id);
        atomic_write_dir(&ticket_path, false, |staging| {
            let meta_path = staging.join("meta.toml");
            let state_path = staging.join("state.toml");
            let content_path = staging.join("content.md");

            fs::write(&meta_path, ticket.meta.to_canonical_toml()).map_err(|error| {
                StorageError::new(format!("failed to write {}: {error}", meta_path.display()))
            })?;

            fs::write(&state_path, state.to_canonical_toml()).map_err(|error| {
                StorageError::new(format!("failed to write {}: {error}", state_path.display()))
            })?;

            fs::write(&content_path, &ticket.content).map_err(|error| {
                StorageError::new(format!(
                    "failed to write {}: {error}",
                    content_path.display()
                ))
            })?;

            Ok(())
        })?;

        Ok(Ticket {
            meta: ticket.meta,
            state,
            content: ticket.content,
        })
    }

    #[allow(clippy::disallowed_methods)]
    fn load_ticket(&self, id: &TicketId) -> Result<Ticket, Self::Error> {
        let ticket_path = ticket_dir(&self.repo_root, id);
        let dir_name = ticket_path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| {
                StorageError::new(format!(
                    "invalid ticket directory path {}",
                    ticket_path.display()
                ))
            })?;

        if dir_name != id.as_str() {
            return Err(StorageError::new(format!(
                "ticket directory name `{dir_name}` does not match requested id `{}`",
                id.as_str()
            )));
        }

        let meta_path = ticket_path.join("meta.toml");
        let state_path = ticket_path.join("state.toml");
        let content_path = ticket_path.join("content.md");

        let meta_text = fs::read_to_string(&meta_path).map_err(|error| {
            StorageError::new(format!("failed to read {}: {error}", meta_path.display()))
        })?;
        let state_text = fs::read_to_string(&state_path).map_err(|error| {
            StorageError::new(format!("failed to read {}: {error}", state_path.display()))
        })?;
        let content = fs::read_to_string(&content_path).map_err(|error| {
            StorageError::new(format!(
                "failed to read {}: {error}",
                content_path.display()
            ))
        })?;

        let raw_meta: RawTicketMeta = toml::from_str(&meta_text).map_err(|error| {
            StorageError::new(format!("failed to parse {}: {error}", meta_path.display()))
        })?;
        let raw_state: RawTicketState = toml::from_str(&state_text).map_err(|error| {
            StorageError::new(format!("failed to parse {}: {error}", state_path.display()))
        })?;

        if raw_meta.schema_version != Some(1) {
            return Err(StorageError::new(format!(
                "unsupported or missing schema_version in {}",
                meta_path.display()
            )));
        }

        if raw_state.schema_version != Some(1) {
            return Err(StorageError::new(format!(
                "unsupported or missing schema_version in {}",
                state_path.display()
            )));
        }

        if raw_meta.id != id.as_str() {
            return Err(StorageError::new(format!(
                "ticket id mismatch: requested `{}`, found `{}` in {}",
                id.as_str(),
                raw_meta.id,
                meta_path.display()
            )));
        }

        let mut depends_on = raw_meta.depends_on.unwrap_or_default();
        depends_on.sort();
        depends_on.dedup();
        let depends_on = depends_on
            .into_iter()
            .map(|dependency_id| {
                TicketId::parse(dependency_id).map_err(|error| {
                    StorageError::new(format!(
                        "invalid depends_on ticket id in {}: {error}",
                        meta_path.display()
                    ))
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        let mut tags = raw_meta.tags.unwrap_or_default();
        tags.sort();
        tags.dedup();

        let mut meta = TicketMeta::new(id.clone(), raw_meta.title).map_err(|error| {
            StorageError::new(format!("invalid meta in {}: {error}", meta_path.display()))
        })?;
        if let Some(ticket_type) = raw_meta.ticket_type {
            meta.ticket_type = TicketType::parse(&ticket_type).map_err(|error| {
                StorageError::new(format!("invalid type in {}: {error}", meta_path.display()))
            })?;
        }
        if let Some(priority) = raw_meta.priority {
            meta.priority = TicketPriority::parse(&priority).map_err(|error| {
                StorageError::new(format!(
                    "invalid priority in {}: {error}",
                    meta_path.display()
                ))
            })?;
        }
        if let Some(effort) = raw_meta.effort {
            meta.effort = Some(TicketEffort::parse(&effort).map_err(|error| {
                StorageError::new(format!(
                    "invalid effort in {}: {error}",
                    meta_path.display()
                ))
            })?);
        }
        meta.depends_on = depends_on;
        meta.tags = tags;

        // Legacy ticket migration: if [[documents]] was not present, inject default
        if raw_meta.documents.is_none() {
            meta.documents = vec![tandem_core::ticket::TicketDocument {
                name: "content".to_string(),
                path: "content.md".to_string(),
            }];
        } else if let Some(raw_docs) = raw_meta.documents {
            meta.documents = raw_docs
                .into_iter()
                .map(|d| tandem_core::ticket::TicketDocument {
                    name: d.name,
                    path: d.path,
                })
                .collect();
        }

        let mut state =
            TicketState::new(raw_state.updated_at, raw_state.revision).map_err(|error| {
                StorageError::new(format!(
                    "invalid state in {}: {error}",
                    state_path.display()
                ))
            })?;
        state.status = TicketStatus::parse(&raw_state.status).map_err(|error| {
            StorageError::new(format!(
                "invalid status in {}: {error}",
                state_path.display()
            ))
        })?;
        state.document_fingerprints = raw_state.document_fingerprints.unwrap_or_default();

        Ok(Ticket {
            meta,
            state,
            content,
        })
    }

    #[allow(clippy::disallowed_methods)]
    fn list_ticket_ids(&self) -> Result<Vec<TicketId>, Self::Error> {
        let tickets_path = tickets_dir(&self.repo_root);

        let entries = match fs::read_dir(&tickets_path) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => {
                return Err(StorageError::new(format!(
                    "failed to read {}: {error}",
                    tickets_path.display()
                )));
            }
        };

        let mut ids = Vec::new();

        for entry in entries {
            let entry = entry.map_err(|error| {
                StorageError::new(format!(
                    "failed to read entry in {}: {error}",
                    tickets_path.display()
                ))
            })?;

            let file_type = entry.file_type().map_err(|error| {
                StorageError::new(format!(
                    "failed to read file type for {}: {error}",
                    entry.path().display()
                ))
            })?;

            if !file_type.is_dir() {
                continue;
            }

            let dir_name = entry.file_name();
            let dir_name = dir_name.to_str().ok_or_else(|| {
                StorageError::new(format!(
                    "ticket directory name is not valid UTF-8: {}",
                    entry.path().display()
                ))
            })?;

            if dir_name.starts_with('.') && dir_name.ends_with(".tmp") {
                continue;
            }

            let id = TicketId::parse(dir_name).map_err(|error| {
                StorageError::new(format!(
                    "invalid ticket directory `{dir_name}` in {}: {error}",
                    tickets_path.display()
                ))
            })?;

            ids.push(id);
        }

        ids.sort();
        Ok(ids)
    }

    #[allow(clippy::disallowed_methods)]
    fn update_ticket(&self, ticket: &Ticket) -> Result<Ticket, Self::Error> {
        let ticket_path = ticket_dir(&self.repo_root, &ticket.meta.id);
        if !ticket_path.is_dir() {
            return Err(StorageError::new(format!(
                "ticket directory does not exist: {}",
                ticket_path.display()
            )));
        }

        atomic_write_dir(&ticket_path, true, |staging| {
            let meta_path = staging.join("meta.toml");
            let state_path = staging.join("state.toml");
            let content_path = staging.join("content.md");

            fs::write(&meta_path, ticket.meta.to_canonical_toml()).map_err(|error| {
                StorageError::new(format!("failed to write {}: {error}", meta_path.display()))
            })?;

            fs::write(&state_path, ticket.state.to_canonical_toml()).map_err(|error| {
                StorageError::new(format!("failed to write {}: {error}", state_path.display()))
            })?;

            fs::write(&content_path, &ticket.content).map_err(|error| {
                StorageError::new(format!(
                    "failed to write {}: {error}",
                    content_path.display()
                ))
            })?;

            // Copy all registered extra documents from the existing ticket directory
            for doc in &ticket.meta.documents {
                if doc.name == "content" {
                    continue;
                }
                let src = ticket_path.join(&doc.path);
                let dst = staging.join(&doc.path);
                if let Some(parent) = dst.parent() {
                    fs::create_dir_all(parent).map_err(|error| {
                        StorageError::new(format!(
                            "failed to create directory for {}: {error}",
                            dst.display()
                        ))
                    })?;
                }
                if src.is_file() {
                    fs::copy(&src, &dst).map_err(|error| {
                        StorageError::new(format!(
                            "failed to copy {} to {}: {error}",
                            src.display(),
                            dst.display()
                        ))
                    })?;
                }
            }

            Ok(())
        })?;

        Ok(ticket.clone())
    }

    fn ticket_exists(&self, id: &TicketId) -> Result<bool, Self::Error> {
        Ok(ticket_dir(&self.repo_root, id).is_dir())
    }
}

/// Write files into a temporary staging directory, then atomically rename to the final path.
///
/// When `allow_overwrite` is false, the rename fails if `final_path` already exists
/// (e.g. `create_ticket` must not overwrite an existing ticket directory).
/// When `allow_overwrite` is true, an existing directory is moved aside first,
/// then the staging directory takes its place. If the second rename fails, the
/// original is restored (rollback).
///
/// On success, `final_path` is replaced atomically (same filesystem).
/// On failure in the non-overwrite case, `final_path` is untouched.
/// On failure in the overwrite case, the original is restored.
fn atomic_write_dir<F>(
    final_path: &Path,
    allow_overwrite: bool,
    write_fn: F,
) -> Result<(), StorageError>
where
    F: FnOnce(&Path) -> Result<(), StorageError>,
{
    let parent = final_path.parent().ok_or_else(|| {
        StorageError::new(format!(
            "cannot determine parent of {}",
            final_path.display()
        ))
    })?;
    let dir_name = final_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| StorageError::new(format!("invalid final path {}", final_path.display())))?;
    let staging_path = parent.join(format!(".{dir_name}.tmp"));
    let old_path = parent.join(format!(".{dir_name}.old.tmp"));

    // Clean stale staging directory
    if staging_path.is_dir() {
        fs::remove_dir_all(&staging_path).map_err(|error| {
            StorageError::new(format!(
                "failed to remove stale staging directory {}: {error}",
                staging_path.display()
            ))
        })?;
    }

    // Clean stale backup directory (from interrupted overwrite)
    if old_path.is_dir() {
        fs::remove_dir_all(&old_path).map_err(|error| {
            StorageError::new(format!(
                "failed to remove stale backup directory {}: {error}",
                old_path.display()
            ))
        })?;
    }

    // Create staging directory
    fs::create_dir(&staging_path).map_err(|error| {
        StorageError::new(format!(
            "failed to create staging directory {}: {error}",
            staging_path.display()
        ))
    })?;

    // Execute write closure
    let result = write_fn(&staging_path);

    if result.is_err() {
        let _ = fs::remove_dir_all(&staging_path);
        return result;
    }

    if allow_overwrite && final_path.exists() {
        // Two-phase rename: move existing aside, then move staging in
        fs::rename(final_path, &old_path).map_err(|error| {
            let _ = fs::remove_dir_all(&staging_path);
            StorageError::new(format!(
                "failed to move aside {}: {error}",
                final_path.display()
            ))
        })?;

        fs::rename(&staging_path, final_path).map_err(|error| {
            // Rollback: restore original
            let _ = fs::rename(&old_path, final_path);
            let _ = fs::remove_dir_all(&staging_path);
            StorageError::new(format!(
                "failed to finalize {}: {error}",
                final_path.display()
            ))
        })?;

        // Success: remove backup
        let _ = fs::remove_dir_all(&old_path);
    } else {
        // Single rename (fails if destination exists, which is desired for create)
        fs::rename(&staging_path, final_path).map_err(|error| {
            let _ = fs::remove_dir_all(&staging_path);
            StorageError::new(format!(
                "failed to finalize {}: {error}",
                final_path.display()
            ))
        })?;
    }

    Ok(())
}
