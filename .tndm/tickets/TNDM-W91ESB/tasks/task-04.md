# Task 4: Extract list handler into ticket/list.rs

Create `crates/tandem-cli/src/cli/ticket/list.rs` containing extracted from `ticket.rs`:

- `TicketListArgs` struct with its `#[derive(Args)]` and clap attributes
- `handle_ticket_list(args: TicketListArgs)` function
- `ticket_matches_definition_filter(ticket, filter)` helper

The `TicketDefinitionFilter` enum lives in `ticket/mod.rs`, so import it from `super::TicketDefinitionFilter`.

Imports needed:
- `super::TicketDefinitionFilter`
- `super::ticket_ctx::TicketCtx`
- `super::render::{TicketJsonEntry, TicketListJson, output_ticket_json}`
- `super::util::{DEFINITION_TAG_QUESTIONS, DEFINITION_TAG_READY, ticket_content_path}`
- `tabled::{builder::Builder, settings::Style}`
- `tandem_core::ticket::{TicketStatus, Ticket}`
- `tandem_core::ports::TicketStore`

Verification: `cargo check -p tandem-cli` (dead code warnings expected).
