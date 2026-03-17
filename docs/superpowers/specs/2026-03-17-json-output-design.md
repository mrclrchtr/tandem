# JSON Output for All CLI Commands

## Problem

tandem's decisions doc states that all commands should support structured JSON output for agent consumption. Currently only `awareness` outputs JSON (and it does so unconditionally). The ticket commands (`create`, `show`, `list`, `update`) output plain text only. Agents cannot programmatically consume ticket data without parsing ad-hoc text formats.

Additionally, `awareness` always outputs JSON with no human-readable option, which is inconsistent with the rest of the CLI.

## Design

### `--json` flag via shared `OutputArgs`

A reusable `OutputArgs` struct is flattened into every command that supports output formatting:

```rust
#[derive(Args, Debug)]
struct OutputArgs {
    /// Output as JSON instead of human-readable text.
    #[arg(long)]
    json: bool,
}
```

Flattened into: `Create`, `Show`, `List`, `Update`, `AwarenessArgs`.

Not added to `Fmt` — it is a formatting tool, not a data query.

Default behavior for all commands is human-readable text. `--json` opts into structured JSON.

### Serialize on domain types

Add `#[derive(Serialize)]` to domain types in `tandem-core`:

| Type | Serialization |
|------|---------------|
| `TicketId` | Plain string (custom `Serialize` impl delegating to inner `String`) |
| `TicketType` | Custom `Serialize` delegating to `as_str()` (produces `"task"`, `"bug"`, etc.) |
| `TicketPriority` | Custom `Serialize` delegating to `as_str()` (produces `"p0"`–`"p4"`) |
| `TicketStatus` | Custom `Serialize` delegating to `as_str()` (produces `"todo"`, `"in_progress"`, etc.) |
| `TicketMeta` | Derives `Serialize`; `ticket_type` renamed to `"type"` via `#[serde(rename = "type")]` |
| `TicketState` | Derives `Serialize` |

`Ticket` does not need `Serialize` directly — the CLI uses an envelope struct (see below).

No new dependencies. `serde` with `derive` feature is already a workspace dependency used by the awareness types.

### JSON output shape

All JSON output uses a consistent envelope with `schema_version`. This `schema_version` tracks the JSON API shape and is independent from the `schema_version` in the TOML files (`meta.toml`, `state.toml`).

Content is never embedded in JSON — a `content_path` is provided instead. The path is relative to the repo root and constructed deterministically from the ticket ID: `.tndm/tickets/{id}/content.md`.

The JSON envelope flattens `TicketMeta` and `TicketState` fields into a single object. The meta/state file split is a storage concern and not exposed in the JSON API.

**Single ticket** (used by `show`, `create`, `update`):

```json
{
  "schema_version": 1,
  "id": "TNDM-ABC123",
  "title": "Refactor auth module",
  "type": "task",
  "priority": "p2",
  "status": "todo",
  "updated_at": "2026-03-17T13:32:22Z",
  "revision": 1,
  "depends_on": [],
  "tags": [],
  "content_path": ".tndm/tickets/TNDM-ABC123/content.md"
}
```

**Ticket list** (used by `list`):

```json
{
  "schema_version": 1,
  "tickets": [
    {
      "id": "TNDM-1",
      "title": "...",
      "type": "task",
      "priority": "p2",
      "status": "todo",
      "updated_at": "...",
      "revision": 1,
      "depends_on": [],
      "tags": [],
      "content_path": ".tndm/tickets/TNDM-1/content.md"
    }
  ]
}
```

**Awareness** (used by `awareness --json`): unchanged from current output shape (includes `schema_version`, `against`, and `tickets` array with change kinds and field diffs).

**Zero-item cases**: `ticket list --json` with no tickets produces `{"schema_version": 1, "tickets": []}`. Awareness with no changes produces `{"schema_version": 1, "against": "...", "tickets": []}`.

**`ticket create --json`**: the `create_ticket` port returns the created `Ticket`. The handler uses this return value to build the JSON envelope — no reload needed.

The single-ticket and list envelope structs live in the CLI crate, not in `tandem-core`. They reference the domain types for serialization and add `schema_version` and `content_path`.

### Error output

Errors remain unstructured on stderr. The exit code is the signal for failure. No JSON error envelope is introduced — this matches standard CLI conventions and avoids complicating the implementation. Agents should check the exit code, not parse stderr.

### Breaking change: awareness default output

Changing awareness from always-JSON to default-text is an intentional breaking change. Any consumer currently parsing awareness output without `--json` will break. This is acceptable because the project is pre-1.0 and consistency across all commands (text default, `--json` opt-in) outweighs backwards compatibility at this stage.

### Text output for awareness

With this change, the awareness default becomes human-readable:

```
Against: main

TNDM-ABC123  added (current)
TNDM-DEF456  added (against)
TNDM-GHI789  diverged
  status:     in_progress -> todo
  priority:   p1 -> p2
  depends_on: [TNDM-1] -> []
```

- One line per changed ticket with the change kind
- Diverged tickets get indented field diffs below
- Empty report: `Against: main\n\nNo changes.`

### Text output for existing commands

Unchanged:

- `ticket create` — prints ticket ID
- `ticket update` — prints ticket ID
- `ticket show` — three-section format (`## meta.toml`, `## state.toml`, `## content.md`)
- `ticket list` — tab-separated `ID\tstatus\ttitle`

## Changes by crate

| Crate | Change |
|---|---|
| `tandem-core` | Add `#[derive(Serialize)]` + serde attrs to `TicketId`, `TicketType`, `TicketPriority`, `TicketStatus`, `TicketMeta`, `TicketState`. |
| `tandem-cli` | Add `OutputArgs` struct. Flatten into subcommands. Add CLI-level envelope structs for JSON. Add text renderer for awareness. Branch handlers on `output.json`. Update tests. |
| `tandem-storage` | No changes. |
| `tandem-repo` | No changes. |

## Test plan

### Existing tests (unchanged behavior)

- `ticket_create_prints_generated_id_and_writes_ticket_files` — text output unchanged
- `ticket_list_prints_sorted_tab_separated_lines` — text output unchanged
- `ticket_show_prints_exact_canonical_sections` — text output unchanged

### Existing tests (need update)

- Awareness integration tests that assert JSON output without `--json` must add `--json` to the command invocation. This minimizes test churn — the JSON shape is unchanged, only the opt-in mechanism changes.

### New tests

- `--json` produces valid JSON for each command (create, show, list, update)
- `--json` on awareness produces same JSON shape as before
- JSON contains `content_path`, not `content`
- `content_path` is relative to repo root
- Awareness text output matches human-readable format
- Awareness text output for empty report shows "No changes."
- Awareness text output for diverged tickets shows field diffs
