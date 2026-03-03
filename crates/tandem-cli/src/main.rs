#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use std::{
    env, fs,
    io::{self, IsTerminal, Read},
    path::PathBuf,
};

use clap::{Parser, Subcommand};
use tandem_core::{
    ports::TicketStore,
    ticket::{NewTicket, TicketId, TicketMeta, TicketState},
};
use tandem_storage::{FileTicketStore, TandemConfig, discover_repo_root, load_config};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

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
        /// Do not write changes (not implemented yet).
        #[arg(long)]
        check: bool,
    },

    /// Ticket operations.
    Ticket {
        #[command(subcommand)]
        command: TicketCommand,
    },

    /// Show awareness of relevant ticket changes elsewhere.
    Awareness,
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
        Command::Fmt { check } => {
            let _ = check;
            anyhow::bail!("tndm fmt is not implemented yet");
        }
        Command::Ticket { command } => match command {
            TicketCommand::Create {
                title,
                id,
                content_file,
            } => handle_ticket_create(title, id, content_file),
            TicketCommand::Show { id } => handle_ticket_show(id),
            TicketCommand::List => anyhow::bail!("tndm ticket list: not implemented yet"),
        },
        Command::Awareness => anyhow::bail!("tndm awareness: not implemented yet"),
    }
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
    let updated_at = OffsetDateTime::now_utc().format(&Rfc3339)?;

    let meta = TicketMeta::new(ticket_id.clone(), title)?;
    let state = TicketState::initial(updated_at)?;

    store
        .create_ticket(NewTicket {
            meta,
            state,
            content,
        })
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

    print!("## meta.toml\n{}\n\n", ticket.meta.to_canonical_toml());
    print!("## state.toml\n{}\n\n", ticket.state.to_canonical_toml());
    print!("## content.md\n{}\n", ticket.content);
    Ok(())
}

fn generate_ticket_id(store: &FileTicketStore, prefix: &str) -> anyhow::Result<TicketId> {
    use rand::Rng;

    let mut rng = rand::thread_rng();

    loop {
        let suffix: String = (0..6)
            .map(|_| {
                let index = rng.gen_range(0..CROCKFORD_BASE32.len());
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
