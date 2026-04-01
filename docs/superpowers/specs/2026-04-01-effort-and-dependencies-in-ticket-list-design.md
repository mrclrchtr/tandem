# Design: Effort and Dependencies in Ticket List

**Date:** 2026-04-01
**Status:** Approved

## Summary

Add an `effort` field (T-shirt size) to the ticket data model and surface both `effort` and `depends_on` in `tndm ticket list` and related commands, consistent with how existing fields are handled.

## Data Model

### New type: `TicketEffort`

Add to `crates/tandem-core/src/ticket.rs`:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TicketEffort {
    Xs,
    S,
    M,
    L,
    Xl,
}
```

Implement `Display` to render lowercase: `xs`, `s`, `m`, `l`, `xl`.

### Updated `TicketMeta`

Add one field:

```rust
pub effort: Option<TicketEffort>,
```

- `None` serializes as absent from TOML (not `effort = null`), consistent with how empty `Vec`s behave.
- Valid TOML values: `"xs"`, `"s"`, `"m"`, `"l"`, `"xl"`.
- `depends_on` already exists as `Vec<TicketId>` — no data model change needed for it.

## CLI Changes

### `ticket list` plain text

Add two columns between `PRIORITY` and `TITLE`. Column order becomes:

```
ID  STATUS  PRIORITY  EFFORT  DEPS  TITLE
```

- `EFFORT`: lowercase size string, or `-` when `None`.
- `DEPS`: comma-separated `TicketId` values, or empty string when `depends_on` is empty.
- Column separator remains `\t`.

Example output:

```
TNDM-A1B2C3	todo	p1	m	TNDM-X1, TNDM-Y2	Fix login timeout
TNDM-D4E5F6	in_progress	p2	-		Refactor auth module
```

### `ticket create` and `ticket update`

Add flag: `-e, --effort <SIZE>` accepting `xs|s|m|l|xl`. Omitting the flag leaves effort unset (create) or unchanged (update). Plain text output (just the ticket ID) is unchanged. `--depends-on` already exists.

### `ticket show`

No code change required. `effort` appears in the `meta.toml` section automatically once the field is stored.

### JSON output

Add `effort` to the ticket envelope as a nullable string (`null` when unset). `depends_on` is already present.

### Plugin command reference

Update `plugins/tndm/skills/ticket/references/command-reference.md` to document `-e, --effort <SIZE>` for `ticket create` and `ticket update`.

## Testing

### `tandem-core` unit tests

- `TicketEffort` serialization round-trip: `"m"` ↔ `TicketEffort::M`.
- `TicketEffort` `Display` renders lowercase for all variants.
- `TicketMeta` with `effort = None` round-trips without an `effort` key in TOML.
- `TicketMeta` with `effort = Some(TicketEffort::L)` round-trips with `effort = "l"`.

### `tandem-cli` integration tests

- `ticket create --effort m` stores `effort = "m"` in `meta.toml`.
- `ticket list` plain text output has 6 tab-separated columns in the correct order.
- `ticket list --json` includes `"effort"` field in each ticket object.
- `ticket update --effort xl` updates the stored value.
- Unset effort renders as `-` in list plain text output and is absent from TOML.
- `depends_on` IDs appear comma-separated in the `DEPS` column; empty when none.

## Out of Scope

- Filtering or sorting by effort in `ticket list`.
- Effort validation beyond enum membership (e.g., no ordering enforcement).
- Changes to `ticket type` or other existing fields.
