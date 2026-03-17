#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use std::{
    env, fs,
    io::{self, IsTerminal, Read},
    path::PathBuf,
};

use clap::{Args, Parser, Subcommand};
use serde::Serialize;
use tandem_core::{
    awareness::compare_snapshots,
    ports::TicketStore,
    ticket::{NewTicket, TicketId, TicketMeta, TicketPriority, TicketStatus, TicketType},
};
use tandem_repo::GitAwarenessProvider;
use tandem_storage::{
    FileTicketStore, TandemConfig, discover_repo_root, load_config, load_ticket_snapshot,
    ticket_dir,
};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

#[derive(Serialize)]
struct TicketJsonEntry<'a> {
    #[serde(flatten)]
    meta: &'a tandem_core::ticket::TicketMeta,
    #[serde(flatten)]
    state: &'a tandem_core::ticket::TicketState,
    content_path: String,
}

#[derive(Serialize)]
struct TicketJson<'a> {
    schema_version: u64,
    #[serde(flatten)]
    ticket: TicketJsonEntry<'a>,
}

#[derive(Serialize)]
struct TicketListJson<'a> {
    schema_version: u64,
    tickets: Vec<TicketJsonEntry<'a>>,
}

fn ticket_content_path(id: &tandem_core::ticket::TicketId) -> String {
    format!(".tndm/tickets/{}/content.md", id)
}

#[derive(Parser, Debug)]
#[command(
    name = "tndm",
    about = "tandem: git-aware ticket coordination",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Format/normalize tandem files.
    Fmt {
        /// Do not write changes.
        #[arg(long)]
        check: bool,
    },

    /// Ticket operations.
    Ticket {
        #[command(subcommand)]
        command: TicketCommand,
    },

    /// Show awareness of relevant ticket changes elsewhere.
    Awareness(AwarenessArgs),
}

#[derive(Args, Debug)]
struct AwarenessArgs {
    #[arg(long)]
    against: String,

    #[command(flatten)]
    output: OutputArgs,
}

#[derive(Args, Debug)]
struct OutputArgs {
    /// Output as JSON instead of human-readable text.
    #[arg(long)]
    json: bool,
}

#[derive(Subcommand, Debug)]
enum TicketCommand {
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

        #[command(flatten)]
        output: OutputArgs,
    },
    Show {
        id: String,

        #[command(flatten)]
        output: OutputArgs,
    },
    List {
        #[command(flatten)]
        output: OutputArgs,
    },
    /// Update an existing ticket.
    Update {
        /// Ticket ID to update.
        id: String,

        /// New status (todo, in_progress, blocked, done).
        #[arg(long)]
        status: Option<String>,

        /// New priority (p0–p4).
        #[arg(long)]
        priority: Option<String>,

        /// New title.
        #[arg(long)]
        title: Option<String>,

        /// New ticket type (task, bug, feature, chore, epic).
        #[arg(long = "type")]
        ticket_type: Option<String>,

        /// Comma-separated tags (replaces existing list, empty string clears).
        #[arg(long)]
        tags: Option<String>,

        /// Comma-separated ticket IDs for dependencies (replaces existing list, empty string clears).
        #[arg(long)]
        depends_on: Option<String>,

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
}

const DEFAULT_CONTENT_TEMPLATE: &str = "## Description\n\n## Design\n\n## Acceptance\n\n## Notes\n";
const CROCKFORD_BASE32: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Fmt { check } => handle_fmt(check),
        Command::Ticket { command } => match command {
            TicketCommand::Create {
                title,
                id,
                content_file,
                content,
                output,
            } => handle_ticket_create(title, id, content_file, content, output.json),
            TicketCommand::Show { id, output } => handle_ticket_show(id, output.json),
            TicketCommand::List { output } => handle_ticket_list(output.json),
            TicketCommand::Update {
                id,
                status,
                priority,
                title,
                ticket_type,
                tags,
                depends_on,
                content_file,
                content,
                output,
            } => handle_ticket_update(
                id,
                status,
                priority,
                title,
                ticket_type,
                tags,
                depends_on,
                content_file,
                content,
                output.json,
            ),
        },
        Command::Awareness(args) => handle_awareness(args),
    }
}

fn handle_fmt(check: bool) -> anyhow::Result<()> {
    let current_dir = env::current_dir().map_err(|error| anyhow::anyhow!("{error}"))?;
    let repo_root = discover_repo_root(&current_dir).map_err(|error| anyhow::anyhow!("{error}"))?;
    let store = FileTicketStore::new(repo_root.clone());
    let ids = store
        .list_ticket_ids()
        .map_err(|error| anyhow::anyhow!("{error}"))?;

    let mut changed_files = Vec::new();

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
    }

    if check && !changed_files.is_empty() {
        for path in &changed_files {
            println!("{}", path.display());
        }
        anyhow::bail!("tndm fmt --check found non-canonical tandem files");
    }

    for path in &changed_files {
        println!("{}", path.display());
    }

    Ok(())
}

fn handle_ticket_create(
    title: String,
    id: Option<String>,
    content_file: Option<PathBuf>,
    content: Option<String>,
    json: bool,
) -> anyhow::Result<()> {
    let current_dir = env::current_dir().map_err(|error| anyhow::anyhow!("{error}"))?;
    let repo_root = discover_repo_root(&current_dir).map_err(|error| anyhow::anyhow!("{error}"))?;
    let store = FileTicketStore::new(repo_root.clone());
    let config = load_config(&repo_root).map_err(|error| anyhow::anyhow!("{error}"))?;

    let ticket_id = match id {
        Some(value) => TicketId::parse(value)?,
        None => generate_ticket_id(&store, &config.id_prefix)?,
    };

    let content = load_ticket_content(content_file, content, &config)?;
    let meta = TicketMeta::new(ticket_id, title)?;

    let ticket = store
        .create_ticket(NewTicket { meta, content })
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

fn handle_ticket_show(id: String, json: bool) -> anyhow::Result<()> {
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
        print!("## meta.toml\n{}\n", ticket.meta.to_canonical_toml());
        print!("## state.toml\n{}\n", ticket.state.to_canonical_toml());
        print!("## content.md\n{}", ticket.content);
    }
    Ok(())
}

fn handle_ticket_list(json: bool) -> anyhow::Result<()> {
    let current_dir = env::current_dir().map_err(|error| anyhow::anyhow!("{error}"))?;
    let repo_root = discover_repo_root(&current_dir).map_err(|error| anyhow::anyhow!("{error}"))?;
    let store = FileTicketStore::new(repo_root);
    let ids = store
        .list_ticket_ids()
        .map_err(|error| anyhow::anyhow!("{error}"))?;

    if json {
        let mut tickets = Vec::new();
        for id in ids {
            let ticket = store
                .load_ticket(&id)
                .map_err(|error| anyhow::anyhow!("{error}"))?;
            tickets.push(ticket);
        }
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
        for id in ids {
            let ticket = store
                .load_ticket(&id)
                .map_err(|error| anyhow::anyhow!("{error}"))?;
            println!(
                "{}\t{}\t{}",
                id,
                ticket.state.status.as_str(),
                ticket.meta.title
            );
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn handle_ticket_update(
    id: String,
    status: Option<String>,
    priority: Option<String>,
    title: Option<String>,
    ticket_type: Option<String>,
    tags: Option<String>,
    depends_on: Option<String>,
    content_file: Option<PathBuf>,
    content: Option<String>,
    json: bool,
) -> anyhow::Result<()> {
    let stdin_content = if content_file.is_none() && content.is_none() && !io::stdin().is_terminal()
    {
        let mut buf = String::new();
        io::stdin()
            .read_to_string(&mut buf)
            .map_err(|error| anyhow::anyhow!("{error}"))?;
        if buf.is_empty() { None } else { Some(buf) }
    } else {
        None
    };

    if status.is_none()
        && priority.is_none()
        && title.is_none()
        && ticket_type.is_none()
        && tags.is_none()
        && depends_on.is_none()
        && content_file.is_none()
        && content.is_none()
        && stdin_content.is_none()
    {
        anyhow::bail!("at least one update flag is required");
    }

    let current_dir = env::current_dir().map_err(|error| anyhow::anyhow!("{error}"))?;
    let repo_root = discover_repo_root(&current_dir).map_err(|error| anyhow::anyhow!("{error}"))?;
    let store = FileTicketStore::new(repo_root);
    let ticket_id = TicketId::parse(id)?;
    let mut ticket = store
        .load_ticket(&ticket_id)
        .map_err(|error| anyhow::anyhow!("{error}"))?;

    if let Some(value) = status {
        ticket.state.status = TicketStatus::parse(&value)?;
    }
    if let Some(value) = priority {
        ticket.meta.priority = TicketPriority::parse(&value)?;
    }
    if let Some(value) = title {
        if value.trim().is_empty() {
            anyhow::bail!("title must not be empty");
        }
        ticket.meta.title = value;
    }
    if let Some(value) = ticket_type {
        ticket.meta.ticket_type = TicketType::parse(&value)?;
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

fn format_awareness_text(report: &tandem_core::awareness::AwarenessReport) -> String {
    use tandem_core::awareness::AwarenessChangeKind;

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
        if let Some(ref depends_on) = ticket.fields.depends_on {
            output.push_str(&format!(
                "  depends_on: {:?} -> {:?}\n",
                depends_on.current, depends_on.against
            ));
        }
    }

    output
}

fn handle_awareness(args: AwarenessArgs) -> anyhow::Result<()> {
    let current_dir = env::current_dir().map_err(|error| anyhow::anyhow!("{error}"))?;
    let repo_root = discover_repo_root(&current_dir).map_err(|error| anyhow::anyhow!("{error}"))?;

    let current_snapshot =
        load_ticket_snapshot(&repo_root).map_err(|error| anyhow::anyhow!("{error}"))?;

    let provider = GitAwarenessProvider::new(repo_root);
    let against_snapshot = match provider
        .materialize_ref_snapshot(&args.against)
        .map_err(|error| anyhow::anyhow!("{error}"))?
    {
        None => tandem_core::awareness::TicketSnapshot::default(),
        Some(snapshot) => load_ticket_snapshot(snapshot.path()).map_err(|error| {
            anyhow::anyhow!(
                "failed to load materialized snapshot for ref `{}`: {}",
                args.against,
                snapshot.sanitize_error_text(&error.to_string())
            )
        })?,
    };

    let report = compare_snapshots(&args.against, &current_snapshot, &against_snapshot);

    if args.output.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print!("{}", format_awareness_text(&report));
    }
    Ok(())
}

fn generate_ticket_id(store: &FileTicketStore, prefix: &str) -> anyhow::Result<TicketId> {
    use rand::RngExt;

    let mut rng = rand::rng();

    loop {
        let suffix: String = (0..6)
            .map(|_| {
                let index = rng.random_range(0..CROCKFORD_BASE32.len());
                CROCKFORD_BASE32[index] as char
            })
            .collect();

        let candidate = TicketId::parse(format!("{prefix}-{suffix}"))?;
        let exists = store
            .ticket_exists(&candidate)
            .map_err(|error| anyhow::anyhow!("{error}"))?;
        if !exists {
            return Ok(candidate);
        }
    }
}

fn load_ticket_content(
    content_file: Option<PathBuf>,
    content: Option<String>,
    config: &TandemConfig,
) -> anyhow::Result<String> {
    if let Some(path) = content_file {
        return fs::read_to_string(path).map_err(|error| anyhow::anyhow!("{error}"));
    }

    if let Some(value) = content {
        return Ok(value);
    }

    if !io::stdin().is_terminal() {
        let mut buf = String::new();
        io::stdin()
            .read_to_string(&mut buf)
            .map_err(|error| anyhow::anyhow!("{error}"))?;
        if !buf.is_empty() {
            return Ok(buf);
        }
    }

    if !config.content_template.is_empty() {
        return Ok(config.content_template.clone());
    }

    Ok(DEFAULT_CONTENT_TEMPLATE.to_string())
}
