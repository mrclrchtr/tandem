//! Handler for `tndm ticket update` and the shared `TicketUpdate` struct.

use std::{fs, path::PathBuf};

use clap::Args;
use tandem_core::{
    ports::TicketStore,
    ticket::{TicketEffort, TicketPriority, TicketStatus, TicketType},
};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

use crate::cli::render::output_ticket_json;
use crate::cli::ticket_ctx::TicketCtx;
use crate::cli::util::{parse_depends_on, parse_tags, read_stdin_if_no_flags};

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
    pub(crate) output: crate::cli::OutputArgs,
}

/// Shared struct for create/update metadata.
///
/// Provides a uniform way to collect field changes from either create or update args
/// and apply them to a ticket's metadata/state.
pub(crate) struct TicketUpdate {
    pub(crate) status: Option<TicketStatus>,
    pub(crate) priority: Option<TicketPriority>,
    pub(crate) title: Option<String>,
    pub(crate) ticket_type: Option<TicketType>,
    pub(crate) tags: Option<String>,
    pub(crate) add_tags: Option<String>,
    pub(crate) remove_tags: Option<String>,
    pub(crate) depends_on: Option<String>,
    pub(crate) effort: Option<TicketEffort>,
    pub(crate) content_file: Option<PathBuf>,
    pub(crate) content: Option<String>,
    pub(crate) stdin_content: Option<String>,
}

impl TicketUpdate {
    pub(crate) fn is_empty(&self) -> bool {
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

    pub(crate) fn apply(
        &self,
        ticket: &mut tandem_core::ticket::Ticket,
        id_prefix: &str,
    ) -> anyhow::Result<()> {
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

    pub(crate) fn from_create_args(args: &crate::cli::ticket::create::TicketCreateArgs) -> Self {
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

    pub(crate) fn from_update_args(args: &TicketUpdateArgs) -> Self {
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

#[cfg(test)]
mod tests {
    use tandem_core::ticket::{
        Ticket, TicketEffort, TicketId, TicketMeta, TicketPriority, TicketState, TicketStatus,
        TicketType,
    };

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
        let args = crate::cli::ticket::create::TicketCreateArgs {
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
            output: crate::cli::OutputArgs { json: false },
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
            output: crate::cli::OutputArgs { json: false },
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
