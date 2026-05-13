use std::{env, fs};

use tandem_core::ports::TicketStore;
use tandem_storage::{FileTicketStore, discover_repo_root, ticket_dir};

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
