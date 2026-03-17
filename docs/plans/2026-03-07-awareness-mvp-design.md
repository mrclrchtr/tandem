# Awareness MVP — design

Date: 2026-03-07

## Summary

This document defines a minimal V1 awareness MVP for `tndm`.

The MVP adds a single awareness command that compares the current checkout’s ticket state against one target Git ref and emits a deterministic JSON summary of all ticket differences.

Goals:

- deliver real awareness value without requiring full multi-worktree aggregation
- keep output deterministic and agent-friendly
- preserve crate boundaries from `docs/architecture.md`
- defer semantic conflict policy until after the snapshot comparison model is proven

Non-goals for this MVP:

- multi-worktree aggregation
- merge-base-aware directionality
- semantic conflict categories
- content diffing for `content.md`
- human-readable default rendering

## Command

The MVP command is:

```sh
tndm awareness --against <ref>
```

Examples:

```sh
tndm awareness --against main
tndm awareness --against origin/main
```

Behavior:

- load all tickets from the current checkout
- load all tickets from the specified Git ref
- compare the two snapshots
- print a deterministic JSON document to stdout

Default output is JSON only.

## Output contract

The initial JSON shape is:

```json
{
  "schema_version": 1,
  "against": "main",
  "tickets": [
    {
      "id": "TNDM-AAAAAA",
      "change": "diverged",
      "fields": {
        "status": {
          "current": "in_progress",
          "against": "todo"
        },
        "priority": {
          "current": "p1",
          "against": "p2"
        }
      }
    }
  ]
}
```

Determinism rules:

- `tickets` are sorted by ticket ID
- field diffs are emitted in fixed order
- unchanged fields are omitted
- default output contains no human prose

## Change kinds

The MVP uses a deliberately small change model:

- `added_current` — ticket exists only in the current checkout
- `added_against` — ticket exists only in the target ref
- `diverged` — ticket exists in both snapshots and one or more tracked fields differ

The design intentionally does not include `changed_current` or `changed_against` yet. Those labels imply directional certainty that the MVP does not have without merge-base-aware comparison logic.

## Tracked fields

The MVP compares only these structured fields:

- `status`
- `priority`
- `depends_on`

For `diverged` tickets, the report includes only fields whose values differ.

Example:

```json
{
  "id": "TNDM-AAAAAA",
  "change": "diverged",
  "fields": {
    "status": {
      "current": "in_progress",
      "against": "todo"
    },
    "depends_on": {
      "current": ["TNDM-000001"],
      "against": []
    }
  }
}
```

`content.md` is excluded from the MVP to avoid premature text-diff design and noisy reports.

## Architecture

The MVP follows the existing workspace boundaries:

- `tandem-core`
  - awareness domain types
  - pure snapshot comparison logic
  - no filesystem or Git access
- `tandem-storage`
  - ticket parsing/loading from a filesystem root
  - may expose a helper for loading all tickets under a root
- `tandem-repo`
  - resolve `--against <ref>`
  - obtain ticket snapshots for the current checkout and the target ref
  - normalize snapshot data for comparison
- `tandem-cli`
  - parse CLI arguments
  - invoke repo/core logic
  - render deterministic JSON

This keeps Git-specific behavior in `tandem-repo`, storage parsing in `tandem-storage`, and pure comparison policy in `tandem-core`.

## Snapshot model

Conceptually, each side is normalized to:

- `TicketSnapshot { tickets: BTreeMap<TicketId, Ticket> }`

Using a map keyed by `TicketId` makes deterministic comparison straightforward and keeps output ordering stable.

## Comparison rules

For each ticket ID in the union of both snapshots:

- if only present in current: `added_current`
- if only present in target ref: `added_against`
- if present in both and tracked fields differ: `diverged`
- if present in both and tracked fields match: omit from report

Normalization rules:

- compare canonicalized `depends_on` arrays
- sort ticket IDs before emitting results
- emit diff fields in a fixed order: `status`, `priority`, `depends_on`

## Error handling

The MVP should fail clearly for:

- unresolved target refs
- unreadable or invalid ticket files in either snapshot
- Git access failures

The initial policy is fail-fast: if either snapshot cannot be loaded correctly, the command returns an error rather than emitting partial results.

## Testing strategy

### Core tests

Add pure comparison tests for:

- identical snapshots produce an empty ticket list
- current-only ticket becomes `added_current`
- target-only ticket becomes `added_against`
- differing `status`, `priority`, or `depends_on` becomes `diverged`
- output ordering is stable by ticket ID and field order

### Repo adapter tests

Add tests for:

- loading current-checkout ticket snapshots
- loading target-ref ticket snapshots
- missing `.tndm/tickets` on one side
- invalid or missing target refs

### CLI tests

Add tests for:

- `tndm awareness --against <ref>` emits valid JSON
- empty diff returns `"tickets": []`
- changed tickets serialize in sorted order
- invalid refs fail with a clear error

## Implementation sequence

1. Add awareness domain/report types and comparison logic in `tandem-core`.
2. Add snapshot-loading APIs in `tandem-repo` for current checkout and target ref.
3. Reuse `tandem-storage` loading/parsing to build snapshots.
4. Implement `tndm awareness --against <ref>` in `tandem-cli`.
5. Add tests across core, repo, and CLI.
6. Run full workspace verification.

## Follow-on work after MVP

After this MVP is proven, the next expansions should be considered in this order:

1. merge-base-aware directional labels such as `changed_current` / `changed_against`
2. semantic conflict categories
3. human-readable output mode
4. multi-worktree aggregation
5. optional content diff awareness
