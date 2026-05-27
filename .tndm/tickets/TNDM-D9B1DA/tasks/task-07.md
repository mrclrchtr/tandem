# Task 7: Migrate fmt_cli_tests.rs to TestRepo

## Goal

Migrate `crates/tandem-cli/tests/fmt_cli_tests.rs` (980 lines) to use `TestRepo`.

## Files

**`crates/tandem-cli/tests/fmt_cli_tests.rs`** — mechanical migration.

### Pattern replacements

This file has two categories of tests:

**Category A: Tests that create tickets via the CLI** (e.g. `fmt_adds_trailing_newline_to_content_md`). These use `create_test_ticket` → switch to `repo.create_ticket(Some("ID"), "Title")`.

**Category B: Tests that manually write files to test fmt against non-canonical state** (e.g. `fmt_check_reports_non_canonical_structured_files`). These manually create `ticket_dir` and write files. Switch to:
- `let repo = TestRepo::new();`
- Write files to `repo.path().join(".tndm/tickets/ID/...")`
- Run CLI via `repo.run(...)` / `repo.run_assert(...)`

### Special cases

- **Tests that write malformed TOML** to verify fmt error handling: Keep `fs::write()` calls but point at `repo.path().join(...)`. Cannot use `write_ticket()` because the goal is specific non-canonical formatting.

- **`fmt_preserves_windows_line_endings`**: Writes content with `\r\n` line endings — keep raw `fs::write()`.

- **Tests checking file contents after fmt**: Read back with `repo.read_ticket_file(id, filename)` where the file is one of `meta.toml`, `state.toml`, `content.md`.

### Tests (14 total)

`fmt_check_reports_non_canonical_structured_files`, `fmt_rewrites_non_canonical_structured_files`, `fmt_adds_trailing_newline_to_content_md`, `fmt_adds_trailing_newline_to_plan_md`, `fmt_adds_trailing_newline_to_task_detail_doc`, `fmt_check_reports_missing_trailing_newline`, `fmt_canonical_content_md_is_noop`, `fmt_check_passes_when_content_md_has_trailing_newline`, `fmt_collapses_multiple_trailing_newlines`, `fmt_normalizes_empty_content`, `fmt_check_reports_drift_and_missing_newline_together`, `fmt_fixes_structured_files_and_content_together`, `fmt_preserves_windows_line_endings`, `fmt_fails_when_managed_files_are_invalid`

### Verification

```bash
cargo test -p tandem-cli fmt_cli -- --nocapture
```
