# Task 7: Split ticket_cli_tests.rs into focused test modules

## Goal

Split the 3012-line `ticket_cli_tests.rs` into five focused test modules by CLI command. Zero assertion changes — pure file split.

## Files

- **Create** `crates/tandem-cli/tests/ticket_create_tests.rs`
- **Create** `crates/tandem-cli/tests/ticket_update_tests.rs`
- **Create** `crates/tandem-cli/tests/ticket_task_tests.rs`
- **Create** `crates/tandem-cli/tests/ticket_list_tests.rs`
- **Create** `crates/tandem-cli/tests/ticket_config_tests.rs`
- **Remove** `crates/tandem-cli/tests/ticket_cli_tests.rs`

## Procedure

For each target file:

1. Create the file with `mod common;` and `use common::*;` at the top.
2. Copy matching test functions from `ticket_cli_tests.rs`. Grouping:
   - `ticket_create_tests.rs` — all `fn ticket_create_*` tests
   - `ticket_update_tests.rs` — all `fn ticket_update_*` and `fn ticket_show_*` tests
   - `ticket_task_tests.rs` — all `fn task_*` tests
   - `ticket_list_tests.rs` — all `fn ticket_list_*` tests
   - `ticket_config_tests.rs` — all `fn tndm_config_*` and ID-prefix-related tests
3. Copy any `#[allow(...)]` attributes and `use` statements needed by those tests.
4. Verify each new file compiles and its tests pass.
5. After all five files are working, remove `ticket_cli_tests.rs`.

## Verification (per file)

- `cargo test -p tandem-cli --test ticket_create_tests` — all pass
- `cargo test -p tandem-cli --test ticket_update_tests` — all pass
- `cargo test -p tandem-cli --test ticket_task_tests` — all pass
- `cargo test -p tandem-cli --test ticket_list_tests` — all pass
- `cargo test -p tandem-cli --test ticket_config_tests` — all pass
- `cargo test -p tandem-cli` — full suite passes (no test left behind)

## Edge cases

- Any helper function used by only one group stays in that group's file.
- If a helper is used by multiple groups but was missed in task 6, move it to `common/mod.rs`.
- The `#![allow(clippy::disallowed_types)]` crate-level attribute goes in each new test file that needs it.
