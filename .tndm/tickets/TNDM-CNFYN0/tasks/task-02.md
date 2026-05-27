# Task 2: Create TicketUpdate struct with is_empty() and apply()

## Goal

Add a `TicketUpdate` struct and its `apply()` method to replace scattered `no_explicit_*` guards and per-field `if let Some(...)` blocks.

## Files

- `crates/tandem-cli/src/cli/ticket.rs`

## Changes

1. Define `TicketUpdate` struct near the top of `ticket.rs` (after use statements, before handler functions):

```rust
struct TicketUpdate {
    status: Option<TicketStatus>,
    priority: Option<TicketPriority>,
    title: Option<String>,
    ticket_type: Option<TicketType>,
    tags: Option<String>,
    add_tags: Option<String>,
    remove_tags: Option<String>,
    depends_on: Option<String>,
    effort: Option<TicketEffort>,
    content_file: Option<PathBuf>,
    content: Option<String>,
    stdin_content: Option<String>,
}
```

2. Implement two methods:

- `is_empty(&self) -> bool` — returns true when all 12 fields are None. This replaces both `no_explicit_create` and `no_explicit_update` guards.

- `apply(&self, ticket: &mut Ticket, id_prefix: &str) -> anyhow::Result<()>` — applies all non-None metadata fields to the ticket. Logic matches current per-field patterns:
  - `status` → `ticket.state.status = value`
  - `priority`, `title`, `ticket_type`, `effort` → set on `ticket.meta`
  - `tags` → `ticket.meta.tags = parse_tags(&value)`
  - `depends_on` → `ticket.meta.depends_on = parse_depends_on(&value, id_prefix)?`
  - `add_tags` → merge (dedup, sort)
  - `remove_tags` → filter out
  - Title empty check for update (return error if empty)

Does NOT handle content_file/content/stdin_content — those remain in the handler functions since create and update have different content semantics.

3. Add a `from_create_args` constructor (or `From` impl) that initializes from `TicketCreateArgs`. Add a `from_update_args` for `TicketUpdateArgs`.

## Verification

- `cargo check -p tandem-cli` compiles
- Unit test: create a `TicketUpdate` with a few fields set, verify `is_empty()` returns false; create one with all None, verify `is_empty()` returns true
- Unit test: apply status+priority to a ticket, verify fields changed and unchanged fields remain intact
