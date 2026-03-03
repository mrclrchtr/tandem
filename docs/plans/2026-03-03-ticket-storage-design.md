# Ticket storage + create/show/list (V1) — design

Date: 2026-03-03

## Summary

This document specifies the V1 on-disk ticket storage model and the CLI behavior for:

- `tndm ticket create`
- `tndm ticket show`
- `tndm ticket list`

The goals are:

- git-friendly diffs and reduced merge friction (split stable metadata vs volatile state)
- strict validation and strong Rust schema tooling for structured data
- deterministic formatting via `tndm fmt` (canonical writer)
- freeform ticket text in Markdown

Non-goals for this V1 slice:

- git/worktree “awareness” implementation
- dependency graph commands (`ready`, `blocked`, `tree`, cycle detection)
- shared sequential ID registry / allocator (optional later addon)

## Storage model

### Root directory

All tandem data lives under:

- `.tndm/`

### Per-ticket directory

Each ticket is a directory named by its ticket ID:

```
.tndm/
  config.toml
  tickets/
    <TICKET_ID>/
      meta.toml
      state.toml
      content.md
```

Rationale:

- Avoids repeating the ticket ID in filenames when multiple artifacts exist.
- Keeps paths predictable (like `Cargo.toml` inside a crate directory).

### Structured file format

- Structured files use TOML (single repo-wide format).
- `tndm fmt` is the canonical writer: it rewrites structured files into a stable, deterministic format.
- Comments/formatting in structured TOML are not preserved.

### Freeform content

- `content.md` is freeform Markdown.
- V1 must not derive semantics from `content.md`.
- `tndm fmt` does not rewrite `content.md`.

## Ticket ID strategy

### Generated IDs (default)

V1 generates IDs without shared state (offline, worktree-safe):

- Format: `<PREFIX>-<SUFFIX>`
- Default prefix: `TNDM` (repo-configurable)
- Suffix: 6 characters (Crockford Base32, uppercase)
  - Alphabet: `0123456789ABCDEFGHJKMNPQRSTVWXYZ`

Example:

- `TNDM-4K7D9Q`

Collision handling:

- If `.tndm/tickets/<id>/` already exists, re-roll suffix.

### Supplied IDs (supported)

The CLI also supports creating a ticket with an explicit ID:

- `tndm ticket create --id PROJ-123 --title "..."`

The ID is validated and must not already exist.

### Future addon (optional)

A later, opt-in sequential allocator may be implemented via a dedicated git ref/branch registry, but it is not required for V1.

## Repo config (.tndm/config.toml)

A repo-local config file controls defaults shared by the whole team.

Proposed shape:

```toml
schema_version = 1

[id]
prefix = "TNDM"

[templates]
content = '''
## Description

## Design

## Acceptance

## Notes
'''
```

Notes:

- `templates.content` is treated as literal text (no placeholder interpolation) in V1.

## CLI behavior

### `tndm ticket create`

Creates the per-ticket directory and its three files.

Content sources for `content.md` (precedence order):

1. `--content-file <path>`: copy that file verbatim to `content.md`
2. piped stdin (stdin is not a TTY): read stdin verbatim to `content.md`
3. `.tndm/config.toml` `templates.content`
4. built-in fallback template (same shape as above)

If (1) or (2) is used, do not apply the template.

### `tndm ticket show`

Reads a single ticket by ID and displays:

- core metadata from `meta.toml`
- current state from `state.toml`
- optionally, the Markdown content (human-friendly view)

Design intent:

- Provide a human-friendly default output.
- Provide a deterministic JSON output mode later (`--json`) for agents and scripting.

### `tndm ticket list`

Lists tickets by scanning `.tndm/tickets/*/`.

Design intent:

- Provide a stable default ordering (V1 can sort by ticket ID string).
- Provide a JSON output mode later (`--json`).

## Schema: meta.toml + state.toml (V1 draft)

Exact field lists are intentionally minimal and may evolve; the core requirement is strict validation.

### `meta.toml`

- `schema_version = 1`
- `id = "TNDM-..."` (required, must match directory name)
- `title = "..."` (required)
- `type = "task" | "bug" | "feature" | "chore" | "epic"` (default `task`)
- `priority = "p0".."p4"` (default `p2`)
- `depends_on = ["TNDM-...", ...]` (default `[]`, canonicalized sort+dedupe)
- `tags = ["...", ...]` (default `[]`, canonicalized sort+dedupe)

### `state.toml`

- `schema_version = 1`
- `status = "todo" | "in_progress" | "blocked" | "done"` (default `todo`)
- `updated_at = "<RFC3339 UTC>"` (tool-managed; set on create)
- `revision = <int>` (tool-managed; start at 1)

## Determinism & formatting

`tndm fmt` (and any writer paths used by `tndm ticket create`/updates) must enforce:

- stable key ordering and whitespace
- consistent quoting rules
- consistent array formatting
- stable timestamp representation (RFC3339 UTC, e.g. `2026-03-03T12:34:56Z`)

Structured TOML files are rewritten; `content.md` is not.

## Open questions (tracked for later)

- JSON output schemas (`--json`) for `show` and `list`
- whether to add derived views (`ready`, `blocked`) in V1.1
- semantic conflict categories and awareness integration
