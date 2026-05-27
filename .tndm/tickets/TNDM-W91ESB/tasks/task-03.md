# Task 3: Extract handle_ticket_sync into ticket/sync.rs

Create `crates/tandem-cli/src/cli/ticket/sync.rs` containing `handle_ticket_sync(id, json)` extracted verbatim from `ticket.rs`.

Imports needed:
- `super::ticket_ctx::TicketCtx`
- `super::render::output_ticket_json`
- `tandem_core::ports::TicketStore`

Verification: `cargo check -p tandem-cli` (dead code warning expected).
