use std::{
    fmt, fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;

use tandem_core::{
    ports::TicketStore,
    ticket::{
        NewTicket, Ticket, TicketId, TicketMeta, TicketPriority, TicketState, TicketStatus,
        TicketType,
    },
};

#[derive(Debug, Clone)]
pub struct FileTicketStore {
    repo_root: PathBuf,
}

impl FileTicketStore {
    pub fn new(repo_root: PathBuf) -> Self {
        Self { repo_root }
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
            content_template: "## Description\n\n## Design\n\n## Acceptance\n\n## Notes\n"
                .to_string(),
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
    ticket_type: String,
    priority: String,
    depends_on: Option<Vec<String>>,
    tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct RawTicketState {
    schema_version: Option<u32>,
    status: String,
    updated_at: String,
    revision: u64,
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
    let mut current = start;

    loop {
        let tndm_dir = current.join(".tndm");
        if tndm_dir.is_dir() {
            return Ok(current.to_path_buf());
        }

        let git_path = current.join(".git");
        if git_path.is_dir() || git_path.is_file() {
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

impl TicketStore for FileTicketStore {
    type Error = StorageError;

    #[allow(clippy::disallowed_methods)]
    fn create_ticket(&self, ticket: NewTicket) -> Result<Ticket, Self::Error> {
        fs::create_dir_all(tickets_dir(&self.repo_root)).map_err(|error| {
            StorageError::new(format!(
                "failed to create tickets directory {}: {error}",
                tickets_dir(&self.repo_root).display()
            ))
        })?;

        let ticket_path = ticket_dir(&self.repo_root, &ticket.meta.id);
        fs::create_dir(&ticket_path).map_err(|error| {
            StorageError::new(format!(
                "failed to create ticket directory {}: {error}",
                ticket_path.display()
            ))
        })?;

        let meta_path = ticket_path.join("meta.toml");
        let state_path = ticket_path.join("state.toml");
        let content_path = ticket_path.join("content.md");

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

        Ok(Ticket {
            meta: ticket.meta,
            state: ticket.state,
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
        meta.ticket_type = TicketType::parse(&raw_meta.ticket_type).map_err(|error| {
            StorageError::new(format!("invalid type in {}: {error}", meta_path.display()))
        })?;
        meta.priority = TicketPriority::parse(&raw_meta.priority).map_err(|error| {
            StorageError::new(format!(
                "invalid priority in {}: {error}",
                meta_path.display()
            ))
        })?;
        meta.depends_on = depends_on;
        meta.tags = tags;

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

    fn ticket_exists(&self, id: &TicketId) -> Result<bool, Self::Error> {
        Ok(ticket_dir(&self.repo_root, id).is_dir())
    }
}
