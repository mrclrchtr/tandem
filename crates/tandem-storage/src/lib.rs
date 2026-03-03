use std::{
    fmt, fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;

use tandem_core::{
    ports::TicketStore,
    ticket::{NewTicket, Ticket, TicketId},
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

    fn not_implemented(operation: &str) -> Self {
        Self {
            message: format!("storage operation `{operation}` is not implemented"),
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

    fn load_ticket(&self, _id: &TicketId) -> Result<Ticket, Self::Error> {
        Err(StorageError::not_implemented("load_ticket"))
    }

    fn list_ticket_ids(&self) -> Result<Vec<TicketId>, Self::Error> {
        Err(StorageError::not_implemented("list_ticket_ids"))
    }

    fn ticket_exists(&self, _id: &TicketId) -> Result<bool, Self::Error> {
        Err(StorageError::not_implemented("ticket_exists"))
    }
}
