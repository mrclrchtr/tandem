mod awareness;
mod doc;
mod fmt;
mod render;
mod ticket;
mod util;

use clap::{Args, Parser, Subcommand};

#[derive(Args, Debug)]
pub(crate) struct OutputArgs {
    /// Output as JSON instead of human-readable text.
    #[arg(long)]
    pub(crate) json: bool,
}

#[derive(Args, Debug)]
pub(crate) struct AwarenessArgs {
    #[arg(long)]
    pub(crate) against: String,

    #[command(flatten)]
    pub(crate) output: OutputArgs,
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
        command: ticket::TicketCommand,
    },

    /// Show awareness of relevant ticket changes elsewhere.
    Awareness(AwarenessArgs),
}

pub(crate) fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Fmt { check } => fmt::handle_fmt(check),
        Command::Ticket { command } => match command {
            ticket::TicketCommand::Create {
                title,
                id,
                content_file,
                content,
                status,
                priority,
                ticket_type,
                tags,
                depends_on,
                effort,
                output,
            } => ticket::handle_ticket_create(
                title,
                id,
                content_file,
                content,
                status,
                priority,
                ticket_type,
                tags,
                depends_on,
                effort,
                output.json,
            ),
            ticket::TicketCommand::Show { id, output } => {
                ticket::handle_ticket_show(id, output.json)
            }
            ticket::TicketCommand::List {
                all,
                definition,
                output,
            } => ticket::handle_ticket_list(output.json, all, definition),
            ticket::TicketCommand::Update {
                id,
                status,
                priority,
                title,
                ticket_type,
                tags,
                add_tags,
                remove_tags,
                depends_on,
                effort,
                content_file,
                content,
                output,
            } => ticket::handle_ticket_update(
                id,
                status,
                priority,
                title,
                ticket_type,
                tags,
                add_tags,
                remove_tags,
                depends_on,
                effort,
                content_file,
                content,
                output.json,
            ),
            ticket::TicketCommand::Doc { command } => match command {
                ticket::DocCommand::Create { id, name, output } => {
                    doc::handle_doc_create(id, name, output.json)
                }
            },
            ticket::TicketCommand::Sync { id, output } => {
                ticket::handle_ticket_sync(id, output.json)
            }
        },
        Command::Awareness(args) => awareness::handle_awareness(args.against, args.output),
    }
}
