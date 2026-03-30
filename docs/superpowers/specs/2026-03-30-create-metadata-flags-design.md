# Add metadata flags to `tndm ticket create`

Date: 2026-03-30

## Problem

Setting priority, type, tags, or dependencies at creation time requires two commands
(`create` + `update`) and two commits. This is friction for agents and humans alike,
and the missing flags on `create` are a natural assumption — the error in the motivating
example was using `--priority` on `create`.

## Decision

Add `--status`, `--priority`, `--type`, `--tags`, and `--depends-on` to the `create`
subcommand, using the same flag names, short flags, and parsing types as `update`.

### Example

```sh
tndm ticket create "Fix login timeout" \
  --priority p1 --type bug --tags auth,security \
  --depends-on TNDM-AAAAAA,TNDM-BBBBBB --status in_progress
```

## Scope

- **CLI layer** (`crates/tandem-cli/src/main.rs`): Add flags to `Create` variant, build
  `TicketMeta` from defaults then override with provided flags.
- **Core**: No changes. `TicketMeta::new()` still provides defaults; the CLI layer
  overrides after construction.
- **Tests**: Integration tests for combined and individual flag usage.
- **Plugin docs**: Update `plugin/tndm/skills/ticket/SKILL.md` and
  `plugin/tndm/skills/ticket/references/command-reference.md`.

### Not in scope

- Changing `NewTicket` or `TicketMeta` in `tandem-core`.
- Adding `--title` flag to `create` (title is already the positional argument).

## Defaults when flags are omitted

Unchanged: status=todo, priority=p2, type=task, tags=[], depends_on=[].

## Flag reference

| Flag           | Short | Values                                          |
|----------------|-------|-------------------------------------------------|
| `--status`     | `-s`  | `todo` `in_progress` `blocked` `done`           |
| `--priority`   | `-p`  | `p0` `p1` `p2` `p3` `p4`  (p0 = critical)      |
| `--type`       | `-T`  | `task` `bug` `feature` `chore` `epic`           |
| `--tags`       | `-g`  | comma-separated strings                         |
| `--depends-on` | `-d`  | comma-separated ticket IDs                      |

Short flags match `update` (`-s`, `-p`, `-T`, `-g`, `-d`).

## Implementation notes

- `--status` sets `TicketState.status` (status lives in state, not meta).
- `--tags` and `--depends-on` use the same comma-split, sort, dedup logic from `update`.
- `--content` and `--content-file` remain unchanged.
- The handler constructs `TicketMeta::new(id, title)` then mutates fields before
  calling `store.create_ticket()`.
