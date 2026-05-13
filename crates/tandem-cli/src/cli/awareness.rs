use std::env;

use tandem_core::awareness::{AwarenessChangeKind, compare_snapshots};
use tandem_repo::GitAwarenessProvider;
use tandem_storage::{discover_repo_root, load_ticket_snapshot};

use super::OutputArgs;

pub(crate) fn handle_awareness(against: String, output: OutputArgs) -> anyhow::Result<()> {
    let current_dir = env::current_dir().map_err(|error| anyhow::anyhow!("{error}"))?;
    let repo_root = discover_repo_root(&current_dir).map_err(|error| anyhow::anyhow!("{error}"))?;

    let current_snapshot =
        load_ticket_snapshot(&repo_root).map_err(|error| anyhow::anyhow!("{error}"))?;

    let provider = GitAwarenessProvider::new(repo_root);
    let against_snapshot = match provider
        .materialize_ref_snapshot(&against)
        .map_err(|error| anyhow::anyhow!("{error}"))?
    {
        None => tandem_core::awareness::TicketSnapshot::default(),
        Some(snapshot) => load_ticket_snapshot(snapshot.path()).map_err(|error| {
            anyhow::anyhow!(
                "failed to load materialized snapshot for ref `{}`: {}",
                against,
                snapshot.sanitize_error_text(&error.to_string())
            )
        })?,
    };

    let report = compare_snapshots(&against, &current_snapshot, &against_snapshot);

    if output.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print!("{}", format_awareness_text(&report));
    }
    Ok(())
}

pub(crate) fn format_awareness_text(report: &tandem_core::awareness::AwarenessReport) -> String {
    let mut output = format!("Against: {}\n\n", report.against);

    if report.tickets.is_empty() {
        output.push_str("No changes.\n");
        return output;
    }

    for ticket in &report.tickets {
        let kind = match &ticket.change {
            AwarenessChangeKind::AddedCurrent => "added (current)",
            AwarenessChangeKind::AddedAgainst => "added (against)",
            AwarenessChangeKind::Diverged => "diverged",
        };
        output.push_str(&format!("{}  {}\n", ticket.id, kind));

        if let Some(ref status) = ticket.fields.status {
            output.push_str(&format!(
                "  status:     {} -> {}\n",
                status.current, status.against
            ));
        }
        if let Some(ref priority) = ticket.fields.priority {
            output.push_str(&format!(
                "  priority:   {} -> {}\n",
                priority.current, priority.against
            ));
        }
        if let Some(ref effort) = ticket.fields.effort {
            output.push_str(&format!(
                "  effort:     {} -> {}\n",
                effort.current, effort.against
            ));
        }
        if let Some(ref title) = ticket.fields.title {
            output.push_str(&format!(
                "  title:      {} -> {}\n",
                title.current, title.against
            ));
        }
        if let Some(ref ticket_type) = ticket.fields.ticket_type {
            output.push_str(&format!(
                "  type:       {} -> {}\n",
                ticket_type.current, ticket_type.against
            ));
        }
        if let Some(ref depends_on) = ticket.fields.depends_on {
            output.push_str(&format!(
                "  depends_on: {:?} -> {:?}\n",
                depends_on.current, depends_on.against
            ));
        }
        if let Some(ref tags) = ticket.fields.tags {
            output.push_str(&format!(
                "  tags:       {:?} -> {:?}\n",
                tags.current, tags.against
            ));
        }
        if let Some(ref documents) = ticket.fields.documents {
            output.push_str("  documents:\n");
            for doc in documents {
                output.push_str(&format!(
                    "    {}: {} -> {}\n",
                    doc.name, doc.current, doc.against
                ));
            }
        }
    }

    output
}
