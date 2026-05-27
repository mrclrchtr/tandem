//! Task handlers for `tndm ticket task *` commands.

use std::{collections::BTreeSet, fs, path::Path};

use tandem_core::{
    ports::TicketStore,
    ticket::{Task, TaskStatus, Ticket, TicketDocument, TicketId},
};
use tandem_storage::{FileTicketStore, ticket_dir};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

use crate::cli::doc::recompute_ticket_document_fingerprints;
use crate::cli::render::output_ticket_json;
use crate::cli::ticket_ctx::TicketCtx;

// ─── Internal helpers ───────────────────────────────────────────

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

fn is_canonical_task_detail_doc(doc: &TicketDocument) -> bool {
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
        .collect::<BTreeSet<_>>();

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
        ticket.meta.documents.push(TicketDocument {
            name: doc_name.clone(),
            path: rel_path.clone(),
        });
    }

    Ok((rel_path, created_file))
}

// ─── Handlers ───────────────────────────────────────────────────

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
        let mut builder = tabled::builder::Builder::new();
        builder.push_record(["#", "STATUS", "TITLE"]);
        for task in &ticket.state.tasks {
            builder.push_record([&task.number.to_string(), task.status.as_str(), &task.title]);
        }
        println!("{}", builder.build().with(tabled::settings::Style::blank()));
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
