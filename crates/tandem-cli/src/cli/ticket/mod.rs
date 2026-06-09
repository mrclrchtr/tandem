//! Command enums for the `tndm ticket` subcommand tree.
//!
//! These enums define the CLI surface and are the stable API contract.
//! Handler implementations live in their respective sub-modules.

pub(crate) mod create;
pub(crate) mod list;
pub(crate) mod show;
pub(crate) mod sync;
pub(crate) mod task;
pub(crate) mod update;

use clap::{Subcommand, ValueEnum};

use super::OutputArgs;

/// Filter tickets by definition state (backed by reserved tags).
#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub(crate) enum TicketDefinitionFilter {
    Ready,
    Questions,
    Unknown,
}

#[derive(Subcommand, Debug)]
pub(crate) enum TicketCommand {
    /// Create a new ticket.
    Create(create::TicketCreateArgs),
    Show {
        id: String,

        #[command(flatten)]
        output: OutputArgs,
    },
    List(list::TicketListArgs),
    /// Update an existing ticket.
    #[command(arg_required_else_help = true)]
    Update(update::TicketUpdateArgs),
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
        /// Ticket ID to synchronize (omit when using --all).
        id: Option<String>,

        /// Synchronize all tickets (including done).
        #[arg(long)]
        all: bool,

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
