//! Handler for `tndm ticket show`.

use crate::cli::render::output_ticket_json;
use crate::cli::ticket_ctx::TicketCtx;
use tandem_core::ports::TicketStore;

pub(crate) fn handle_ticket_show(id: String, json: bool) -> anyhow::Result<()> {
    let ctx = TicketCtx::new()?;
    let id = ctx.resolve_id(&id)?;
    let ticket = ctx
        .store
        .load_ticket(&id)
        .map_err(|error| anyhow::anyhow!("{error}"))?;

    if json {
        output_ticket_json(&ticket)?;
    } else {
        crate::cli::render::print_ticket_human(&ticket);
    }
    Ok(())
}
