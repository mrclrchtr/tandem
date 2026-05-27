# Task 4: Migrate ticket_update_tests.rs to TestRepo

## Goal

Migrate `crates/tandem-cli/tests/ticket_update_tests.rs` (1044 lines, the largest test file) to use `TestRepo`.

## Files

**`crates/tandem-cli/tests/ticket_update_tests.rs`** — mechanical migration.

### Pattern replacements

Same as previous files: `TestRepo::new()` for setup, `repo.run()` / `repo.run_assert()` / `repo.run_json()` for CLI invocations, `repo.create_ticket()` for setup tickets, `repo.path()` for file paths.

### Special cases in this file

- **`ticket_show_prints_exact_canonical_sections`**: This test verifies exact stdout formatting and uses a complex expected string with regex timestamp extraction. The `repo.run_assert()` return value is the stdout String, which can be asserted against directly. The `updated_at` timestamp regex pattern stays unchanged.

- **`ticket_show_surfaces_invalid_meta_toml_errors`**: This test writes malformed TOML directly (no valid meta.toml), then runs `ticket show` expecting a helpful error. This is a case where `repo.write_ticket()` can't be used (the goal is invalid state). Instead, the test manually writes files to `repo.path().join(".tndm/tickets/...")`.

- **`handle_update_content_from_stdin`**: Uses stdin piping via `.stdin(Stdio::piped())`. This uses a raw `Command` (not `repo.run()`). That's fine — the setup still uses `TestRepo::new()`.

- **Tests that use raw file writes** for specific state setup should keep their `fs::write()` calls pointed at `repo.path().join(...)`.

### Verification

```bash
cargo test -p tandem-cli ticket_update -- --nocapture
```
