# Task 4: Refactor handle_ticket_update to use TicketUpdate

## Goal

Rewrite `handle_ticket_update` to use `TicketUpdate`, eliminating the manual guards and per-field assignments.

## Files

- `crates/tandem-cli/src/cli/ticket.rs`

## Changes

1. Change function signature to accept `TicketUpdateArgs` directly:

```rust
pub(crate) fn handle_ticket_update(args: TicketUpdateArgs) -> anyhow::Result<()>
```

2. Build `TicketUpdate` and use it for stdin guard:

```rust
let update = TicketUpdate::from_update_args(&args);
let stdin_content = read_stdin_if_no_flags(update.is_empty())?;
```

3. Replace the two guards (`no_explicit_update` + "at least one flag required"):

```rust
if update.is_empty() && args.content_file.is_none() && args.content.is_none() && stdin_content.is_none() {
    anyhow::bail!("at least one update flag is required...");
}
```

4. Replace all per-field `if let Some(value) = ...` with `update.apply(&mut ticket, ctx.id_prefix())?;`

5. Content handling stays separate: file/inline/stdin content replaces ticket.content as before.

6. Revision bump and `update_ticket` call stay the same.

## Verification

- All existing `ticket_update_*` integration tests pass without modification
- `cargo test -p tandem-cli ticket_update` green
- Manual test: `./tndm-dev ticket update TNDM-XXXXXX --status done` works, `./tndm-dev ticket update TNDM-XXXXXX` (no flags) returns error
