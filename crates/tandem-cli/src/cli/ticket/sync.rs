//! Handler for `tndm ticket sync`.

use tandem_core::ports::TicketStore;

use crate::cli::render::output_ticket_json;
use crate::cli::ticket_ctx::TicketCtx;

pub(crate) fn handle_ticket_sync(id: Option<String>, all: bool, json: bool) -> anyhow::Result<()> {
    // Validate arguments before touching the filesystem.
    match (&id, all) {
        (Some(_), true) => {
            anyhow::bail!("cannot provide both a ticket ID and --all");
        }
        (None, false) => {
            anyhow::bail!("provide a ticket ID or use --all");
        }
        _ => {}
    }

    let ctx = TicketCtx::new()?;

    match (id, all) {
        (Some(input), false) => {
            // Single-ticket sync
            let ticket_id = ctx.resolve_id(&input)?;
            let updated = ctx
                .store
                .sync_ticket_documents(&ticket_id)
                .map_err(|error| anyhow::anyhow!("{error}"))?;

            if json {
                output_ticket_json(&updated)?;
            } else {
                println!("{ticket_id}");
            }
            Ok(())
        }
        (None, true) => {
            // Batch sync all tickets
            let ids = ctx
                .store
                .list_ticket_ids()
                .map_err(|error| anyhow::anyhow!("{error}"))?;

            let total = ids.len();
            let mut results: Vec<(String, Option<String>)> = Vec::with_capacity(total);
            let mut ok_count = 0u32;
            let mut fail_count = 0u32;

            for ticket_id in &ids {
                match ctx.store.sync_ticket_documents(ticket_id) {
                    Ok(_) => {
                        ok_count += 1;
                        results.push((ticket_id.to_string(), None));
                        if !json {
                            println!("{ticket_id}");
                        }
                    }
                    Err(error) => {
                        fail_count += 1;
                        let msg = format!("{error}");
                        results.push((ticket_id.to_string(), Some(msg.clone())));
                        if !json {
                            eprintln!("{ticket_id}: {msg}");
                        }
                    }
                }
            }

            if json {
                let entries: Vec<serde_json::Value> = results
                    .into_iter()
                    .map(|(tid, err)| {
                        serde_json::json!({
                            "id": tid,
                            "status": if err.is_some() { "error" } else { "ok" },
                            "message": err,
                        })
                    })
                    .collect();
                let summary = serde_json::json!({
                    "schema_version": 1,
                    "synced": ok_count,
                    "failed": fail_count,
                    "total": total,
                    "results": entries,
                });
                println!("{}", serde_json::to_string_pretty(&summary)?);
            } else {
                eprintln!(
                    "Synced {ok_count}/{total} ticket(s){}",
                    if fail_count > 0 {
                        format!(" ({fail_count} failed)")
                    } else {
                        String::new()
                    }
                );
            }

            if fail_count > 0 {
                anyhow::bail!("{fail_count} ticket(s) failed to sync");
            }
            Ok(())
        }
        // Already validated above.
        _ => unreachable!(),
    }
}
