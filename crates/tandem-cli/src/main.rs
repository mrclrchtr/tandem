#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use std::{
    env, fs,
    io::{self, IsTerminal, Read},
    path::PathBuf,
};

use clap::{Args, Parser, Subcommand};
use tandem_core::{
    awareness::compare_snapshots,
    ports::TicketStore,
    ticket::{NewTicket, TicketId, TicketMeta},
};
use tandem_repo::GitAwarenessProvider;
use tandem_storage::{
    FileTicketStore, TandemConfig, discover_repo_root, load_config, load_ticket_snapshot,
    ticket_dir,
};

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
        #[arg(long)]
        content_file: Option<PathBuf>,
    },
    Show {
        id: String,
    },
    List,
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
            } => handle_ticket_create(title, id, content_file),
            TicketCommand::Show { id } => handle_ticket_show(id),
            TicketCommand::List => handle_ticket_list(),
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
) -> anyhow::Result<()> {
    let current_dir = env::current_dir().map_err(|error| anyhow::anyhow!("{error}"))?;
    let repo_root = discover_repo_root(&current_dir).map_err(|error| anyhow::anyhow!("{error}"))?;
    let store = FileTicketStore::new(repo_root.clone());
    let config = load_config(&repo_root).map_err(|error| anyhow::anyhow!("{error}"))?;

    let ticket_id = match id {
        Some(value) => TicketId::parse(value)?,
        None => generate_ticket_id(&store, &config.id_prefix)?,
    };

    let content = load_ticket_content(content_file, &config)?;

    let meta = TicketMeta::new(ticket_id.clone(), title)?;

    store
        .create_ticket(NewTicket { meta, content })
        .map_err(|error| anyhow::anyhow!("{error}"))?;

    println!("{ticket_id}");
    Ok(())
}

fn handle_ticket_show(id: String) -> anyhow::Result<()> {
    let current_dir = env::current_dir().map_err(|error| anyhow::anyhow!("{error}"))?;
    let repo_root = discover_repo_root(&current_dir).map_err(|error| anyhow::anyhow!("{error}"))?;
    let store = FileTicketStore::new(repo_root);
    let id = TicketId::parse(id)?;
    let ticket = store
        .load_ticket(&id)
        .map_err(|error| anyhow::anyhow!("{error}"))?;

    print!("## meta.toml\n{}\n", ticket.meta.to_canonical_toml());
    print!("## state.toml\n{}\n", ticket.state.to_canonical_toml());
    print!("## content.md\n{}", ticket.content);
    Ok(())
}

fn handle_ticket_list() -> anyhow::Result<()> {
    let current_dir = env::current_dir().map_err(|error| anyhow::anyhow!("{error}"))?;
    let repo_root = discover_repo_root(&current_dir).map_err(|error| anyhow::anyhow!("{error}"))?;
    let store = FileTicketStore::new(repo_root);
    let ids = store
        .list_ticket_ids()
        .map_err(|error| anyhow::anyhow!("{error}"))?;

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

    Ok(())
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
    println!("{}", serde_json::to_string_pretty(&report)?);
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
    config: &TandemConfig,
) -> anyhow::Result<String> {
    if let Some(path) = content_file {
        return fs::read_to_string(path).map_err(|error| anyhow::anyhow!("{error}"));
    }

    if !io::stdin().is_terminal() {
        let mut content = String::new();
        io::stdin()
            .read_to_string(&mut content)
            .map_err(|error| anyhow::anyhow!("{error}"))?;
        return Ok(content);
    }

    if !config.content_template.is_empty() {
        return Ok(config.content_template.clone());
    }

    Ok(DEFAULT_CONTENT_TEMPLATE.to_string())
}
