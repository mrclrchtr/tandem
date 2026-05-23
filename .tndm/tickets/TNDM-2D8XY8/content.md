## Goal

Eliminate the "inline task" code path. Every task gets a canonical `tasks/task-XX.md` detail doc at creation time. No backward compatibility — remove the dead path, don't gate it.

## Approach

### Rust CLI (`tandem-cli`)

- `TaskCommand::Add` — remove `detail_path` field. `handle_task_add` always runs detail-ensure inline: creates `tasks/task-{N}.md` with `# Task {N}: {title}\n\n`, registers it in `meta.documents`, sets `detail_path`.
- `TaskCommand::Edit` — remove `detail_path` field. Path is always canonical, never user-editable.
- `TaskDetailCommand` — delete `Clear` variant. Delete `handle_task_detail_clear`.
- `handle_task_set` — auto-create detail docs for any incoming task whose canonical doc doesn't exist.
- `validate_registered_task_detail_path` — role flips from "validate existing" to "ensure canonical doc exists, return path." Rename or inline.
- `prune_unlinked_canonical_task_detail_docs` — keeps working as-is for task removal.
- `Task` struct — keeps `detail_path: Option<String>` for loading existing `state.toml` files; new tasks always populate it.

### supi-flow plugin

- `supi_flow_task` params — remove `clear_detail`. When `detail` is provided, write full content. When omitted, still call detail ensure (minimal template is fine).
- `CLAUDE.md` — replace "use headline-only tasks when possible" with guidance that every task always gets a detail doc automatically.

### Test changes

- Remove `task_detail_clear_detaches_link_without_deleting_doc` integration test
- Remove `TaskDetailCommand::Clear` match arm tests
- Update `handle_task_add` tests that exercise `--detail-path`
- Remove `clear_detail` tests in `flow-tools.test.ts`

## Non-goals

- Migrating existing inline tasks (existing state.toml files with null detail_path remain loadable)
- Changing the Task struct (stays backward-compatible)
