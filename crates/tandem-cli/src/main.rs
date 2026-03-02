#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use clap::{Parser, Subcommand};

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
    Create { id: String },
    Show { id: String },
    List,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Fmt { check } => {
            let _ = check;
            anyhow::bail!("tndm fmt is not implemented yet");
        }
        Command::Ticket { command } => match command {
            TicketCommand::Create { id } => {
                anyhow::bail!("tndm ticket create {id}: not implemented yet")
            }
            TicketCommand::Show { id } => {
                anyhow::bail!("tndm ticket show {id}: not implemented yet")
            }
            TicketCommand::List => anyhow::bail!("tndm ticket list: not implemented yet"),
        },
        Command::Awareness => anyhow::bail!("tndm awareness: not implemented yet"),
    }
}
