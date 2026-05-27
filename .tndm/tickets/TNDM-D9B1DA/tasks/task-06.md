# Task 6: Migrate ticket_config_tests.rs to TestRepo

## Goal

Migrate `crates/tandem-cli/tests/ticket_config_tests.rs` (416 lines) to use `TestRepo`. Remove the local duplicate `create_test_ticket` function.

## Files

**`crates/tandem-cli/tests/ticket_config_tests.rs`** — mechanical migration + cleanup.

### Special: Remove local duplicate

This file defines its own `create_test_ticket(repo_root, id, title)` at the bottom (line 363-377). After migration, this is unused and must be removed. All callers switch to `repo.create_ticket(Some(id), title)`.

### Pattern replacements

- **Tests that need config**: Use `TestRepo::with_config("PROJ")` instead of `let repo_root = tempfile::tempdir(); fs::create_dir_all(...); write_prefix_config(repo_root.path(), "PROJ");`
- Standard replacements for CLI invocations → `repo.run(...)`, `repo.run_assert(...)`
- `repo_root.path()` → `repo.path()`

### Tests

1. `bare_ticket_show_uses_configured_prefix` — uses config prefix
2. `bare_ticket_update_uses_configured_prefix`
3. `bare_ticket_sync_uses_configured_prefix`
4. `bare_ticket_doc_create_uses_configured_prefix`
5. `ticket_doc_create_accepts_nested_ticket_relative_path`
6. `ticket_doc_create_rejects_absolute_and_traversing_paths`
7. `ticket_doc_create_rejects_existing_registered_path`
8. `bare_ticket_create_depends_on_uses_configured_prefix`
9. `bare_ticket_update_depends_on_uses_configured_prefix`
10. `doc_create_rejects_conflicting_path_for_existing_name`

### Verification

```bash
cargo test -p tandem-cli ticket_config -- --nocapture
```
