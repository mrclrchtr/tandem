# Task 5: Migrate ticket_task_tests.rs to TestRepo

## Goal

Migrate `crates/tandem-cli/tests/ticket_task_tests.rs` (860 lines) to use `TestRepo`.

## Files

**`crates/tandem-cli/tests/ticket_task_tests.rs`** — mechanical migration.

### Pattern replacements

This file already uses `create_test_ticket` from common. Replace with `repo.create_ticket(Some("ID"), "Title")`.

Standard replacements:
- Setup → `let repo = TestRepo::new();`
- CLI invocations → `repo.run(...)`, `repo.run_assert(...)`, `repo.run_json(...)`
- JSON task list parsing → `repo.run_json(&["ticket", "task", "list", "ID"])` returns the task array directly

### Tests

1. `task_add_creates_task_with_auto_number`
2. `task_add_increments_number`
3. `task_list_json_output`
4. `task_complete_marks_task_done`
5. `task_complete_twice_is_idempotent`
6. `task_complete_nonexistent_fails`
7. `task_remove_deletes_task`
8. `task_edit_updates_fields`
9. `task_set_bulk_replace`
10. `task_set_empty_clears`
11. `task_add_creates_detail_doc` (if present)
12. `task_remove_prunes_unlinked_detail_docs` (if present)

### Note

The task list JSON output is a bare array `[{number, title, status}]`, not the full ticket envelope. `repo.run_json()` should handle this correctly.

### Verification

```bash
cargo test -p tandem-cli ticket_task -- --nocapture
```
