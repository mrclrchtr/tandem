use std::{
    fs,
    path::{Path, PathBuf},
};

use clap::{Args, Subcommand, ValueEnum};
use tabled::{builder::Builder, settings::Style};
use tandem_core::{
    ports::TicketStore,
    ticket::{
        NewTicket, Task, TaskStatus, Ticket, TicketEffort, TicketId, TicketMeta, TicketPriority,
        TicketStatus, TicketType,
    },
};
use tandem_storage::{FileTicketStore, ticket_dir};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

use super::OutputArgs;
use super::doc::recompute_ticket_document_fingerprints;
use super::render::{TicketJsonEntry, output_ticket_json};
use super::ticket_ctx::TicketCtx;
use super::util::{
    DEFINITION_TAG_QUESTIONS, DEFINITION_TAG_READY, generate_ticket_id, load_ticket_content,
    parse_depends_on, parse_tags, read_stdin_if_no_flags, ticket_content_path,
};

// ─── Clap argument structs ────────────────────────────────

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
    pub(crate) output: OutputArgs,
}

#[derive(Args, Debug)]
pub(crate) struct TicketUpdateArgs {
    /// Ticket ID to update.
    pub(crate) id: String,

    /// New status [possible values: todo, in_progress, blocked, done].
    #[arg(long, short)]
    pub(crate) status: Option<TicketStatus>,

    /// New priority [possible values: p0, p1, p2, p3, p4].
    #[arg(long, short)]
    pub(crate) priority: Option<TicketPriority>,

    /// New title.
    #[arg(long, short)]
    pub(crate) title: Option<String>,

    /// New ticket type [possible values: task, bug, feature, chore, epic].
    #[arg(long = "type", short = 'T')]
    pub(crate) ticket_type: Option<TicketType>,

    /// Comma-separated tags (replaces existing list, empty string clears).
    #[arg(long, short = 'g')]
    pub(crate) tags: Option<String>,

    /// Tags to add (comma-separated, preserves existing tags).
    #[arg(long, conflicts_with = "tags")]
    pub(crate) add_tags: Option<String>,

    /// Tags to remove (comma-separated).
    #[arg(long, conflicts_with = "tags")]
    pub(crate) remove_tags: Option<String>,

    /// Comma-separated ticket IDs for dependencies (replaces existing list, empty string clears).
    #[arg(long, short = 'd')]
    pub(crate) depends_on: Option<String>,

    /// Effort estimate [possible values: xs, s, m, l, xl].
    #[arg(long, short = 'e')]
    pub(crate) effort: Option<TicketEffort>,

    /// Markdown file replacing content.
    #[arg(long, conflicts_with = "update_content")]
    pub(crate) content_file: Option<PathBuf>,

    /// Inline content body replacing existing content.
    #[arg(
        long = "content",
        id = "update_content",
        conflicts_with = "content_file"
    )]
    pub(crate) content: Option<String>,

    #[command(flatten)]
    pub(crate) output: OutputArgs,
}

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
    pub(crate) output: OutputArgs,
}

// ─── TicketUpdate: shared struct for create/update metadata ───

struct TicketUpdate {
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
    stdin_content: Option<String>,
}

impl TicketUpdate {
    fn is_empty(&self) -> bool {
        self.status.is_none()
            && self.priority.is_none()
            && self.title.is_none()
            && self.ticket_type.is_none()
            && self.tags.is_none()
            && self.add_tags.is_none()
            && self.remove_tags.is_none()
            && self.depends_on.is_none()
            && self.effort.is_none()
            && self.content_file.is_none()
            && self.content.is_none()
            && self.stdin_content.is_none()
    }

    fn apply(&self, ticket: &mut Ticket, id_prefix: &str) -> anyhow::Result<()> {
        if let Some(value) = self.status {
            ticket.state.status = value;
        }
        if let Some(value) = self.priority {
            ticket.meta.priority = value;
        }
        if let Some(ref value) = self.title {
            if value.trim().is_empty() {
                anyhow::bail!("title must not be empty");
            }
            ticket.meta.title = value.clone();
        }
        if let Some(value) = self.ticket_type {
            ticket.meta.ticket_type = value;
        }
        if let Some(ref value) = self.tags {
            ticket.meta.tags = parse_tags(value);
        }
        if let Some(ref value) = self.depends_on {
            ticket.meta.depends_on = parse_depends_on(value, id_prefix)?;
        }
        if let Some(value) = self.effort {
            ticket.meta.effort = Some(value);
        }
        if let Some(ref value) = self.remove_tags {
            let to_remove: Vec<String> = value.split(',').map(|s| s.trim().to_string()).collect();
            ticket.meta.tags.retain(|t| !to_remove.contains(t));
        }
        if let Some(ref value) = self.add_tags {
            for tag in value.split(',') {
                let trimmed = tag.trim().to_string();
                if !trimmed.is_empty() && !ticket.meta.tags.contains(&trimmed) {
                    ticket.meta.tags.push(trimmed);
                }
            }
            ticket.meta.tags.sort();
        }
        if let Some(ref path) = self.content_file {
            ticket.content = fs::read_to_string(path)
                .map_err(|error| anyhow::anyhow!("failed to read {}: {error}", path.display()))?;
        } else if let Some(ref value) = self.content {
            ticket.content = value.clone();
        } else if let Some(ref value) = self.stdin_content {
            ticket.content = value.clone();
        }
        Ok(())
    }

    fn from_create_args(args: &TicketCreateArgs) -> Self {
        Self {
            status: args.status,
            priority: args.priority,
            title: None,
            ticket_type: args.ticket_type,
            tags: args.tags.clone(),
            add_tags: None,
            remove_tags: None,
            depends_on: args.depends_on.clone(),
            effort: args.effort,
            content_file: args.content_file.clone(),
            content: args.content.clone(),
            stdin_content: None,
        }
    }

    fn from_update_args(args: &TicketUpdateArgs) -> Self {
        Self {
            status: args.status,
            priority: args.priority,
            title: args.title.clone(),
            ticket_type: args.ticket_type,
            tags: args.tags.clone(),
            add_tags: args.add_tags.clone(),
            remove_tags: args.remove_tags.clone(),
            depends_on: args.depends_on.clone(),
            effort: args.effort,
            content_file: args.content_file.clone(),
            content: args.content.clone(),
            stdin_content: None,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub(crate) enum TicketDefinitionFilter {
    Ready,
    Questions,
    Unknown,
}

#[derive(Subcommand, Debug)]
pub(crate) enum TicketCommand {
    /// Create a new ticket.
    Create(TicketCreateArgs),
    Show {
        id: String,

        #[command(flatten)]
        output: OutputArgs,
    },
    List(TicketListArgs),
    /// Update an existing ticket.
    #[command(arg_required_else_help = true)]
    Update(TicketUpdateArgs),
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
    /// Edit a task's title.
    Edit {
        /// Ticket ID.
        id: String,

        /// Task number to edit.
        number: u32,

        /// New title (optional).
        #[arg(long, short = 't')]
        title: Option<String>,

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
        super::render::print_ticket_human(&ticket);
    }
    Ok(())
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
pub(crate) fn handle_ticket_update(args: TicketUpdateArgs) -> anyhow::Result<()> {
    let ctx = TicketCtx::new()?;
    let ticket_id = ctx.resolve_id(&args.id)?;

    let mut update = TicketUpdate::from_update_args(&args);
    let stdin_content = read_stdin_if_no_flags(update.is_empty())?;

    if update.is_empty()
        && args.content_file.is_none()
        && args.content.is_none()
        && stdin_content.is_none()
    {
        anyhow::bail!(
            "at least one update flag is required\n\n  \
             Example: tndm ticket update {ticket_id} --status done\n\n  \
             Run 'tndm ticket update --help' for all options"
        );
    }

    // Include stdin content so update.apply() handles all content uniformly.
    update.stdin_content = stdin_content;

    let mut ticket = ctx
        .store
        .load_ticket(&ticket_id)
        .map_err(|error| anyhow::anyhow!("{error}"))?;

    update.apply(&mut ticket, ctx.id_prefix())?;

    ticket.state.revision += 1;
    ticket.state.updated_at = OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|error| anyhow::anyhow!("failed to format timestamp: {error}"))?;

    let updated = ctx
        .store
        .update_ticket(&ticket)
        .map_err(|error| anyhow::anyhow!("{error}"))?;

    if args.output.json {
        output_ticket_json(&updated)?;
    } else {
        println!("{ticket_id}");
    }
    Ok(())
}

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
        output_ticket_json(ticket)?;
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
        .map(|task| canonical_task_detail_doc(task.number).1)
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
/// ticket-relative path along with whether a new file was created on disk.
fn ensure_canonical_task_detail_doc(
    repo_root: &Path,
    ticket_id: &TicketId,
    ticket: &mut Ticket,
    task_number: u32,
    title: &str,
) -> anyhow::Result<(String, bool)> {
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

    let mut created_file = false;
    if !abs_path.is_file() {
        if let Some(parent) = abs_path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                anyhow::anyhow!("failed to create directory {}: {error}", parent.display())
            })?;
        }
        fs::write(&abs_path, format!("# Task {task_number}: {title}\n\n"))
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

    Ok((rel_path, created_file))
}

pub(crate) fn handle_task_add(id: String, title: String, json: bool) -> anyhow::Result<()> {
    let ctx = TicketCtx::new()?;
    let ticket_id = ctx.resolve_id(&id)?;

    if title.trim().is_empty() {
        anyhow::bail!("task title must not be empty");
    }

    let mut ticket = load_and_bump(&ctx.store, &ticket_id)?;

    let next_number = ticket
        .state
        .tasks
        .iter()
        .map(|t| t.number)
        .max()
        .unwrap_or(0)
        + 1;

    let (_rel_path, _created) = ensure_canonical_task_detail_doc(
        &ctx.repo_root,
        &ticket_id,
        &mut ticket,
        next_number,
        &title,
    )?;

    ticket.state.tasks.push(Task {
        number: next_number,
        title,
        status: TaskStatus::Todo,
    });

    recompute_ticket_document_fingerprints(&ctx.repo_root, &ticket_id, &mut ticket)?;
    persist_and_output(&ctx.store, &ticket, json)?;
    Ok(())
}

pub(crate) fn handle_task_list(id: String, json: bool) -> anyhow::Result<()> {
    let ctx = TicketCtx::new()?;
    let ticket_id = ctx.resolve_id(&id)?;
    let ticket = ctx
        .store
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
    let ctx = TicketCtx::new()?;
    let ticket_id = ctx.resolve_id(&id)?;

    let mut ticket = load_and_bump(&ctx.store, &ticket_id)?;

    let (idx, _) = find_task(&ticket.state.tasks, number)?;
    ticket.state.tasks[idx].status = TaskStatus::Done;

    persist_and_output(&ctx.store, &ticket, json)?;
    Ok(())
}

pub(crate) fn handle_task_remove(id: String, number: u32, json: bool) -> anyhow::Result<()> {
    let ctx = TicketCtx::new()?;
    let ticket_id = ctx.resolve_id(&id)?;

    let mut ticket = load_and_bump(&ctx.store, &ticket_id)?;

    let (idx, _) = find_task(&ticket.state.tasks, number)?;
    ticket.state.tasks.remove(idx);
    prune_unlinked_canonical_task_detail_docs(&ctx.repo_root, &ticket_id, &mut ticket)?;

    persist_and_output(&ctx.store, &ticket, json)?;
    Ok(())
}

pub(crate) fn handle_task_edit(
    id: String,
    number: u32,
    title: Option<String>,
    json: bool,
) -> anyhow::Result<()> {
    let ctx = TicketCtx::new()?;
    let ticket_id = ctx.resolve_id(&id)?;

    let mut ticket = load_and_bump(&ctx.store, &ticket_id)?;

    let (idx, _) = find_task(&ticket.state.tasks, number)?;
    let task = &mut ticket.state.tasks[idx];
    if let Some(value) = title {
        if value.trim().is_empty() {
            anyhow::bail!("task title must not be empty");
        }
        task.title = value;
    }

    persist_and_output(&ctx.store, &ticket, json)?;
    Ok(())
}

pub(crate) fn handle_task_detail_ensure(id: String, number: u32, json: bool) -> anyhow::Result<()> {
    let ctx = TicketCtx::new()?;
    let ticket_id = ctx.resolve_id(&id)?;

    let mut ticket = load_and_bump(&ctx.store, &ticket_id)?;
    let task_title = {
        let (_idx, task) = find_task(&ticket.state.tasks, number)?;
        task.title.clone()
    };
    let (rel_path, created_file) = ensure_canonical_task_detail_doc(
        &ctx.repo_root,
        &ticket_id,
        &mut ticket,
        number,
        &task_title,
    )?;

    let doc_name = canonical_task_detail_doc(number).0;
    let abs_path = ticket_dir(&ctx.repo_root, &ticket_id).join(&rel_path);

    recompute_ticket_document_fingerprints(&ctx.repo_root, &ticket_id, &mut ticket)?;

    if let Err(error) = ctx
        .store
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
    let ctx = TicketCtx::new()?;
    let ticket_id = ctx.resolve_id(&id)?;

    let mut ticket = load_and_bump(&ctx.store, &ticket_id)?;

    let mut new_tasks: Vec<Task> = serde_json::from_str(&tasks_json)
        .map_err(|error| anyhow::anyhow!("invalid tasks JSON: {error}"))?;

    tandem_core::ticket::validate_tasks(&new_tasks).map_err(|error| anyhow::anyhow!("{error}"))?;
    for task in &mut new_tasks {
        let (_rel_path, _created) = ensure_canonical_task_detail_doc(
            &ctx.repo_root,
            &ticket_id,
            &mut ticket,
            task.number,
            &task.title,
        )?;
    }

    ticket.state.tasks = new_tasks;
    prune_unlinked_canonical_task_detail_docs(&ctx.repo_root, &ticket_id, &mut ticket)?;
    recompute_ticket_document_fingerprints(&ctx.repo_root, &ticket_id, &mut ticket)?;

    persist_and_output(&ctx.store, &ticket, json)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use tandem_core::ticket::{Ticket, TicketId, TicketMeta, TicketState};

    use super::*;

    fn make_ticket() -> Ticket {
        let meta = TicketMeta::new(TicketId::parse("TNDM-TEST").unwrap(), "Test ticket").unwrap();
        let state = TicketState::initial("2026-01-01T00:00:00Z".to_string()).unwrap();
        Ticket {
            meta,
            state,
            content: String::new(),
        }
    }

    #[test]
    fn ticket_update_is_empty_when_all_fields_none() {
        let update = TicketUpdate {
            status: None,
            priority: None,
            title: None,
            ticket_type: None,
            tags: None,
            add_tags: None,
            remove_tags: None,
            depends_on: None,
            effort: None,
            content_file: None,
            content: None,
            stdin_content: None,
        };
        assert!(update.is_empty());
    }

    #[test]
    fn ticket_update_is_not_empty_when_any_field_set() {
        let update = TicketUpdate {
            status: Some(TicketStatus::InProgress),
            priority: None,
            title: None,
            ticket_type: None,
            tags: None,
            add_tags: None,
            remove_tags: None,
            depends_on: None,
            effort: None,
            content_file: None,
            content: None,
            stdin_content: None,
        };
        assert!(!update.is_empty());
    }

    #[test]
    fn ticket_update_apply_sets_status_and_priority() {
        let mut ticket = make_ticket();
        let update = TicketUpdate {
            status: Some(TicketStatus::InProgress),
            priority: Some(TicketPriority::P1),
            title: None,
            ticket_type: None,
            tags: None,
            add_tags: None,
            remove_tags: None,
            depends_on: None,
            effort: None,
            content_file: None,
            content: None,
            stdin_content: None,
        };
        update.apply(&mut ticket, "TNDM").unwrap();

        assert_eq!(ticket.state.status, TicketStatus::InProgress);
        assert_eq!(ticket.meta.priority, TicketPriority::P1);
    }

    #[test]
    fn ticket_update_apply_keeps_unchanged_fields_intact() {
        let mut ticket = make_ticket();
        let original_type = ticket.meta.ticket_type;
        let original_title = ticket.meta.title.clone();

        let update = TicketUpdate {
            status: Some(TicketStatus::Done),
            priority: None,
            title: None,
            ticket_type: None,
            tags: None,
            add_tags: None,
            remove_tags: None,
            depends_on: None,
            effort: None,
            content_file: None,
            content: None,
            stdin_content: None,
        };
        update.apply(&mut ticket, "TNDM").unwrap();

        assert_eq!(ticket.state.status, TicketStatus::Done);
        assert_eq!(ticket.meta.ticket_type, original_type);
        assert_eq!(ticket.meta.title, original_title);
    }

    #[test]
    fn ticket_update_apply_rejects_empty_title() {
        let mut ticket = make_ticket();
        let update = TicketUpdate {
            status: None,
            priority: None,
            title: Some("   ".to_string()),
            ticket_type: None,
            tags: None,
            add_tags: None,
            remove_tags: None,
            depends_on: None,
            effort: None,
            content_file: None,
            content: None,
            stdin_content: None,
        };
        let result = update.apply(&mut ticket, "TNDM");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("title must not be empty"),
            "expected empty-title error"
        );
    }

    #[test]
    fn ticket_update_from_create_args_copies_fields() {
        let args = TicketCreateArgs {
            title: "My ticket".to_string(),
            id: None,
            content_file: None,
            content: None,
            status: Some(TicketStatus::Todo),
            priority: Some(TicketPriority::P1),
            ticket_type: Some(TicketType::Feature),
            tags: Some("foo,bar".to_string()),
            depends_on: None,
            effort: Some(TicketEffort::S),
            output: super::super::OutputArgs { json: false },
        };
        let update = TicketUpdate::from_create_args(&args);

        assert_eq!(update.status, Some(TicketStatus::Todo));
        assert_eq!(update.priority, Some(TicketPriority::P1));
        assert_eq!(update.ticket_type, Some(TicketType::Feature));
        assert_eq!(update.tags, Some("foo,bar".to_string()));
        assert_eq!(update.title, None);
        assert_eq!(update.add_tags, None);
        assert_eq!(update.remove_tags, None);
    }

    #[test]
    fn ticket_update_from_update_args_copies_fields() {
        let args = TicketUpdateArgs {
            id: "TNDM-XXX".to_string(),
            status: Some(TicketStatus::Done),
            priority: Some(TicketPriority::P0),
            title: Some("New title".to_string()),
            ticket_type: Some(TicketType::Bug),
            tags: None,
            add_tags: Some("urgent".to_string()),
            remove_tags: Some("old".to_string()),
            depends_on: None,
            effort: None,
            content_file: None,
            content: None,
            output: super::super::OutputArgs { json: false },
        };
        let update = TicketUpdate::from_update_args(&args);

        assert_eq!(update.status, Some(TicketStatus::Done));
        assert_eq!(update.priority, Some(TicketPriority::P0));
        assert_eq!(update.title, Some("New title".to_string()));
        assert_eq!(update.ticket_type, Some(TicketType::Bug));
        assert_eq!(update.add_tags, Some("urgent".to_string()));
        assert_eq!(update.remove_tags, Some("old".to_string()));
    }
}
