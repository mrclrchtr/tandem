# Task 1: Create ticket/mod.rs with command enums

Create `crates/tandem-cli/src/cli/ticket/mod.rs` containing only the command enums extracted from `ticket.rs`:

- `TicketCommand` enum with all variants (Create, Show, List, Update, Doc, Task, Sync)
- `DocCommand` enum
- `TaskCommand` enum
- `TaskDetailCommand` enum
- `TicketDefinitionFilter` enum (since it's used by `TicketListArgs` which will be in `list.rs`)

All enum variants must be public (`pub(crate)`). Preserve all `#[derive]`, `#[command]`, `#[arg]`, and doc comment attributes exactly.

Verification: `cargo check -p tandem-cli` — the old `ticket.rs` still exists, so compilation should still work (new `ticket/mod.rs` is additional). No new warnings.
