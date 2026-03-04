#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use std::{fmt, fs, path::Path};

use serde::Deserialize;

use tandem_core::{
    ports::TicketStore,
    ticket::{NewTicket, Ticket, TicketId},
};

#[derive(Debug, Default, Clone, Copy)]
pub struct FileTicketStore;

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

pub fn load_config(repo_root: &Path) -> Result<TandemConfig, StorageError> {
    let config_path = repo_root.join(".tndm").join("config.toml");

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

    fn create_ticket(&self, _ticket: NewTicket) -> Result<Ticket, Self::Error> {
        Err(StorageError::not_implemented("create_ticket"))
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
