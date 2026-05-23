# Task 4: Update handle_task_set to auto-create detail docs for incoming tasks

## Goal

`handle_task_set` currently calls `validate_registered_task_detail_path` for each incoming task, which errors if the detail doc doesn't exist. Change this to auto-create the doc instead.

## Change

In `handle_task_set`:
- Replace the `validate_registered_task_detail_path` loop with logic that, for each task, ensures the canonical `tasks/task-{N}.md` exists (creating it with `# Task {N}: {title}\n\n` if missing), registers it in `meta.documents`, and sets `detail_path = Some(rel_path)`.
- Reuse the same pattern from `handle_task_add` (after Task 1 is done) — extract a shared helper if the logic is identical.
