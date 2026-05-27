//! Handler for `tndm ticket list`.

use clap::Args;
use tabled::{builder::Builder, settings::Style};
use tandem_core::{
    ports::TicketStore,
    ticket::{Ticket, TicketStatus},
};

use crate::cli::render::{TicketJsonEntry, TicketListJson};
use crate::cli::ticket::TicketDefinitionFilter;
use crate::cli::ticket_ctx::TicketCtx;
use crate::cli::util::{DEFINITION_TAG_QUESTIONS, DEFINITION_TAG_READY, ticket_content_path};

#[derive(Args, Debug)]
pub(crate) struct TicketListArgs {
    /// Include tickets with status "done".
    #[arg(long)]
    pub(crate) all: bool,

    /// Filter by current definition state backed by reserved tags:
    /// definition:ready, definition:questions, or no definition:* tag.
    #[arg(long = "definition", value_enum)]
    pub(crate) definition: Option<TicketDefinitionFilter>,

    #[command(flatten)]
    pub(crate) output: crate::cli::OutputArgs,
}

pub(crate) fn handle_ticket_list(args: TicketListArgs) -> anyhow::Result<()> {
    let ctx = TicketCtx::new()?;
    let ids = ctx
        .store
        .list_ticket_ids()
        .map_err(|error| anyhow::anyhow!("{error}"))?;

    let mut tickets = Vec::new();
    for id in ids {
        let ticket = ctx
            .store
            .load_ticket(&id)
            .map_err(|error| anyhow::anyhow!("{error}"))?;
        let matches_status = args.all || ticket.state.status != TicketStatus::Done;
        let matches_definition = args
            .definition
            .map(|value| ticket_matches_definition_filter(&ticket, value))
            .unwrap_or(true);
        if matches_status && matches_definition {
            tickets.push(ticket);
        }
    }

    tickets.sort_by(|a, b| {
        a.meta
            .priority
            .cmp(&b.meta.priority)
            .then_with(|| a.meta.id.cmp(&b.meta.id))
    });

    if args.output.json {
        let envelope = TicketListJson {
            schema_version: 1,
            tickets: tickets
                .iter()
                .map(|t| TicketJsonEntry {
                    meta: &t.meta,
                    state: &t.state,
                    content_path: ticket_content_path(&t.meta.id),
                })
                .collect(),
        };
        println!("{}", serde_json::to_string_pretty(&envelope)?);
    } else {
        let mut builder = Builder::new();
        builder.push_record(["ID", "STATUS", "PRIO", "EFFORT", "DEPS", "TITLE"]);
        for ticket in &tickets {
            let deps = ticket
                .meta
                .depends_on
                .iter()
                .map(|id| id.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            builder.push_record([
                ticket.meta.id.as_str(),
                ticket.state.status.as_str(),
                ticket.meta.priority.as_str(),
                ticket
                    .meta
                    .effort
                    .as_ref()
                    .map(|e| e.as_str())
                    .unwrap_or("-"),
                &deps,
                &ticket.meta.title,
            ]);
        }
        println!("{}", builder.build().with(Style::blank()));
    }

    Ok(())
}

pub(crate) fn ticket_matches_definition_filter(
    ticket: &Ticket,
    filter: TicketDefinitionFilter,
) -> bool {
    let has_ready = ticket
        .meta
        .tags
        .iter()
        .any(|tag| tag == DEFINITION_TAG_READY);
    let has_questions = ticket
        .meta
        .tags
        .iter()
        .any(|tag| tag == DEFINITION_TAG_QUESTIONS);

    match filter {
        TicketDefinitionFilter::Ready => has_ready,
        TicketDefinitionFilter::Questions => has_questions,
        TicketDefinitionFilter::Unknown => !has_ready && !has_questions,
    }
}
