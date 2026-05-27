# Task 4: Add validate_tasks() to tandem-core with unit tests

## Goal

Add a public `validate_tasks` function to `tandem-core` that centralizes task validation rules, with unit tests.

## What to change

In `crates/tandem-core/src/ticket/mod.rs`:

1. Add a public function after the `Task` struct definition:

```rust
/// Validate a task list against business rules.
///
/// Returns Ok(()) if all tasks are valid, or a ValidationError describing the first violation.
pub fn validate_tasks(tasks: &[Task]) -> Result<(), ValidationError> {
    if tasks.iter().any(|t| t.number == 0) {
        return Err(ValidationError::new("task numbers must be >= 1"));
    }
    let mut seen = std::collections::BTreeSet::new();
    for task in tasks {
        if !seen.insert(task.number) {
            return Err(ValidationError::new(format!(
                "duplicate task number: {}",
                task.number
            )));
        }
    }
    if tasks.iter().any(|t| t.title.trim().is_empty()) {
        return Err(ValidationError::new("task title must not be empty"));
    }
    Ok(())
}
```

2. Add unit tests in the existing `mod tests` block:
   - `validate_tasks_accepts_valid_tasks` — task with number=1, non-empty title
   - `validate_tasks_rejects_number_zero` — task number 0
   - `validate_tasks_rejects_duplicate_numbers` — two tasks with same number
   - `validate_tasks_rejects_empty_title` — task with whitespace-only title
   - `validate_tasks_accepts_empty_list` — empty vec

## Verification

- `cargo test --package tandem-core validate_tasks` — all new tests pass
- `mise run test` — no regressions
- `mise run clippy` clean

## Notes

- The function lives in `tandem-core` because validation rules are domain invariants, not CLI concerns
- Uses `BTreeSet` for deterministic duplicate detection (already used in `handle_task_set`)
- Returns `ValidationError` (the core error type), not `anyhow`
