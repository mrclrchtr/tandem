# Task 5: Use validate_tasks() in handle_task_set in tandem-cli

## Goal

Replace inlined task validation in CLI handlers with calls to the new `validate_tasks()` from `tandem-core`.

## What to change

In `crates/tandem-cli/src/cli/ticket.rs`:

### `handle_task_set` (line ~924)

Replace the inlined validation block (~15 lines):
```rust
// Validate task numbers are >= 1 and unique
if new_tasks.iter().any(|t| t.number == 0) {
    anyhow::bail!("task numbers must be >= 1");
}
let mut seen = std::collections::BTreeSet::new();
for task in &new_tasks {
    if !seen.insert(task.number) {
        anyhow::bail!("duplicate task number: {}", task.number);
    }
}
if new_tasks.iter().any(|t| t.title.trim().is_empty()) {
    anyhow::bail!("task title must not be empty");
}
```

With:
```rust
tandem_core::ticket::validate_tasks(&new_tasks)
    .map_err(|error| anyhow::anyhow!("{error}"))?;
```

### `handle_task_add` (line ~759)

Replace the inlined title validation:
```rust
if title.trim().is_empty() {
    anyhow::bail!("task title must not be empty");
}
```

This one validates a single title string (not a Task), and it's before the Task is constructed. The `validate_tasks` function validates `&[Task]`, so this inlined check serves a different purpose (early rejection before task construction). **Keep this inline check as-is** — it's a different validation path and not duplication.

### `handle_task_edit` (line ~849)

The title validation:
```rust
if let Some(value) = title {
    if value.trim().is_empty() {
        anyhow::bail!("task title must not be empty");
    }
    task.title = value;
}
```

This also validates before assignment — a different context. **Keep this inline check as-is**.

The primary consumer of `validate_tasks` is `handle_task_set`. The other handlers validate individual fields before Task construction, which is a different pattern.

### Import update

Add the import at the top of `ticket.rs` (if not already present — `tandem_core::ticket` is already imported, but `validate_tasks` needs to be in scope):

```rust
use tandem_core::ticket::validate_tasks;
```

Or call it qualified: `tandem_core::ticket::validate_tasks(...)`.

## Verification

- `cargo test --package tandem-cli` — all CLI tests pass
- `./tndm-dev ticket task set TNDM-YWG9H6 --tasks '[{"number":0,"title":"bad","status":"todo"}]'` → error "task numbers must be >= 1"
- `./tndm-dev ticket task set TNDM-YWG9H6 --tasks '[{"number":1,"title":"good","status":"todo"}]'` → succeeds
- `mise run clippy` clean

## Notes

- Only `handle_task_set` benefits directly from `validate_tasks()` since it receives a full `Vec<Task>`
- The other handlers validate individual fields before `Task` construction — keeping their inline checks is appropriate since the validation context differs
