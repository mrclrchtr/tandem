# Task 2: Remove --detail-path from TaskCommand::Edit and handle_task_edit

## Goal

Remove the `detail_path` parameter from `TaskCommand::Edit` and `handle_task_edit`. The detail doc path is always canonical and never user-editable.

## Change

In `TaskCommand::Edit`:
- Remove `detail_path: Option<String>` field and its `#[arg(long)]` attribute.

In `handle_task_edit`:
- Remove `detail_path` parameter from function signature.
- Remove the `let validated_detail_path = ...` block and the `if let Some(detail_path) = validated_detail_path { task.detail_path = detail_path; }` line.

In `mod.rs`:
- Remove `detail_path` from the `TaskCommand::Edit` destructuring.
- Remove `detail_path` from the `handle_task_edit` call.
