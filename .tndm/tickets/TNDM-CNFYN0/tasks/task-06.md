# Task 6: Move shared test helpers to tests/common/mod.rs

## Goal

Extract shared test helpers from `ticket_cli_tests.rs` into `tests/common/mod.rs` so split test modules can reuse them.

## Files

- **Create** `crates/tandem-cli/tests/common/mod.rs`
- **Modify** `crates/tandem-cli/tests/ticket_cli_tests.rs` (remove moved helpers, update imports)

## Changes in `common/mod.rs`

Extract these items verbatim (no logic changes):

1. `create_test_ticket(repo_root, id, title) -> Ticket` — assembles and writes a complete ticket. This is the primary shared helper referenced in CLAUDE.md.

2. `write_prefix_config(repo_root, prefix)` — writes `.tndm/config.toml` with a custom ID prefix.

3. Any other helper functions used across multiple existing tests (e.g., timestamp generation utilities).

4. Common `use` statements (regex, time, tandem_core types).

## Changes in `ticket_cli_tests.rs`

1. Add `mod common;` at the top.
2. Add `use common::*;` or qualified `use common::{create_test_ticket, write_prefix_config};`.
3. Remove the moved function definitions.
4. Verify all tests still compile and pass.

## Verification

- `cargo test -p tandem-cli` — all tests pass
- Confirm `create_test_ticket` is still accessible from the original file
