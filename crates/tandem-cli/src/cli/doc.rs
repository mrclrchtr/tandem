use std::{
    fs,
    path::{Component, Path, PathBuf},
};

use tandem_core::ports::TicketStore;
use tandem_core::ticket::{Ticket, TicketDocument, TicketId};
use tandem_storage::{fingerprint_file, ticket_dir};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

use super::ticket_ctx::TicketCtx;

pub(crate) fn handle_doc_create(
    id: String,
    name: String,
    path: Option<String>,
    json: bool,
) -> anyhow::Result<()> {
    let ctx = TicketCtx::new()?;
    let ticket_id = ctx.resolve_id(&id)?;

    let mut ticket = ctx
        .store
        .load_ticket(&ticket_id)
        .map_err(|error| anyhow::anyhow!("{error}"))?;

    let requested_rel_path = path
        .as_deref()
        .map(normalize_ticket_relative_doc_path)
        .transpose()?;

    // Check if document with this name already exists
    if let Some(existing) = ticket.meta.documents.iter().find(|d| d.name == name) {
        if let Some(requested_path) = requested_rel_path.as_ref()
            && requested_path != &existing.path
        {
            anyhow::bail!(
                "document {} is already registered at {}; requested path {} does not match",
                name,
                existing.path,
                requested_path
            );
        }

        // Already registered — return the existing path
        let doc_path = ticket_dir(&ctx.repo_root, &ticket_id).join(&existing.path);
        if json {
            println!(
                "{}",
                serde_json::json!({
                    "ticket_id": ticket_id.as_str(),
                    "name": name,
                    "path": doc_path.to_string_lossy(),
                    "existing": true,
                })
            );
        } else {
            println!("{}", doc_path.display());
        }
        return Ok(());
    }

    let rel_path = requested_rel_path.unwrap_or_else(|| format!("{name}.md"));
    if let Some(existing) = ticket
        .meta
        .documents
        .iter()
        .find(|doc| doc.path == rel_path)
    {
        anyhow::bail!(
            "document path {} is already registered as document {}",
            rel_path,
            existing.name
        );
    }

    let abs_path = ticket_dir(&ctx.repo_root, &ticket_id).join(&rel_path);
    if abs_path.exists() {
        anyhow::bail!(
            "document path {} already exists on disk; refusing to overwrite it",
            rel_path
        );
    }

    if let Some(parent) = abs_path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            anyhow::anyhow!("failed to create directory {}: {error}", parent.display())
        })?;
    }

    // Write an empty template
    fs::write(&abs_path, format!("# {name}\n\n"))
        .map_err(|error| anyhow::anyhow!("failed to write {}: {error}", abs_path.display()))?;

    // Register the document
    ticket.meta.documents.push(TicketDocument {
        name: name.clone(),
        path: rel_path.clone(),
    });

    recompute_ticket_document_fingerprints(&ctx.repo_root, &ticket_id, &mut ticket)?;

    // Bump revision and update timestamp
    ticket.state.revision += 1;
    ticket.state.updated_at = OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|error| anyhow::anyhow!("failed to format timestamp: {error}"))?;

    // Write canonical meta and state
    let meta_path = ticket_dir(&ctx.repo_root, &ticket_id).join("meta.toml");
    let state_path = ticket_dir(&ctx.repo_root, &ticket_id).join("state.toml");
    fs::write(&meta_path, ticket.meta.to_canonical_toml())
        .map_err(|error| anyhow::anyhow!("failed to write {}: {error}", meta_path.display()))?;
    fs::write(&state_path, ticket.state.to_canonical_toml())
        .map_err(|error| anyhow::anyhow!("failed to write {}: {error}", state_path.display()))?;

    if json {
        println!(
            "{}",
            serde_json::json!({
                "ticket_id": ticket_id.as_str(),
                "name": name,
                "path": abs_path.to_string_lossy(),
                "existing": false,
            })
        );
    } else {
        println!("{}", abs_path.display());
    }
    Ok(())
}

pub(crate) fn recompute_ticket_document_fingerprints(
    repo_root: &Path,
    ticket_id: &TicketId,
    ticket: &mut Ticket,
) -> anyhow::Result<()> {
    let mut fingerprints = std::collections::BTreeMap::new();
    for doc in &ticket.meta.documents {
        let doc_path = ticket_dir(repo_root, ticket_id).join(&doc.path);
        if doc_path.is_file() {
            let hash = fingerprint_file(&doc_path).map_err(|e| anyhow::anyhow!("{e}"))?;
            fingerprints.insert(doc.name.clone(), hash);
        }
    }
    ticket.state.document_fingerprints = fingerprints;
    Ok(())
}

pub(crate) fn normalize_ticket_relative_doc_path(path: &str) -> anyhow::Result<String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        anyhow::bail!("document path must not be empty");
    }

    let candidate = Path::new(trimmed);
    if candidate.is_absolute() {
        anyhow::bail!("document path must be ticket-relative");
    }

    let mut normalized = PathBuf::new();
    for component in candidate.components() {
        match component {
            Component::Normal(part) => normalized.push(part),
            Component::CurDir => {}
            Component::ParentDir => {
                anyhow::bail!("document path must not traverse outside the ticket directory")
            }
            Component::RootDir | Component::Prefix(_) => {
                anyhow::bail!("document path must be ticket-relative")
            }
        }
    }

    if normalized.as_os_str().is_empty() {
        anyhow::bail!("document path must not be empty");
    }

    Ok(normalized.to_string_lossy().into_owned())
}
