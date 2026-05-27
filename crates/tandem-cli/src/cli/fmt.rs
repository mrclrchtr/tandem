use std::{env, fs};

use tandem_core::ports::TicketStore;
use tandem_storage::{FileTicketStore, discover_repo_root, fingerprint_bytes, ticket_dir};

pub(crate) fn handle_fmt(check: bool) -> anyhow::Result<()> {
    let current_dir = env::current_dir().map_err(|error| anyhow::anyhow!("{error}"))?;
    let repo_root = discover_repo_root(&current_dir).map_err(|error| anyhow::anyhow!("{error}"))?;
    let store = FileTicketStore::new(repo_root.clone());
    let ids = store
        .list_ticket_ids()
        .map_err(|error| anyhow::anyhow!("{error}"))?;

    let mut changed_files = Vec::new();
    let mut drift_found = false;

    for id in ids {
        let ticket = store
            .load_ticket(&id)
            .map_err(|error| anyhow::anyhow!("{error}"))?;
        let base = ticket_dir(&repo_root, &id);
        let expected_files = [
            (base.join("meta.toml"), ticket.meta.to_canonical_toml()),
            (base.join("state.toml"), ticket.state.to_canonical_toml()),
        ];

        for (path, canonical) in expected_files {
            let current = fs::read_to_string(&path).map_err(|error| anyhow::anyhow!("{error}"))?;
            if current != canonical {
                changed_files.push(path.clone());
                if !check {
                    fs::write(&path, canonical).map_err(|error| anyhow::anyhow!("{error}"))?;
                }
            }
        }

        // Normalize registered documents to end with a single trailing newline
        let mut fingerprints_changed = false;
        let mut state_fingerprints = ticket.state.document_fingerprints.clone();
        for doc in &ticket.meta.documents {
            let doc_path = base.join(&doc.path);
            if !doc_path.is_file() {
                continue;
            }
            let current =
                fs::read_to_string(&doc_path).map_err(|error| anyhow::anyhow!("{error}"))?;
            let normalized = ensure_trailing_newline(&current);
            if current != normalized {
                changed_files.push(doc_path.clone());
                if !check {
                    fs::write(&doc_path, &normalized)
                        .map_err(|error| anyhow::anyhow!("{error}"))?;
                }
                // Recompute fingerprint in both modes so state.toml
                // is reported in --check too (even though we won't write)
                let new_fp = fingerprint_bytes(normalized.as_bytes());
                if state_fingerprints
                    .get(&doc.name)
                    .is_none_or(|fp| fp != &new_fp)
                {
                    state_fingerprints.insert(doc.name.clone(), new_fp);
                    fingerprints_changed = true;
                }
            }
        }

        // If fingerprints changed due to document normalization, update state.toml
        if fingerprints_changed {
            let state_path = base.join("state.toml");
            let mut updated_state = ticket.state.clone();
            updated_state.document_fingerprints = state_fingerprints;
            let canonical_state = updated_state.to_canonical_toml();
            if !check {
                fs::write(&state_path, &canonical_state)
                    .map_err(|error| anyhow::anyhow!("{error}"))?;
            }
            if !changed_files.contains(&state_path) {
                changed_files.push(state_path);
            }
        }

        // Check document fingerprint drift
        let drift = store
            .document_drift(&id)
            .map_err(|error| anyhow::anyhow!("{error}"))?;
        if !drift.is_empty() {
            drift_found = true;
            for (name, _) in &drift {
                println!(
                    "stale fingerprint for document `{name}` in ticket {} (run `tndm ticket sync {id}` to refresh)",
                    id.as_str()
                );
            }
        }
    }

    if check && (!changed_files.is_empty() || drift_found) {
        for path in &changed_files {
            println!("{}", path.display());
        }
        let reasons = if !changed_files.is_empty() && drift_found {
            "non-canonical tandem files and stale fingerprints"
        } else if !changed_files.is_empty() {
            "non-canonical tandem files"
        } else {
            "stale document fingerprints"
        };
        anyhow::bail!("tndm fmt --check found {reasons}");
    }

    for path in &changed_files {
        println!("{}", path.display());
    }

    Ok(())
}

/// Ensure content ends with exactly one trailing newline.
///
/// - Empty content => `"\n"`
/// - Already ends with single `\n` => unchanged
/// - Multiple trailing `\n` => collapsed to one
/// - Windows `\r\n` terminators are preserved (only `\n` is trimmed,
///   so `"hello\r\n"` stays as-is).
fn ensure_trailing_newline(content: &str) -> String {
    if content.is_empty() {
        return "\n".to_string();
    }
    let trimmed = content.trim_end_matches('\n');
    format!("{}\n", trimmed)
}
