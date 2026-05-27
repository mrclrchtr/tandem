# CLI Refactoring Plan

Consolidate duplicated metadata-handling patterns in `ticket.rs`, eliminate
clap-dispatch boilerplate in `mod.rs`, and split the monolithic test file.

## Approach

Three changes that naturally converge:

1. **Shared `TicketUpdate` struct** — replace `no_explicit_create` /
   `no_explicit_update` guards and per-field `if let Some(...)` assignments
   with a single struct that self-reports empty/full and applies all fields in
   one pass.

2. **Pass structs through mod.rs** — extract inline clap subcommand variants
   into named `pub(crate)` structs so `mod.rs` dispatch becomes
   `handle_ticket_create(args)` instead of destructuring 10+ positional
   parameters.

3. **Split test file** — break `ticket_cli_tests.rs` (3012 lines) into five
   focused modules under `tests/` with shared helpers in `tests/common/mod.rs`.

No behavioral changes. All existing tests must pass without modification to
assertions.

## Files

### Modified

- `crates/tandem-cli/src/cli/ticket.rs` — new `TicketUpdate` struct + `apply()`;
  simplified handler signatures; task handler signatures unchanged (they already
  use clean parameter lists).

- `crates/tandem-cli/src/cli/mod.rs` — extract named clap structs for Create,
  Update, and other sub-commands; replace field-by-field destructuring with
  direct struct passing.

### Created

- `crates/tandem-cli/tests/common/mod.rs` — shared helpers: `create_test_ticket`,
  `write_prefix_config`, repo setup utilities.

- `crates/tandem-cli/tests/ticket_create_tests.rs` — all `ticket_create_*` tests.

- `crates/tandem-cli/tests/ticket_update_tests.rs` — all `ticket_update_*` tests.

- `crates/tandem-cli/tests/ticket_task_tests.rs` — all `task_*` tests.

- `crates/tandem-cli/tests/ticket_list_tests.rs` — `ticket_list_*`,
  `ticket_show_*` tests.

- `crates/tandem-cli/tests/ticket_config_tests.rs` — ID prefix, `tndm_config_*`
  tests.

### Removed

- `crates/tandem-cli/tests/ticket_cli_tests.rs` — contents moved to split files.

## Key design decisions

- `TicketUpdate` struct lives in `ticket.rs` (CLI-internal, no core dependency).
- `TicketUpdate::is_empty()` checks all 13 fields to replace both `no_explicit_*`
  guards.
- `TicketUpdate::apply(&self, ticket, id_prefix)` handles all field assignments,
  including tag merging and depends-on parsing.
- Content handling (file/stdin/inline) stays in the handler functions since
  create and update have slightly different content semantics.
- Test file split is purely organizational — zero assertion changes.

## Constraints

- `clap` types remain in `tandem-cli` only.
- `TicketCtx` is unchanged.
- No changes to `tandem-core`, `tandem-storage`, `tandem-repo`.
- All checks (`fmt`, `compile`, `arch`, `clippy`, `test`) must pass.
