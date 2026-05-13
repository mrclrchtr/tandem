use std::{env, fs, path::PathBuf};

use clap::{Subcommand, ValueEnum};
use tabled::{builder::Builder, settings::Style};
use tandem_core::{
    ports::TicketStore,
    ticket::{
        NewTicket, TicketEffort, TicketId, TicketMeta, TicketPriority, TicketStatus, TicketType,
    },
};
use tandem_storage::{FileTicketStore, discover_repo_root, load_config};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

use super::OutputArgs;
use super::render::{TicketJson, TicketJsonEntry};
use super::util::{
    DEFINITION_TAG_QUESTIONS, DEFINITION_TAG_READY, generate_ticket_id, load_ticket_content,
    read_stdin_if_no_flags, ticket_content_path,
};

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub(crate) enum TicketDefinitionFilter {
    Ready,
    Questions,
    Unknown,
}

#[derive(Subcommand, Debug)]
pub(crate) enum TicketCommand {
    /// Create a new ticket.
    Create {
        /// Ticket title.
        title: String,

        /// Optional explicit ticket ID.
        #[arg(long)]
        id: Option<String>,

        /// Optional content markdown file path.
        #[arg(long, conflicts_with = "content")]
        content_file: Option<PathBuf>,

        /// Optional inline content body.
        #[arg(long, conflicts_with = "content_file")]
        content: Option<String>,

        /// Initial status [possible values: todo, in_progress, blocked, done].
        #[arg(long, short)]
        status: Option<TicketStatus>,

        /// Initial priority [possible values: p0, p1, p2, p3, p4].
        #[arg(long, short)]
        priority: Option<TicketPriority>,

        /// Initial ticket type [possible values: task, bug, feature, chore, epic].
        #[arg(long = "type", short = 'T')]
        ticket_type: Option<TicketType>,

        /// Comma-separated tags.
        #[arg(long, short = 'g')]
        tags: Option<String>,

        /// Comma-separated ticket IDs for dependencies.
        #[arg(long, short = 'd')]
        depends_on: Option<String>,

        /// Effort estimate [possible values: xs, s, m, l, xl].
        #[arg(long, short = 'e')]
        effort: Option<TicketEffort>,

        #[command(flatten)]
        output: OutputArgs,
    },
    Show {
        id: String,

        #[command(flatten)]
        output: OutputArgs,
    },
    List {
        /// Include tickets with status "done".
        #[arg(long)]
        all: bool,

        /// Filter by current definition state backed by reserved tags:
        /// definition:ready, definition:questions, or no definition:* tag.
        #[arg(long = "definition", value_enum)]
        definition: Option<TicketDefinitionFilter>,

        #[command(flatten)]
        output: OutputArgs,
    },
    /// Update an existing ticket.
    #[command(arg_required_else_help = true)]
    Update {
        /// Ticket ID to update.
        id: String,

        /// New status [possible values: todo, in_progress, blocked, done].
        #[arg(long, short)]
        status: Option<TicketStatus>,

        /// New priority [possible values: p0, p1, p2, p3, p4].
        #[arg(long, short)]
        priority: Option<TicketPriority>,

        /// New title.
        #[arg(long, short)]
        title: Option<String>,

        /// New ticket type [possible values: task, bug, feature, chore, epic].
        #[arg(long = "type", short = 'T')]
        ticket_type: Option<TicketType>,

        /// Comma-separated tags (replaces existing list, empty string clears).
        #[arg(long, short = 'g')]
        tags: Option<String>,

        /// Tags to add (comma-separated, preserves existing tags).
        #[arg(long, conflicts_with = "tags")]
        add_tags: Option<String>,

        /// Tags to remove (comma-separated).
        #[arg(long, conflicts_with = "tags")]
        remove_tags: Option<String>,

        /// Comma-separated ticket IDs for dependencies (replaces existing list, empty string clears).
        #[arg(long, short = 'd')]
        depends_on: Option<String>,

        /// Effort estimate [possible values: xs, s, m, l, xl].
        #[arg(long, short = 'e')]
        effort: Option<TicketEffort>,

        /// Markdown file replacing content.
        #[arg(long, conflicts_with = "update_content")]
        content_file: Option<PathBuf>,

        /// Inline content body replacing existing content.
        #[arg(
            long = "content",
            id = "update_content",
            conflicts_with = "content_file"
        )]
        content: Option<String>,

        #[command(flatten)]
        output: OutputArgs,
    },
    /// Manage registered ticket documents.
    Doc {
        #[command(subcommand)]
        command: DocCommand,
    },
    /// Synchronize document fingerprints after file edits.
    Sync {
        /// Ticket ID to synchronize.
        id: String,

        #[command(flatten)]
        output: OutputArgs,
    },
}

#[derive(Subcommand, Debug)]
pub(crate) enum DocCommand {
    /// Create and register a new document file for a ticket.
    Create {
        /// Ticket ID.
        id: String,

        /// Document name (e.g. plan, archive).
        name: String,

        #[command(flatten)]
        output: OutputArgs,
    },
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn handle_ticket_create(
    title: String,
    id: Option<String>,
    content_file: Option<PathBuf>,
    content: Option<String>,
    status: Option<TicketStatus>,
    priority: Option<TicketPriority>,
    ticket_type: Option<TicketType>,
    tags: Option<String>,
    depends_on: Option<String>,
    effort: Option<TicketEffort>,
    json: bool,
) -> anyhow::Result<()> {
    let current_dir = env::current_dir().map_err(|error| anyhow::anyhow!("{error}"))?;
    let repo_root = discover_repo_root(&current_dir).map_err(|error| anyhow::anyhow!("{error}"))?;
    let store = FileTicketStore::new(repo_root.clone());
    let config = load_config(&repo_root).map_err(|error| anyhow::anyhow!("{error}"))?;

    // Only read stdin when no explicit create flags are provided. Speculatively reading
    // stdin when flags like --status or --tags are present causes an infinite hang in
    // non-TTY environments (e.g. Node.js execFile) where the write end of stdin stays open.
    let no_explicit_create = content_file.is_none()
        && content.is_none()
        && id.is_none()
        && status.is_none()
        && priority.is_none()
        && ticket_type.is_none()
        && tags.is_none()
        && depends_on.is_none()
        && effort.is_none();
    let stdin_content = read_stdin_if_no_flags(no_explicit_create)?;

    let ticket_id = match id {
        Some(value) => TicketId::parse(value)?,
        None => generate_ticket_id(&store, &config.id_prefix)?,
    };

    let content = load_ticket_content(content_file, content, stdin_content, &config)?;
    let mut meta = TicketMeta::new(ticket_id, title)?;

    if let Some(value) = priority {
        meta.priority = value;
    }
    if let Some(value) = ticket_type {
        meta.ticket_type = value;
    }
    if let Some(value) = tags {
        let mut parsed: Vec<String> = if value.trim().is_empty() {
            Vec::new()
        } else {
            value.split(',').map(|s| s.trim().to_string()).collect()
        };
        parsed.sort();
        parsed.dedup();
        meta.tags = parsed;
    }
    if let Some(value) = depends_on {
        let mut parsed: Vec<TicketId> = if value.trim().is_empty() {
            Vec::new()
        } else {
            value
                .split(',')
                .map(|s| TicketId::parse(s.trim()))
                .collect::<Result<Vec<_>, _>>()?
        };
        parsed.sort();
        parsed.dedup();
        meta.depends_on = parsed;
    }
    if let Some(value) = effort {
        meta.effort = Some(value);
    }

    let mut ticket = store
        .create_ticket(NewTicket { meta, content })
        .map_err(|error| anyhow::anyhow!("{error}"))?;

    if let Some(value) = status {
        ticket.state.status = value;
        ticket = store
            .update_ticket(&ticket)
            .map_err(|error| anyhow::anyhow!("{error}"))?;
    }

    if json {
        let envelope = TicketJson {
            schema_version: 1,
            ticket: TicketJsonEntry {
                meta: &ticket.meta,
                state: &ticket.state,
                content_path: ticket_content_path(&ticket.meta.id),
            },
        };
        println!("{}", serde_json::to_string_pretty(&envelope)?);
    } else {
        println!("{}", ticket.meta.id);
    }
    Ok(())
}

pub(crate) fn handle_ticket_show(id: String, json: bool) -> anyhow::Result<()> {
    let current_dir = env::current_dir().map_err(|error| anyhow::anyhow!("{error}"))?;
    let repo_root = discover_repo_root(&current_dir).map_err(|error| anyhow::anyhow!("{error}"))?;
    let store = FileTicketStore::new(repo_root);
    let id = TicketId::parse(id)?;
    let ticket = store
        .load_ticket(&id)
        .map_err(|error| anyhow::anyhow!("{error}"))?;

    if json {
        let envelope = TicketJson {
            schema_version: 1,
            ticket: TicketJsonEntry {
                meta: &ticket.meta,
                state: &ticket.state,
                content_path: ticket_content_path(&ticket.meta.id),
            },
        };
        println!("{}", serde_json::to_string_pretty(&envelope)?);
    } else {
        super::render::print_ticket_human(&ticket);
    }
    Ok(())
}

pub(crate) fn handle_ticket_list(
    json: bool,
    all: bool,
    definition: Option<TicketDefinitionFilter>,
) -> anyhow::Result<()> {
    let current_dir = env::current_dir().map_err(|error| anyhow::anyhow!("{error}"))?;
    let repo_root = discover_repo_root(&current_dir).map_err(|error| anyhow::anyhow!("{error}"))?;
    let store = FileTicketStore::new(repo_root);
    let ids = store
        .list_ticket_ids()
        .map_err(|error| anyhow::anyhow!("{error}"))?;

    let mut tickets = Vec::new();
    for id in ids {
        let ticket = store
            .load_ticket(&id)
            .map_err(|error| anyhow::anyhow!("{error}"))?;
        let matches_status = all || ticket.state.status != TicketStatus::Done;
        let matches_definition = definition
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

    if json {
        let envelope = super::render::TicketListJson {
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
    ticket: &tandem_core::ticket::Ticket,
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

#[allow(clippy::too_many_arguments)]
pub(crate) fn handle_ticket_update(
    id: String,
    status: Option<TicketStatus>,
    priority: Option<TicketPriority>,
    title: Option<String>,
    ticket_type: Option<TicketType>,
    tags: Option<String>,
    add_tags: Option<String>,
    remove_tags: Option<String>,
    depends_on: Option<String>,
    effort: Option<TicketEffort>,
    content_file: Option<PathBuf>,
    content: Option<String>,
    json: bool,
) -> anyhow::Result<()> {
    // Only read stdin when no explicit update flags are provided. Speculatively reading stdin
    // when metadata flags like --status are present causes an infinite hang in non-TTY
    // environments (e.g. scripted pipelines) where the write end of stdin stays open.
    let no_explicit_update = content_file.is_none()
        && content.is_none()
        && status.is_none()
        && priority.is_none()
        && title.is_none()
        && ticket_type.is_none()
        && tags.is_none()
        && add_tags.is_none()
        && remove_tags.is_none()
        && depends_on.is_none()
        && effort.is_none();
    let stdin_content = read_stdin_if_no_flags(no_explicit_update)?;

    if status.is_none()
        && priority.is_none()
        && title.is_none()
        && ticket_type.is_none()
        && tags.is_none()
        && add_tags.is_none()
        && remove_tags.is_none()
        && depends_on.is_none()
        && effort.is_none()
        && content_file.is_none()
        && content.is_none()
        && stdin_content.is_none()
    {
        anyhow::bail!(
            "at least one update flag is required\n\n  \
             Example: tndm ticket update {id} --status done\n\n  \
             Run 'tndm ticket update --help' for all options"
        );
    }

    let current_dir = env::current_dir().map_err(|error| anyhow::anyhow!("{error}"))?;
    let repo_root = discover_repo_root(&current_dir).map_err(|error| anyhow::anyhow!("{error}"))?;
    let store = FileTicketStore::new(repo_root);
    let ticket_id = TicketId::parse(id)?;
    let mut ticket = store
        .load_ticket(&ticket_id)
        .map_err(|error| anyhow::anyhow!("{error}"))?;

    if let Some(value) = status {
        ticket.state.status = value;
    }
    if let Some(value) = priority {
        ticket.meta.priority = value;
    }
    if let Some(value) = title {
        if value.trim().is_empty() {
            anyhow::bail!("title must not be empty");
        }
        ticket.meta.title = value;
    }
    if let Some(value) = ticket_type {
        ticket.meta.ticket_type = value;
    }
    if let Some(value) = tags {
        let mut parsed: Vec<String> = if value.trim().is_empty() {
            Vec::new()
        } else {
            value.split(',').map(|s| s.trim().to_string()).collect()
        };
        parsed.sort();
        parsed.dedup();
        ticket.meta.tags = parsed;
    }
    if let Some(value) = depends_on {
        let mut parsed: Vec<TicketId> = if value.trim().is_empty() {
            Vec::new()
        } else {
            value
                .split(',')
                .map(|s| TicketId::parse(s.trim()))
                .collect::<Result<Vec<_>, _>>()?
        };
        parsed.sort();
        parsed.dedup();
        ticket.meta.depends_on = parsed;
    }
    if let Some(value) = effort {
        ticket.meta.effort = Some(value);
    }
    if let Some(value) = add_tags {
        for tag in value.split(',') {
            let trimmed = tag.trim().to_string();
            if !trimmed.is_empty() && !ticket.meta.tags.contains(&trimmed) {
                ticket.meta.tags.push(trimmed);
            }
        }
        ticket.meta.tags.sort();
    }
    if let Some(value) = remove_tags {
        let to_remove: Vec<String> = value.split(',').map(|s| s.trim().to_string()).collect();
        ticket.meta.tags.retain(|t| !to_remove.contains(t));
    }
    if let Some(path) = content_file {
        ticket.content = fs::read_to_string(&path)
            .map_err(|error| anyhow::anyhow!("failed to read {}: {error}", path.display()))?;
    } else if let Some(value) = content {
        ticket.content = value;
    } else if let Some(value) = stdin_content {
        ticket.content = value;
    }

    ticket.state.revision += 1;
    ticket.state.updated_at = OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|error| anyhow::anyhow!("failed to format timestamp: {error}"))?;

    let updated = store
        .update_ticket(&ticket)
        .map_err(|error| anyhow::anyhow!("{error}"))?;

    if json {
        let envelope = TicketJson {
            schema_version: 1,
            ticket: TicketJsonEntry {
                meta: &updated.meta,
                state: &updated.state,
                content_path: ticket_content_path(&updated.meta.id),
            },
        };
        println!("{}", serde_json::to_string_pretty(&envelope)?);
    } else {
        println!("{ticket_id}");
    }
    Ok(())
}

pub(crate) fn handle_ticket_sync(id: String, json: bool) -> anyhow::Result<()> {
    let current_dir = env::current_dir().map_err(|error| anyhow::anyhow!("{error}"))?;
    let repo_root = discover_repo_root(&current_dir).map_err(|error| anyhow::anyhow!("{error}"))?;
    let store = FileTicketStore::new(repo_root);
    let ticket_id = TicketId::parse(id)?;

    let updated = store
        .sync_ticket_documents(&ticket_id)
        .map_err(|error| anyhow::anyhow!("{error}"))?;

    if json {
        let envelope = TicketJson {
            schema_version: 1,
            ticket: TicketJsonEntry {
                meta: &updated.meta,
                state: &updated.state,
                content_path: ticket_content_path(&updated.meta.id),
            },
        };
        println!("{}", serde_json::to_string_pretty(&envelope)?);
    } else {
        println!("{ticket_id}");
    }
    Ok(())
}
