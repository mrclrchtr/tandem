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
        /// Do not write changes; exit non-zero if formatting would change files.
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
            if check {
                eprintln!("tndm fmt --check: not implemented yet");
            } else {
                eprintln!("tndm fmt: not implemented yet");
            }
        }
        Command::Ticket { command } => match command {
            TicketCommand::Create { id } => {
                eprintln!("tndm ticket create {id}: not implemented yet")
            }
            TicketCommand::Show { id } => eprintln!("tndm ticket show {id}: not implemented yet"),
            TicketCommand::List => eprintln!("tndm ticket list: not implemented yet"),
        },
        Command::Awareness => eprintln!("tndm awareness: not implemented yet"),
    }

    Ok(())
}
