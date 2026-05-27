# Task 7: Extract task handlers + helpers into ticket/task.rs

Create `crates/tandem-cli/src/cli/ticket/task.rs` containing extracted from `ticket.rs`:

**Handlers:**
- `handle_task_add(id, title, json)`
- `handle_task_list(id, json)`
- `handle_task_complete(id, number, json)`
- `handle_task_remove(id, number, json)`
- `handle_task_edit(id, number, title, json)`
- `handle_task_detail_ensure(id, number, json)`
- `handle_task_set(id, tasks_json, json)`

**Internal helpers (pub(crate) or private):**
- `load_and_bump(store, ticket_id)`
- `persist_and_output(store, ticket, json)`
- `find_task(tasks, number)`
- `canonical_task_detail_doc(number)`
- `is_canonical_task_detail_doc(doc)`
- `prune_unlinked_canonical_task_detail_docs(repo_root, ticket_id, ticket)`
- `ensure_canonical_task_detail_doc(repo_root, ticket_id, ticket, task_number, title)`

Imports needed:
- Standard: `std::{fs, path::Path, collections::BTreeSet}`
- `super::super::doc::recompute_ticket_document_fingerprints` (note: two levels up since task/ is under ticket/)
- `super::ticket_ctx::TicketCtx`
- `super::render::output_ticket_json`
- `tandem_core::{ports::TicketStore, ticket::*}`
- `tandem_storage::{FileTicketStore, ticket_dir}`
- `time::{OffsetDateTime, format_description::well_known::Rfc3339}`

All functions should be `pub(crate)` for now.

Verification: `cargo check -p tandem-cli` — should compile. Dead code warnings expected.
