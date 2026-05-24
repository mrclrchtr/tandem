# Strip Task to {number, title, status}

## Approach

Remove `files`, `verification`, `notes`, and `detail_path` from the `Task` struct. Keep `number`, `title`, `status` as the core identity. Everything else moves to the task detail markdown doc.

## Field disposition

| Field | Old location | New location | Why |
|-------|-------------|-------------|-----|
| `detail_path` | state.toml task struct | Derived: `tasks/task-{NN}.md` | Fully predictable from number |
| `notes` | state.toml task struct | Task detail markdown doc | Freeform text belongs in markdown |
| `verification` | state.toml task struct | Task detail markdown doc | Already documented in detail docs |
| `files` | state.toml task struct | Task detail markdown doc | Rarely accurate; better as doc section |

The document registry in `meta.toml` still tracks task detail docs for fingerprint verification. `detail_path` remains available via `canonical_task_detail_doc(number)`.

## What changes

- **Task struct** shrinks to `{number, title, status}`
- **CLI** drops `--file`, `--verification`, `--notes`, `--clear-files` from `task add` and `task edit`
- **Awareness** task snapshots shrink accordingly
- **Storage** backward-compat: old files with extra fields deserialize fine
- **Tests** updated for all crates

## What doesn't change

- Task detail docs still created at `tasks/task-NN.md` on `task add`
- Doc registry tracks them in `meta.toml`
- `canonical_task_detail_doc(number)` still available
- `task complete`, `task remove`, `task list` work the same
- JSON output for task lists becomes simpler

## Backward compatibility

Old `state.toml` files with extra fields deserialize fine — `#[serde(deny_unknown_fields)]` is not used. On next write, extra fields are dropped.
