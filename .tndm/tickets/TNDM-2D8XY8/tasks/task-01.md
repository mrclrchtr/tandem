# Task 1: Remove --detail-path from TaskCommand::Add, auto-create detail doc in handle_task_add

## Goal

Make `handle_task_add` always create a canonical `tasks/task-{N}.md` detail doc and set `detail_path`. Remove the `--detail-path` CLI flag since the path is now always canonical.

## Change

In `TaskCommand::Add`:
- Remove `detail_path: Option<String>` field and its `#[arg(long)]` attribute.

In `handle_task_add`:
- Remove `detail_path` parameter from function signature.
- Remove the `let detail_path = validate_registered_task_detail_path(...)` call.
- After computing `next_number`, inline the detail-ensure logic: call `canonical_task_detail_doc(next_number)` to get `(doc_name, rel_path)`, create the file at `ticket_dir(...).join(&rel_path)` with content `# Task {next_number}: {title}\n\n` if it doesn't exist, register it in `ticket.meta.documents` if not already there, set `detail_path = Some(rel_path)` on the new task.
- Reuse the existing patterns from `handle_task_detail_ensure` but inline them.

In `mod.rs`:
- Remove `detail_path` from the `TaskCommand::Add` destructuring in the match arm.
- Remove `detail_path` from the `handle_task_add` call.
