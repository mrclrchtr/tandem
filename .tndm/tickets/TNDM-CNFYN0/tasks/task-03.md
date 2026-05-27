# Task 3: Refactor handle_ticket_create to use TicketUpdate

## Goal

Rewrite `handle_ticket_create` to use `TicketUpdate`, eliminating the manual guard and per-field assignments.

## Files

- `crates/tandem-cli/src/cli/ticket.rs`

## Changes

1. Change the function signature to accept `TicketCreateArgs` directly (extracted in task 1):

```rust
pub(crate) fn handle_ticket_create(args: TicketCreateArgs) -> anyhow::Result<()>
```

2. Build `TicketUpdate` from args and use it:

```rust
let update = TicketUpdate::from_create_args(&args);
let stdin_content = read_stdin_if_no_flags(update.is_empty())?;
```

3. Replace the manual `no_explicit_create` boolean with `update.is_empty()`.

4. Replace all per-field `if let Some(value) = ...` assignments with `update.apply(&mut ticket, ctx.id_prefix())?;`

5. Content handling stays in the function body: load content from file/stdin/inline/config template as before, call `ctx.store.create_ticket(NewTicket { meta, content })`.

6. Status handling: after creating the ticket, if status is Some, update separately (current behavior — status is applied after creation to get initial state right).

## Verification

- All existing `ticket_create_*` integration tests pass without modification
- `cargo test -p tandem-cli ticket_create` green
