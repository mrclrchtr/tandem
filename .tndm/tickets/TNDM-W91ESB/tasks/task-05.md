# Task 5: Extract update handler + TicketUpdate into ticket/update.rs

Create `crates/tandem-cli/src/cli/ticket/update.rs` containing extracted from `ticket.rs`:

- `TicketUpdateArgs` struct with all clap attributes
- `TicketUpdate` struct with all fields, `is_empty()`, `apply()`, `from_create_args()`, `from_update_args()`
- `handle_ticket_update(args: TicketUpdateArgs)` function
- All existing unit tests (`make_ticket`, `ticket_update_is_empty_*`, `ticket_update_apply_*`, `ticket_update_from_*_args_*`)

Make `TicketUpdate` and its methods `pub(crate)` — `create.rs` will import from here.

Imports needed:
- Standard: `std::{fs, path::{Path, PathBuf}}`
- `super::ticket_ctx::TicketCtx`
- `super::render::{TicketJsonEntry, output_ticket_json}`
- `super::util::{parse_tags, parse_depends_on, read_stdin_if_no_flags, load_ticket_content}`
- `tandem_core::{ports::TicketStore, ticket::*}` for all ticket types
- `time::{OffsetDateTime, format_description::well_known::Rfc3339}`

Verification: `cargo test -p tandem-cli -- ticket_update` — all moved tests should pass.
