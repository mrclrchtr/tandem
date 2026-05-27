//! Handler for `tndm ticket sync`.

use crate::cli::render::output_ticket_json;
use crate::cli::ticket_ctx::TicketCtx;

pub(crate) fn handle_ticket_sync(id: String, json: bool) -> anyhow::Result<()> {
    let ctx = TicketCtx::new()?;
    let ticket_id = ctx.resolve_id(&id)?;

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
