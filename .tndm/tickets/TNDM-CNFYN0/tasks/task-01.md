# Task 1: Extract named clap structs for Create, Update, and List subcommands

## Goal

Convert inline clap subcommand variants in `TicketCommand` into named `pub(crate)` structs so `mod.rs` can pass them directly.

## Files

- `crates/tandem-cli/src/cli/ticket.rs`

## Changes

1. Extract `TicketCreateArgs` struct from `TicketCommand::Create { ... }` fields. Keep all clap attributes (`#[arg(long)]`, conflicts, etc.). Make it `#[derive(Args, Debug)]` and `pub(crate)`.

2. Extract `TicketUpdateArgs` struct from `TicketCommand::Update { ... }` fields. Include `add_tags`/`remove_tags` with their `conflicts_with = "tags"` constraints.

3. Extract `TicketListArgs` struct from `TicketCommand::List { all, definition, output }` fields. Make it `#[derive(Args, Debug)]` and `pub(crate)`.

4. Update the enum variants to use the new structs:

```rust
Create(TicketCreateArgs),
Update(TicketUpdateArgs),
List(TicketListArgs),
```

5. Update `mod.rs` to match the new variant shapes (just structural, field access changes from `Create { title, ... }` to `Create(args)` — keep the destructuring for now, we'll simplify in task 5).

## Verification

- `cargo check -p tandem-cli` compiles
- `./tndm-dev ticket create --help` shows unchanged help output
- `./tndm-dev ticket update --help` shows unchanged help output with add-tags/remove-tags conflicts intact
