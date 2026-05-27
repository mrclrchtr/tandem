mod awareness;
mod doc;
mod fmt;
mod render;
mod ticket;
mod ticket_ctx;
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
            ticket::TicketCommand::Create(args) => ticket::handle_ticket_create(args),
            ticket::TicketCommand::Show { id, output } => {
                ticket::handle_ticket_show(id, output.json)
            }
            ticket::TicketCommand::List(args) => ticket::handle_ticket_list(args),
            ticket::TicketCommand::Update(args) => ticket::handle_ticket_update(args),
            ticket::TicketCommand::Doc { command } => match command {
                ticket::DocCommand::Create {
                    id,
                    name,
                    path,
                    output,
                } => doc::handle_doc_create(id, name, path, output.json),
            },
            ticket::TicketCommand::Sync { id, output } => {
                ticket::handle_ticket_sync(id, output.json)
            }
            ticket::TicketCommand::Task { command } => match command {
                ticket::TaskCommand::Add { id, title, output } => {
                    ticket::handle_task_add(id, title, output.json)
                }
                ticket::TaskCommand::List { id, output } => {
                    ticket::handle_task_list(id, output.json)
                }
                ticket::TaskCommand::Complete { id, number, output } => {
                    ticket::handle_task_complete(id, number, output.json)
                }
                ticket::TaskCommand::Remove { id, number, output } => {
                    ticket::handle_task_remove(id, number, output.json)
                }
                ticket::TaskCommand::Edit {
                    id,
                    number,
                    title,
                    output,
                } => ticket::handle_task_edit(id, number, title, output.json),
                ticket::TaskCommand::Detail { command } => match command {
                    ticket::TaskDetailCommand::Ensure { id, number, output } => {
                        ticket::handle_task_detail_ensure(id, number, output.json)
                    }
                },
                ticket::TaskCommand::Set { id, tasks, output } => {
                    ticket::handle_task_set(id, tasks, output.json)
                }
            },
        },
        Command::Awareness(args) => awareness::handle_awareness(args.against, args.output),
    }
}
