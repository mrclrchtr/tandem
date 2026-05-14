use std::{env, fs};

use tandem_core::ports::TicketStore;
use tandem_core::ticket::TicketDocument;
use tandem_storage::{
    FileTicketStore, discover_repo_root, fingerprint_file, load_config, ticket_dir,
};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

use super::util::parse_ticket_id_input;

pub(crate) fn handle_doc_create(id: String, name: String, json: bool) -> anyhow::Result<()> {
    let current_dir = env::current_dir().map_err(|error| anyhow::anyhow!("{error}"))?;
    let repo_root = discover_repo_root(&current_dir).map_err(|error| anyhow::anyhow!("{error}"))?;
    let config = load_config(&repo_root).map_err(|error| anyhow::anyhow!("{error}"))?;
    let store = FileTicketStore::new(repo_root.clone());
    let ticket_id = parse_ticket_id_input(&id, &config.id_prefix)?;

    let mut ticket = store
        .load_ticket(&ticket_id)
        .map_err(|error| anyhow::anyhow!("{error}"))?;

    // Check if document with this name already exists
    if ticket.meta.documents.iter().any(|d| d.name == name) {
        // Already registered — return the existing path
        let existing = ticket
            .meta
            .documents
            .iter()
            .find(|d| d.name == name)
            .unwrap();
        let doc_path = ticket_dir(&repo_root, &ticket_id).join(&existing.path);
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

    // Derive the path from name: <name>.md at ticket root
    let rel_path = format!("{name}.md");
    let abs_path = ticket_dir(&repo_root, &ticket_id).join(&rel_path);

    // Write an empty template
    fs::write(&abs_path, format!("# {name}\n\n"))
        .map_err(|error| anyhow::anyhow!("failed to write {}: {error}", abs_path.display()))?;

    // Register the document
    ticket.meta.documents.push(TicketDocument {
        name: name.clone(),
        path: rel_path.clone(),
    });

    // Recompute fingerprints
    let mut fingerprints = std::collections::BTreeMap::new();
    for doc in &ticket.meta.documents {
        let doc_path = ticket_dir(&repo_root, &ticket_id).join(&doc.path);
        if doc_path.is_file() {
            let hash = fingerprint_file(&doc_path).map_err(|e| anyhow::anyhow!("{e}"))?;
            fingerprints.insert(doc.name.clone(), hash);
        }
    }
    ticket.state.document_fingerprints = fingerprints;

    // Bump revision and update timestamp
    ticket.state.revision += 1;
    ticket.state.updated_at = OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|error| anyhow::anyhow!("failed to format timestamp: {error}"))?;

    // Write canonical meta and state
    let meta_path = ticket_dir(&repo_root, &ticket_id).join("meta.toml");
    let state_path = ticket_dir(&repo_root, &ticket_id).join("state.toml");
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
