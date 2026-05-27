# Task 6: Extract create handler into ticket/create.rs

Create `crates/tandem-cli/src/cli/ticket/create.rs` containing extracted from `ticket.rs`:

- `TicketCreateArgs` struct with all clap attributes
- `handle_ticket_create(args: TicketCreateArgs)` function

Imports needed:
- Standard: `std::path::PathBuf`
- `super::ticket_ctx::TicketCtx`
- `super::update::TicketUpdate` — cross-module dependency for `from_create_args`
- `super::render::output_ticket_json`
- `super::util::{generate_ticket_id, load_ticket_content, read_stdin_if_no_flags}`
- `tandem_core::{ports::TicketStore, ticket::*}`
- `time::{OffsetDateTime, format_description::well_known::Rfc3339}`

Verification: `cargo check -p tandem-cli` — should compile with the cross-module import.
