# Task 3: Remove TaskDetailCommand::Clear and handle_task_detail_clear

## Goal

Delete `TaskDetailCommand::Clear` variant and `handle_task_detail_clear` function. Unlinking a detail doc is no longer a valid operation.

## Change

In `TaskDetailCommand`:
- Delete `Clear` variant and its fields (`id`, `number`, `output`).

In `ticket.rs`:
- Delete `handle_task_detail_clear` function entirely.

In `mod.rs`:
- Remove the `TaskDetailCommand::Clear { ... } =>` match arm.

If `TaskDetailCommand` becomes a single variant, consider simplifying it to a struct variant on `TaskCommand::Detail` directly, or keep the enum for clarity.
