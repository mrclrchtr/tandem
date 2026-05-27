# Task 2: Extract handle_ticket_show into ticket/show.rs

Create `crates/tandem-cli/src/cli/ticket/show.rs` containing `handle_ticket_show(id, json)` extracted verbatim from `ticket.rs`.

Imports needed:
- `super::ticket_ctx::TicketCtx`
- `super::render::output_ticket_json` (or `print_ticket_human` — check current usage)
- `tandem_core::ports::TicketStore`

The function takes `id: String, json: bool`, creates a `TicketCtx`, resolves the ID, loads the ticket, and outputs either JSON or human-readable format.

Do not add `pub mod show;` to `ticket/mod.rs` yet — that comes in a later task when the old file is deleted.

Verification: `cargo check -p tandem-cli` (dead code warning is expected since the function isn't called yet).
