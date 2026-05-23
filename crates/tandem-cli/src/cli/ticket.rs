use std::{
    env, fs,
    path::{Path, PathBuf},
};

use clap::{Subcommand, ValueEnum};
use tabled::{builder::Builder, settings::Style};
use tandem_core::{
    ports::TicketStore,
    ticket::{
        NewTicket, Task, TaskStatus, Ticket, TicketEffort, TicketId, TicketMeta, TicketPriority,
        TicketStatus, TicketType,
    },
};
use tandem_storage::{FileTicketStore, discover_repo_root, load_config, ticket_dir};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

use super::OutputArgs;
use super::doc::recompute_ticket_document_fingerprints;
use super::render::{TicketJson, TicketJsonEntry};
use super::util::{
    DEFINITION_TAG_QUESTIONS, DEFINITION_TAG_READY, generate_ticket_id, load_ticket_content,
    parse_ticket_id_input, read_stdin_if_no_flags, ticket_content_path,
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
    /// Manage ticket tasks.
    Task {
        #[command(subcommand)]
        command: TaskCommand,
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

        /// Ticket-relative document path (optional; defaults to <name>.md).
        #[arg(long)]
        path: Option<String>,

        #[command(flatten)]
        output: OutputArgs,
    },
}

#[derive(Subcommand, Debug)]
pub(crate) enum TaskCommand {
    /// Add a task to a ticket.
    Add {
        /// Ticket ID.
        id: String,

        /// Task title.
        #[arg(long, short = 't')]
        title: String,

        /// File paths (optional; repeat --file for multiple entries).
        #[arg(long, short = 'f')]
        file: Vec<String>,

        /// Verification command (optional).
        #[arg(long, short = 'v')]
        verification: Option<String>,

        /// Extra notes (optional).
        #[arg(long, short = 'n')]
        notes: Option<String>,

        #[command(flatten)]
        output: OutputArgs,
    },
    /// List tasks for a ticket.
    List {
        /// Ticket ID.
        id: String,

        #[command(flatten)]
        output: OutputArgs,
    },
    /// Mark a task as done.
    Complete {
        /// Ticket ID.
        id: String,

        /// Task number to complete.
        number: u32,

        #[command(flatten)]
        output: OutputArgs,
    },
    /// Remove a task from a ticket.
    Remove {
        /// Ticket ID.
        id: String,

        /// Task number to remove.
        number: u32,

        #[command(flatten)]
        output: OutputArgs,
    },
    /// Edit a task's fields.
    Edit {
        /// Ticket ID.
        id: String,

        /// Task number to edit.
        number: u32,

        /// New title (optional).
        #[arg(long, short = 't')]
        title: Option<String>,

        /// New file paths (optional; repeat --file for multiple entries).
        #[arg(long, short = 'f')]
        file: Option<Vec<String>>,

        /// Clear all file paths.
        #[arg(long, conflicts_with = "file")]
        clear_files: bool,

        /// New verification command (optional).
        #[arg(long, short = 'v')]
        verification: Option<String>,

        /// New notes (optional).
        #[arg(long, short = 'n')]
        notes: Option<String>,

        #[command(flatten)]
        output: OutputArgs,
    },
    /// Ensure or clear a task's linked detail document.
    Detail {
        #[command(subcommand)]
        command: TaskDetailCommand,
    },
    /// Bulk-replace all tasks from a JSON string.
    Set {
        /// Ticket ID.
        id: String,

        /// JSON array of task objects.
        #[arg(long)]
        tasks: String,

        #[command(flatten)]
        output: OutputArgs,
    },
}

#[derive(Subcommand, Debug)]
pub(crate) enum TaskDetailCommand {
    /// Ensure the canonical task detail document exists and is linked.
    Ensure {
        /// Ticket ID.
        id: String,

        /// Task number.
        number: u32,

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
                .map(|s| parse_ticket_id_input(s, &config.id_prefix))
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
    let config = load_config(&repo_root).map_err(|error| anyhow::anyhow!("{error}"))?;
    let store = FileTicketStore::new(repo_root);
    let id = parse_ticket_id_input(&id, &config.id_prefix)?;
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
    let config = load_config(&repo_root).map_err(|error| anyhow::anyhow!("{error}"))?;
    let store = FileTicketStore::new(repo_root);
    let ticket_id = parse_ticket_id_input(&id, &config.id_prefix)?;
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
                .map(|s| parse_ticket_id_input(s, &config.id_prefix))
                .collect::<Result<Vec<_>, _>>()?
        };
        parsed.sort();
        parsed.dedup();
        ticket.meta.depends_on = parsed;
    }
    if let Some(value) = effort {
        ticket.meta.effort = Some(value);
    }
    if let Some(value) = remove_tags {
        let to_remove: Vec<String> = value.split(',').map(|s| s.trim().to_string()).collect();
        ticket.meta.tags.retain(|t| !to_remove.contains(t));
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
    let config = load_config(&repo_root).map_err(|error| anyhow::anyhow!("{error}"))?;
    let store = FileTicketStore::new(repo_root);
    let ticket_id = parse_ticket_id_input(&id, &config.id_prefix)?;

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

// ─── Task handlers ───────────────────────────────────────────

fn load_and_bump(store: &FileTicketStore, ticket_id: &TicketId) -> anyhow::Result<Ticket> {
    let mut ticket = store
        .load_ticket(ticket_id)
        .map_err(|error| anyhow::anyhow!("{error}"))?;
    ticket.state.revision += 1;
    ticket.state.updated_at = OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|error| anyhow::anyhow!("failed to format timestamp: {error}"))?;
    Ok(ticket)
}

fn persist_and_output(store: &FileTicketStore, ticket: &Ticket, json: bool) -> anyhow::Result<()> {
    let _ = store
        .update_ticket(ticket)
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
        println!("{}", ticket.meta.id);
    }
    Ok(())
}

fn find_task(tasks: &[Task], number: u32) -> Result<(usize, &Task), anyhow::Error> {
    tasks
        .iter()
        .enumerate()
        .find(|(_, t)| t.number == number)
        .ok_or_else(|| anyhow::anyhow!("task {number} not found"))
}

fn normalize_task_files(files: Vec<String>) -> Vec<String> {
    files
        .into_iter()
        .filter_map(|file| {
            let trimmed = file.trim();
            (!trimmed.is_empty()).then(|| trimmed.to_string())
        })
        .collect()
}

fn canonical_task_detail_doc(number: u32) -> (String, String) {
    let name = format!("task-{:02}", number);
    let path = format!("tasks/{name}.md");
    (name, path)
}

fn is_canonical_task_detail_doc(doc: &tandem_core::ticket::TicketDocument) -> bool {
    let Some(number) = doc
        .name
        .strip_prefix("task-")
        .and_then(|value| value.parse::<u32>().ok())
    else {
        return false;
    };
    let (expected_name, expected_path) = canonical_task_detail_doc(number);
    doc.name == expected_name && doc.path == expected_path
}

fn prune_unlinked_canonical_task_detail_docs(
    repo_root: &Path,
    ticket_id: &TicketId,
    ticket: &mut Ticket,
) -> anyhow::Result<()> {
    let linked_detail_paths = ticket
        .state
        .tasks
        .iter()
        .filter_map(|task| task.detail_path.clone())
        .collect::<std::collections::BTreeSet<_>>();

    let original_len = ticket.meta.documents.len();
    ticket.meta.documents.retain(|doc| {
        !is_canonical_task_detail_doc(doc) || linked_detail_paths.contains(&doc.path)
    });

    if ticket.meta.documents.len() != original_len {
        recompute_ticket_document_fingerprints(repo_root, ticket_id, ticket)?;
    }

    Ok(())
}

/// Ensure the canonical task detail document exists, is registered, and return its
/// ticket-relative path.
fn ensure_canonical_task_detail_doc(
    repo_root: &Path,
    ticket_id: &TicketId,
    ticket: &mut Ticket,
    task_number: u32,
    title: &str,
) -> anyhow::Result<String> {
    let (doc_name, rel_path) = canonical_task_detail_doc(task_number);
    let ticket_path = ticket_dir(repo_root, ticket_id);
    let abs_path = ticket_path.join(&rel_path);

    if let Some(existing) = ticket
        .meta
        .documents
        .iter()
        .find(|doc| doc.name == doc_name)
        && existing.path != rel_path
    {
        anyhow::bail!(
            "task detail document {} is registered at unexpected path {} (expected {})",
            doc_name,
            existing.path,
            rel_path
        );
    }
    if let Some(existing) = ticket
        .meta
        .documents
        .iter()
        .find(|doc| doc.path == rel_path && doc.name != doc_name)
    {
        anyhow::bail!(
            "ticket-relative path {} is already registered as document {}",
            rel_path,
            existing.name
        );
    }

    let mut _created_file = false;
    if !abs_path.is_file() {
        if let Some(parent) = abs_path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                anyhow::anyhow!("failed to create directory {}: {error}", parent.display())
            })?;
        }
        fs::write(&abs_path, format!("# Task {task_number}: {title}\n\n"))
            .map_err(|error| anyhow::anyhow!("failed to write {}: {error}", abs_path.display()))?;
        _created_file = true;
    }

    if !ticket.meta.documents.iter().any(|doc| doc.name == doc_name) {
        ticket
            .meta
            .documents
            .push(tandem_core::ticket::TicketDocument {
                name: doc_name.clone(),
                path: rel_path.clone(),
            });
    }

    Ok(rel_path)
}

pub(crate) fn handle_task_add(
    id: String,
    title: String,
    file: Vec<String>,
    verification: Option<String>,
    notes: Option<String>,
    json: bool,
) -> anyhow::Result<()> {
    let current_dir = env::current_dir().map_err(|error| anyhow::anyhow!("{error}"))?;
    let repo_root = discover_repo_root(&current_dir).map_err(|error| anyhow::anyhow!("{error}"))?;
    let config = load_config(&repo_root).map_err(|error| anyhow::anyhow!("{error}"))?;
    let store = FileTicketStore::new(repo_root.clone());
    let ticket_id = parse_ticket_id_input(&id, &config.id_prefix)?;

    if title.trim().is_empty() {
        anyhow::bail!("task title must not be empty");
    }

    let mut ticket = load_and_bump(&store, &ticket_id)?;

    let next_number = ticket
        .state
        .tasks
        .iter()
        .map(|t| t.number)
        .max()
        .unwrap_or(0)
        + 1;

    let files = normalize_task_files(file);
    let verification = verification.and_then(|v| if v.trim().is_empty() { None } else { Some(v) });
    let notes = notes.and_then(|n| if n.trim().is_empty() { None } else { Some(n) });
    let detail_path =
        ensure_canonical_task_detail_doc(&repo_root, &ticket_id, &mut ticket, next_number, &title)?;

    ticket.state.tasks.push(Task {
        number: next_number,
        title,
        status: TaskStatus::Todo,
        files,
        verification,
        notes,
        detail_path: Some(detail_path),
    });

    recompute_ticket_document_fingerprints(&repo_root, &ticket_id, &mut ticket)?;
    persist_and_output(&store, &ticket, json)?;
    Ok(())
}

pub(crate) fn handle_task_list(id: String, json: bool) -> anyhow::Result<()> {
    let current_dir = env::current_dir().map_err(|error| anyhow::anyhow!("{error}"))?;
    let repo_root = discover_repo_root(&current_dir).map_err(|error| anyhow::anyhow!("{error}"))?;
    let config = load_config(&repo_root).map_err(|error| anyhow::anyhow!("{error}"))?;
    let store = FileTicketStore::new(repo_root);
    let ticket_id = parse_ticket_id_input(&id, &config.id_prefix)?;
    let ticket = store
        .load_ticket(&ticket_id)
        .map_err(|error| anyhow::anyhow!("{error}"))?;

    if json {
        println!("{}", serde_json::to_string_pretty(&ticket.state.tasks)?);
    } else {
        if ticket.state.tasks.is_empty() {
            println!("No tasks found.");
            return Ok(());
        }
        let mut builder = Builder::new();
        builder.push_record(["#", "STATUS", "TITLE"]);
        for task in &ticket.state.tasks {
            builder.push_record([&task.number.to_string(), task.status.as_str(), &task.title]);
        }
        println!("{}", builder.build().with(Style::blank()));
    }
    Ok(())
}

pub(crate) fn handle_task_complete(id: String, number: u32, json: bool) -> anyhow::Result<()> {
    let current_dir = env::current_dir().map_err(|error| anyhow::anyhow!("{error}"))?;
    let repo_root = discover_repo_root(&current_dir).map_err(|error| anyhow::anyhow!("{error}"))?;
    let config = load_config(&repo_root).map_err(|error| anyhow::anyhow!("{error}"))?;
    let store = FileTicketStore::new(repo_root);
    let ticket_id = parse_ticket_id_input(&id, &config.id_prefix)?;

    let mut ticket = load_and_bump(&store, &ticket_id)?;

    let (idx, _) = find_task(&ticket.state.tasks, number)?;
    ticket.state.tasks[idx].status = TaskStatus::Done;

    persist_and_output(&store, &ticket, json)?;
    Ok(())
}

pub(crate) fn handle_task_remove(id: String, number: u32, json: bool) -> anyhow::Result<()> {
    let current_dir = env::current_dir().map_err(|error| anyhow::anyhow!("{error}"))?;
    let repo_root = discover_repo_root(&current_dir).map_err(|error| anyhow::anyhow!("{error}"))?;
    let config = load_config(&repo_root).map_err(|error| anyhow::anyhow!("{error}"))?;
    let store = FileTicketStore::new(repo_root.clone());
    let ticket_id = parse_ticket_id_input(&id, &config.id_prefix)?;

    let mut ticket = load_and_bump(&store, &ticket_id)?;

    let (idx, _) = find_task(&ticket.state.tasks, number)?;
    ticket.state.tasks.remove(idx);
    prune_unlinked_canonical_task_detail_docs(&repo_root, &ticket_id, &mut ticket)?;

    persist_and_output(&store, &ticket, json)?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn handle_task_edit(
    id: String,
    number: u32,
    title: Option<String>,
    file: Option<Vec<String>>,
    clear_files: bool,
    verification: Option<String>,
    notes: Option<String>,
    json: bool,
) -> anyhow::Result<()> {
    let current_dir = env::current_dir().map_err(|error| anyhow::anyhow!("{error}"))?;
    let repo_root = discover_repo_root(&current_dir).map_err(|error| anyhow::anyhow!("{error}"))?;
    let config = load_config(&repo_root).map_err(|error| anyhow::anyhow!("{error}"))?;
    let store = FileTicketStore::new(repo_root.clone());
    let ticket_id = parse_ticket_id_input(&id, &config.id_prefix)?;

    let mut ticket = load_and_bump(&store, &ticket_id)?;

    let (idx, _) = find_task(&ticket.state.tasks, number)?;
    let task = &mut ticket.state.tasks[idx];
    if let Some(value) = title {
        if value.trim().is_empty() {
            anyhow::bail!("task title must not be empty");
        }
        task.title = value;
    }
    if let Some(value) = file {
        task.files = normalize_task_files(value);
    }
    if clear_files {
        task.files = Vec::new();
    }
    if let Some(value) = verification {
        task.verification = if value.trim().is_empty() {
            None
        } else {
            Some(value)
        };
    }
    if let Some(value) = notes {
        task.notes = if value.trim().is_empty() {
            None
        } else {
            Some(value)
        };
    }

    persist_and_output(&store, &ticket, json)?;
    Ok(())
}

pub(crate) fn handle_task_detail_ensure(id: String, number: u32, json: bool) -> anyhow::Result<()> {
    let current_dir = env::current_dir().map_err(|error| anyhow::anyhow!("{error}"))?;
    let repo_root = discover_repo_root(&current_dir).map_err(|error| anyhow::anyhow!("{error}"))?;
    let config = load_config(&repo_root).map_err(|error| anyhow::anyhow!("{error}"))?;
    let store = FileTicketStore::new(repo_root.clone());
    let ticket_id = parse_ticket_id_input(&id, &config.id_prefix)?;

    let mut ticket = load_and_bump(&store, &ticket_id)?;
    let (idx, _) = find_task(&ticket.state.tasks, number)?;
    let (doc_name, rel_path) = canonical_task_detail_doc(number);
    let ticket_path = ticket_dir(&repo_root, &ticket_id);
    let abs_path = ticket_path.join(&rel_path);

    if let Some(existing) = ticket
        .meta
        .documents
        .iter()
        .find(|doc| doc.name == doc_name)
        && existing.path != rel_path
    {
        anyhow::bail!(
            "task detail document {} is registered at unexpected path {} (expected {})",
            doc_name,
            existing.path,
            rel_path
        );
    }
    if let Some(existing) = ticket
        .meta
        .documents
        .iter()
        .find(|doc| doc.path == rel_path && doc.name != doc_name)
    {
        anyhow::bail!(
            "ticket-relative path {} is already registered as document {}",
            rel_path,
            existing.name
        );
    }

    let mut created_file = false;
    if !abs_path.is_file() {
        if let Some(parent) = abs_path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                anyhow::anyhow!("failed to create directory {}: {error}", parent.display())
            })?;
        }
        fs::write(&abs_path, format!("# Task {number}\n\n"))
            .map_err(|error| anyhow::anyhow!("failed to write {}: {error}", abs_path.display()))?;
        created_file = true;
    }

    if !ticket.meta.documents.iter().any(|doc| doc.name == doc_name) {
        ticket
            .meta
            .documents
            .push(tandem_core::ticket::TicketDocument {
                name: doc_name.clone(),
                path: rel_path.clone(),
            });
    }
    ticket.state.tasks[idx].detail_path = Some(rel_path.clone());
    recompute_ticket_document_fingerprints(&repo_root, &ticket_id, &mut ticket)?;

    if let Err(error) = store
        .update_ticket(&ticket)
        .map_err(|error| anyhow::anyhow!("{error}"))
    {
        if created_file {
            let _ = fs::remove_file(&abs_path);
        }
        return Err(error);
    }

    if json {
        println!(
            "{}",
            serde_json::json!({
                "ticket_id": ticket_id.as_str(),
                "task_number": number,
                "name": doc_name,
                "detail_path": rel_path,
                "path": abs_path.to_string_lossy(),
            })
        );
    } else {
        println!("{}", abs_path.display());
    }
    Ok(())
}

pub(crate) fn handle_task_set(id: String, tasks_json: String, json: bool) -> anyhow::Result<()> {
    let current_dir = env::current_dir().map_err(|error| anyhow::anyhow!("{error}"))?;
    let repo_root = discover_repo_root(&current_dir).map_err(|error| anyhow::anyhow!("{error}"))?;
    let config = load_config(&repo_root).map_err(|error| anyhow::anyhow!("{error}"))?;
    let store = FileTicketStore::new(repo_root.clone());
    let ticket_id = parse_ticket_id_input(&id, &config.id_prefix)?;

    let mut ticket = load_and_bump(&store, &ticket_id)?;

    let mut new_tasks: Vec<Task> = serde_json::from_str(&tasks_json)
        .map_err(|error| anyhow::anyhow!("invalid tasks JSON: {error}"))?;

    // Validate task numbers are >= 1 and unique
    if new_tasks.iter().any(|t| t.number == 0) {
        anyhow::bail!("task numbers must be >= 1");
    }
    let mut seen = std::collections::BTreeSet::new();
    for task in &new_tasks {
        if !seen.insert(task.number) {
            anyhow::bail!("duplicate task number: {}", task.number);
        }
    }
    if new_tasks.iter().any(|t| t.title.trim().is_empty()) {
        anyhow::bail!("task title must not be empty");
    }
    for task in &mut new_tasks {
        task.detail_path = Some(ensure_canonical_task_detail_doc(
            &repo_root,
            &ticket_id,
            &mut ticket,
            task.number,
            &task.title,
        )?);
    }

    ticket.state.tasks = new_tasks;
    prune_unlinked_canonical_task_detail_docs(&repo_root, &ticket_id, &mut ticket)?;
    recompute_ticket_document_fingerprints(&repo_root, &ticket_id, &mut ticket)?;

    persist_and_output(&store, &ticket, json)?;
    Ok(())
}
