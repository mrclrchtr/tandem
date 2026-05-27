# Task 8: Wire cli/mod.rs dispatches and delete old ticket.rs

In `crates/tandem-cli/src/cli/mod.rs`:

1. Replace `mod ticket;` with directory-based module declaration:
   - Keep `mod ticket;` (Rust will resolve to `ticket/mod.rs`)
   - In `ticket/mod.rs`, add `pub(crate) mod create; pub(crate) mod list; pub(crate) mod show; pub(crate) mod sync; pub(crate) mod update; pub(crate) mod task;`

2. Update all dispatch calls in the `Command::Ticket { command } => match command { ... }` block:
   - `ticket::handle_ticket_create(args)` → `ticket::create::handle_ticket_create(args)`
   - `ticket::handle_ticket_show(id, output.json)` → `ticket::show::handle_ticket_show(id, output.json)`
   - `ticket::handle_ticket_list(args)` → `ticket::list::handle_ticket_list(args)`
   - `ticket::handle_ticket_update(args)` → `ticket::update::handle_ticket_update(args)`
   - `ticket::handle_ticket_sync(id, output.json)` → `ticket::sync::handle_ticket_sync(id, output.json)`
   - `ticket::handle_task_add(id, title, output.json)` → `ticket::task::handle_task_add(id, title, output.json)`
   - `ticket::handle_task_list(id, output.json)` → `ticket::task::handle_task_list(id, output.json)`
   - `ticket::handle_task_complete(id, number, output.json)` → `ticket::task::handle_task_complete(id, number, output.json)`
   - `ticket::handle_task_remove(id, number, output.json)` → `ticket::task::handle_task_remove(id, number, output.json)`
   - `ticket::handle_task_edit(id, number, title, output.json)` → `ticket::task::handle_task_edit(id, number, title, output.json)`
   - `ticket::handle_task_detail_ensure(id, number, output.json)` → `ticket::task::handle_task_detail_ensure(id, number, output.json)`
   - `ticket::handle_task_set(id, tasks, output.json)` → `ticket::task::handle_task_set(id, tasks, output.json)`

3. Delete `crates/tandem-cli/src/cli/ticket.rs`.

4. Remove `pub mod ticket;` from `ticket/mod.rs` (since ticket.rs no longer exists).

Verification: `cargo build -p tandem-cli` must succeed with no errors.
