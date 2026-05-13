# Design Decisions

Architectural and design decisions for tandem. See `docs/vision.md` for product goals.

## Product scope

- The core product is a coordination layer, not just a passive planning format.
- The core feature is a ticket system for AI agents.
- The main requirement is that agents can tell what other agents or parallel branches/worktrees are doing.
- Exploration, conversation forks, and alternative implementations are relevant, but they are not the core feature.

## Architecture direction

- The system is git-aware.
- It is designed to work inside a monorepo.
- It must support git worktrees.
- It detects that changes happened on other branches/worktrees, including remote changes.
- It does not depend on a central web service as the main architecture.
- Repo-local ticket files are the system of record. Optional adapters may integrate with external issue trackers
  (GitHub/Linear/etc.), but those are not required for core operation.

## Storage model

- Ticket state is stored in the repository.
- Storage model: one directory per ticket.
- Structure:
    - `.tndm/tickets/<TICKET-ID>/meta.toml`
    - `.tndm/tickets/<TICKET-ID>/state.toml`
    - `.tndm/tickets/<TICKET-ID>/content.md`
- The split between stable metadata (`meta.toml`) and volatile state (`state.toml`) reduces Git friction.
- Repository-wide configuration lives in `.tndm/config.toml` (optional). It controls:
    - `[id].prefix` — prefix for generated ticket IDs (default: `TNDM`)
    - `[templates].content` — default markdown template for new ticket content

## File format + determinism

- Ticket metadata and state are stored as TOML files.
- The CLI is the canonical writer/formatter for these files.
- `tndm fmt` and `tndm fmt --check` enforce stable serialization (ordering, whitespace, encoding, timestamp
  representation) and minimize churn in diffs.
- Freeform text belongs in `content.md`, not in the TOML files.

## Ticket model

The ticket model is strictly validated. Fields are split across two files:

**`meta.toml`** (stable metadata):

| Field | Type | Notes |
|-------|------|-------|
| `schema_version` | integer | Format version for forward compatibility |
| `id` | string | Validated ticket identifier (e.g. `TNDM-A1B2C3`) |
| `title` | string | Human/agent-readable summary |
| `type` | enum | `task` (default), `bug`, `feature`, `chore`, `epic` |
| `priority` | enum | `p0`, `p1`, `p2` (default), `p3`, `p4` |
| `depends_on` | string array | Ticket IDs this ticket depends on |
| `tags` | string array | Freeform labels, including reserved coordination tags such as `definition:ready` and `definition:questions` |

**`state.toml`** (volatile state):

| Field | Type | Notes |
|-------|------|-------|
| `schema_version` | integer | Format version for forward compatibility |
| `status` | enum | `todo` (default), `in_progress`, `blocked`, `done` |
| `updated_at` | string | RFC 3339 timestamp, tool-managed |
| `revision` | integer | Monotonic counter, incremented on each state change |

All enums parse case-insensitively for CLI friendliness. There is no assignee field.

The system uses a lightweight convention for current ticket-definition quality:

- `definition:questions` means the ticket still has open definition questions.
- `definition:ready` means the ticket is currently considered implementable.
- Absence of a `definition:*` tag means definition state is still unknown or unreviewed.

These tags describe current coordination state only. Historical refinement remains in Git history and ticket content,
not in dedicated mutable counters.

`updated_at` is load-bearing for freshness, awareness, and change comparison. Because wall clocks can skew across
machines/worktrees, the system avoids relying on `updated_at` as the only ordering signal. The `revision` field
provides a monotonic, tool-derived counter for unambiguous ordering within a single ticket.

The default `content.md` template is intentionally structured around:

- `Context`
- `Goal`
- `Open Questions`
- `Acceptance`
- `Ready When`

This keeps rich ticket nuance in markdown while allowing agents to use reserved tags as coarse machine-readable
signals.

### Document registry

Starting in 0.6.0, tickets use a **document registry** in metadata.

- `meta.toml` contains a `[[documents]]` table listing registered document files by name and path.
- `state.toml` contains `[document_fingerprints]` with SHA-256 hashes for freshness verification.
- `content.md` is the default registered document (implicitly present on creation).
- Additional documents (e.g. `plan.md`) are created via `tndm ticket doc create <ID> <name>`.
- Files inside `.tndm/tickets/<ID>/` are owned by TNDM — agents should not create files there directly.
- After editing a registered document file, run `tndm ticket sync <ID>` to update fingerprints.
- `tndm fmt --check` fails if registered documents have stale fingerprints.

## Awareness model

- V1 awareness is a command-bound function invoked via `tndm awareness --against <ref>`.
- The baseline behavior is deterministic:
    - materialize ticket state at the given git ref
    - compare it against the current working tree
    - expose changes in structured JSON
- Awareness output distinguishes:
    - **added_current:** ticket exists in the working tree but not at the compared ref
    - **added_against:** ticket exists at the compared ref but not in the working tree
    - **diverged:** ticket exists in both but fields differ (with field-level diffs for status, priority, title, type, depends_on, tags)
- Awareness may surface local, uncommitted ticket changes as early hints, but distinguishes them from changes observed
  on Git refs (since uncommitted state is machine-local and non-reproducible).

## Machine-readable output

- All commands should support structured JSON output for agent consumption.
- JSON is the standard machine-readable format across the CLI.
- The awareness command outputs a schema-versioned JSON report (`AwarenessReport`).
- Other commands (e.g. `ticket list`, `ticket show`) should follow the same pattern for agent interoperability.
- `ticket list` may expose lightweight workflow filters backed by existing metadata, such as tag-backed definition
  state, without expanding the core schema.

## Branch/worktree information in tickets

- Tickets may carry light branch/worktree-related metadata.
- Branch/worktree context should be present where useful, but there is no decision to make branch-specific attempts the
  main domain model.

## Interfaces

- The product supports AI-first usage and is also usable by humans for oversight.
- The interface direction is:
    - deterministic CLI for agents and humans
    - machine-readable JSON output for agents
    - possible future adapter layers if needed
- Name: **tandem**
- CLI command: **`tndm`**

## LLM integration

- LLM integration is not part of the required core.
- Optional direction: a plugin that summarizes structured changes for the requesting agent.
- The deterministic ticket/change model must exist independently of any LLM component.

## Testing infrastructure

- Shared test utilities use feature-gated exports (`#[cfg(feature = "test-support")]`)
  from the owning crate (typically `tandem-core`), not a separate workspace
  `-test-support` crate. This keeps the dependency graph flat and follows
  standard Rust idiom.

## Things intentionally not over-fixed yet

- Final status state machine (transitions between statuses)
- Exact API surface beyond the CLI
