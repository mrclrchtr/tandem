use std::env;
use std::path::PathBuf;

use tandem_core::ticket::TicketId;
use tandem_storage::{FileTicketStore, TandemConfig, discover_repo_root, load_config};

use super::util::parse_ticket_id_input;

/// Shared context for ticket CLI handlers.
///
/// Eliminates the repetitive preamble (`env::current_dir → discover_repo_root →
/// load_config → FileTicketStore::new → parse_ticket_id_input`) that was
/// duplicated across every handler in `ticket/` and `doc.rs`.
pub(crate) struct TicketCtx {
    pub(crate) store: FileTicketStore,
    pub(crate) repo_root: PathBuf,
    pub(crate) config: TandemConfig,
}

impl TicketCtx {
    /// Create a new context by discovering the repo root, loading config,
    /// and initializing the file-based ticket store.
    pub(crate) fn new() -> anyhow::Result<Self> {
        let current_dir = env::current_dir().map_err(|error| anyhow::anyhow!("{error}"))?;
        let repo_root =
            discover_repo_root(&current_dir).map_err(|error| anyhow::anyhow!("{error}"))?;
        let config = load_config(&repo_root).map_err(|error| anyhow::anyhow!("{error}"))?;
        let store = FileTicketStore::new(repo_root.clone());
        Ok(Self {
            store,
            repo_root,
            config,
        })
    }

    /// Parse a ticket ID string, applying the configured prefix if needed.
    pub(crate) fn resolve_id(&self, input: &str) -> anyhow::Result<TicketId> {
        Ok(parse_ticket_id_input(input, &self.config.id_prefix)?)
    }

    /// Return the configured ticket ID prefix.
    pub(crate) fn id_prefix(&self) -> &str {
        &self.config.id_prefix
    }
}
