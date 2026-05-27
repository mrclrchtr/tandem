# Task 5: Simplify mod.rs dispatch: pass structs instead of destructuring fields

## Goal

Replace field-by-field destructuring in `mod.rs` dispatch with direct struct passing to handlers.

## Files

- `crates/tandem-cli/src/cli/mod.rs`

## Changes

1. Update `Command::Ticket { command } => match command { ... }` to pass structs instead of individual fields.

Before:
```rust
ticket::TicketCommand::Create {
    title, id, content_file, content, status, priority,
    ticket_type, tags, depends_on, effort, output,
} => ticket::handle_ticket_create(
    title, id, content_file, content, status, priority,
    ticket_type, tags, depends_on, effort, output.json,
),
```

After:
```rust
ticket::TicketCommand::Create(args) => ticket::handle_ticket_create(args),
```

2. Apply same pattern to `Update`, `List`.

3. For `Show`, `Sync`, `Doc`, and `Task` subcommands, keep inline destructuring since those have simple parameter lists already.

4. Update imports in `mod.rs` to import the new arg structs if needed.

## Verification

- `cargo check -p tandem-cli` compiles
- `cargo clippy -p tandem-cli` no new warnings
- `./tndm-dev ticket create "test" --priority p1 --tags auth` works correctly
- `./tndm-dev ticket update TNDM-XXX --status done` works correctly
