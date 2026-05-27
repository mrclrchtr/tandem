//! Handler for `tndm ticket create`.

use std::path::PathBuf;

use clap::Args;
use tandem_core::{
    ports::TicketStore,
    ticket::{
        NewTicket, TicketEffort, TicketId, TicketMeta, TicketPriority, TicketStatus, TicketType,
    },
};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

use crate::cli::render::output_ticket_json;
use crate::cli::ticket::update::TicketUpdate;
use crate::cli::ticket_ctx::TicketCtx;
use crate::cli::util::{generate_ticket_id, load_ticket_content, read_stdin_if_no_flags};

#[derive(Args, Debug)]
pub(crate) struct TicketCreateArgs {
    /// Ticket title.
    pub(crate) title: String,

    /// Optional explicit ticket ID.
    #[arg(long)]
    pub(crate) id: Option<String>,

    /// Optional content markdown file path.
    #[arg(long, conflicts_with = "content")]
    pub(crate) content_file: Option<PathBuf>,

    /// Optional inline content body.
    #[arg(long, conflicts_with = "content_file")]
    pub(crate) content: Option<String>,

    /// Initial status [possible values: todo, in_progress, blocked, done].
    #[arg(long, short)]
    pub(crate) status: Option<TicketStatus>,

    /// Initial priority [possible values: p0, p1, p2, p3, p4].
    #[arg(long, short)]
    pub(crate) priority: Option<TicketPriority>,

    /// Initial ticket type [possible values: task, bug, feature, chore, epic].
    #[arg(long = "type", short = 'T')]
    pub(crate) ticket_type: Option<TicketType>,

    /// Comma-separated tags.
    #[arg(long, short = 'g')]
    pub(crate) tags: Option<String>,

    /// Comma-separated ticket IDs for dependencies.
    #[arg(long, short = 'd')]
    pub(crate) depends_on: Option<String>,

    /// Effort estimate [possible values: xs, s, m, l, xl].
    #[arg(long, short = 'e')]
    pub(crate) effort: Option<TicketEffort>,

    #[command(flatten)]
    pub(crate) output: crate::cli::OutputArgs,
}

pub(crate) fn handle_ticket_create(args: TicketCreateArgs) -> anyhow::Result<()> {
    let ctx = TicketCtx::new()?;

    let update = TicketUpdate::from_create_args(&args);
    let stdin_content = read_stdin_if_no_flags(update.is_empty())?;

    let ticket_id = match args.id {
        Some(ref value) => TicketId::parse(value)?,
        None => generate_ticket_id(&ctx.store, ctx.id_prefix())?,
    };

    let content = load_ticket_content(args.content_file, args.content, stdin_content, &ctx.config)?;
    let meta = TicketMeta::new(ticket_id, args.title)?;

    let mut ticket = ctx
        .store
        .create_ticket(NewTicket { meta, content })
        .map_err(|error| anyhow::anyhow!("{error}"))?;

    // Apply metadata fields (priority, type, tags, etc.) after creation.
    // Only persist when metadata fields or status were explicitly requested.
    // Content-only creates don't need an extra write since NewTicket already has it.
    let has_meta_changes = update.status.is_some()
        || update.priority.is_some()
        || update.ticket_type.is_some()
        || update.tags.is_some()
        || update.depends_on.is_some()
        || update.effort.is_some();
    let needs_persist = has_meta_changes || args.status.is_some();

    if has_meta_changes {
        update.apply(&mut ticket, ctx.id_prefix())?;
    }
    if let Some(value) = args.status {
        ticket.state.status = value;
    }

    if needs_persist {
        ticket.state.revision += 1;
        ticket.state.updated_at = OffsetDateTime::now_utc()
            .format(&Rfc3339)
            .map_err(|error| anyhow::anyhow!("failed to format timestamp: {error}"))?;
        ticket = ctx
            .store
            .update_ticket(&ticket)
            .map_err(|error| anyhow::anyhow!("{error}"))?;
    }

    if args.output.json {
        output_ticket_json(&ticket)?;
    } else {
        println!("{}", ticket.meta.id);
    }
    Ok(())
}
