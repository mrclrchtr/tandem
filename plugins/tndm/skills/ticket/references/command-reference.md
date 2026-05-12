# tndm Command Reference

Complete syntax for all `tndm` subcommands and flags.

## tndm ticket create

Create a new ticket. Prints the ticket ID (text) or full ticket (JSON).

```sh
tndm ticket create <TITLE> [OPTIONS]

Options:
      --id <ID>                  Explicit ticket ID (e.g. TNDM-A1B2C3). Auto-generated if omitted.
  -s, --status <STATUS>          Initial status. Values: todo | in_progress | blocked | done
  -p, --priority <PRIORITY>      Initial priority. Values: p0 | p1 | p2 | p3 | p4
  -T, --type <TYPE>              Initial type. Values: task | bug | feature | chore | epic
  -g, --tags <TAGS>              Comma-separated tags.
  -d, --depends-on <IDS>         Comma-separated ticket IDs for dependencies.
  -e, --effort <SIZE>            Effort estimate. Values: xs | s | m | l | xl
      --content <BODY>           Inline content body.
      --content-file <PATH>      Load ticket body from a markdown file.
      --json                     Output the created ticket as JSON.

Content can also be piped via stdin.
--content, --content-file, and stdin are mutually exclusive.
```

Defaults when flags are omitted: status=todo, priority=p2, type=task, effort=unset, tags=[], depends_on=[].

Examples:

```sh
# Minimal — auto-generates ID
tndm ticket create "Refactor auth module"

# With metadata — set everything at creation
tndm ticket create "Fix login timeout" \
  --priority p1 --type bug --tags auth,security \
  --depends-on TNDM-B2C3D4 --status in_progress

# With explicit ID
tndm ticket create "Fix login redirect" --id TNDM-FIX001
```

## tndm ticket update

Update fields on an existing ticket.

```sh
tndm ticket update <ID> [OPTIONS]

Options:
  -s, --status <STATUS>         New status. Values: todo | in_progress | blocked | done
  -p, --priority <PRIORITY>     New priority. Values: p0 | p1 | p2 | p3 | p4
  -t, --title <TITLE>           Replace the ticket title.
  -T, --type <TYPE>             Set type. Values: task | bug | feature | chore | epic
  -g, --tags <TAGS>             Comma-separated tags (replaces the full list; empty string clears).
      --add-tags <TAGS>         Comma-separated tags to add (preserves existing tags).
      --remove-tags <TAGS>      Comma-separated tags to remove from the list.
      --add-tags and --remove-tags conflict with --tags.
  -d, --depends-on <IDS>        Comma-separated ticket IDs (replaces the full list).
  -e, --effort <SIZE>           Effort estimate. Values: xs | s | m | l | xl
      --content <BODY>          Inline content body replacing existing content.
      --content-file <PATH>     Replace ticket body with content from a markdown file.
      --json                    Output the updated ticket as JSON.

Content can also be piped via stdin.
--content, --content-file, and stdin are mutually exclusive.
```

Examples:

```sh
# Mark in-progress immediately after creating
tndm ticket update TNDM-A1B2C3 -s in_progress

# Block with reason (use document registry — no large CLI strings):
tndm ticket doc create TNDM-A1B2C3 block-reason
# Edit docs/block-reason.md with your edit tool, then:
tndm ticket sync TNDM-A1B2C3
tndm ticket update TNDM-A1B2C3 -s blocked

# Set priority and type
tndm ticket update TNDM-A1B2C3 -p p1 -T bug

# Replace tags
tndm ticket update TNDM-A1B2C3 -g auth,security,p1

# Add tags (preserves existing)
tndm ticket update TNDM-A1B2C3 --add-tags flow:planned

# Remove tags
tndm ticket update TNDM-A1B2C3 --remove-tags oldtag,deprecated

# Declare dependency
tndm ticket update TNDM-A1B2C3 -d TNDM-B2C3D4

# Mark done
tndm ticket update TNDM-A1B2C3 -s done

# Replace content from file (when content already exists on disk)
tndm ticket update TNDM-A1B2C3 --content-file /tmp/blocker.md
```

## tndm ticket doc create

Create and register a new document file for a ticket. Returns the file path.

```sh
tndm ticket doc create <ID> <NAME> [OPTIONS]

Options:
      --json    Output as JSON with document metadata.
```

The document file is created at `docs/<name>.md` inside the ticket directory.
Once created, agents should:
1. Edit the returned path with their edit tool.
2. Run `tndm ticket sync <ID>` to refresh fingerprints.

Examples:

```sh
# Create a plan document
.tndm/tickets/TNDM-A1B2C3/docs/plan.md

# With JSON output
tndm ticket doc create TNDM-A1B2C3 archive --json
```

If the document is already registered, returns the existing path without
overwriting content.

## tndm ticket sync

Recompute document fingerprints after file edits, update `revision` and
`updated_at`.

```sh
tndm ticket sync <ID> [OPTIONS]

Options:
      --json    Output the updated ticket as JSON.
```

Run this after editing any registered ticket document file. Agents should
_not_ pass large content through `--content` on `ticket update` — use the
document registry + edit + sync workflow instead.

Examples:

```sh
# Sync fingerprints after editing docs/plan.md
TNDM-A1B2C3

# With JSON output
tndm ticket sync TNDM-A1B2C3 --json
```

## tndm ticket show

Display a single ticket by ID with a rich, human-readable layout.

```sh
tndm ticket show <ID> [OPTIONS]

Options:
  --json    Output as JSON.
```

The standard (non-JSON) output includes:
- **Header** — ticket ID and title
- **Metadata** — status, priority, type, effort, tags, dependencies, timestamp
- **Status color-coding** — todo (yellow), in_progress (blue), blocked (red), done (green)
- **Markdown rendered content** — headings, bold, italic, inline code, code blocks,
  lists, and blockquotes are all styled in the terminal

Colors and markdown rendering automatically disable when output is piped.

Examples:

```sh
tndm ticket show TNDM-A1B2C3
tndm ticket show TNDM-A1B2C3 --json
```

## tndm ticket list

List tickets in the repository, sorted by priority (highest first), then by ID.
Done tickets are hidden by default.

```sh
tndm ticket list [OPTIONS]

Options:
  --all     Include tickets with status "done".
  --definition <STATE>
            Filter by definition state backed by reserved tags.
            Values: ready | questions | unknown
  --json    Output as JSON array.
```

Examples:

```sh
tndm ticket list
tndm ticket list --all
tndm ticket list --definition ready
tndm ticket list --definition questions --json
tndm ticket list --json

# Filter in-progress tickets with jq
tndm ticket list --json | jq '[.[] | select(.status == "in_progress")]'

# Show only blocked tickets
tndm ticket list --json | jq '[.[] | select(.status == "blocked")]'

# Show done tickets
tndm ticket list --all --json | jq '[.[] | select(.status == "done")]'
```

Definition filtering uses reserved tags:

- `definition:ready` — ticket is currently considered implementable
- `definition:questions` — ticket still has open definition questions
- `unknown` — no `definition:*` tag is present

Convention:

- Keep richer detail in `content.md`, especially `Open Questions`, `Acceptance`, and `Ready When`.
- Use at most one current `definition:*` tag at a time.
- Prefer tags for current state only; do not treat them as historical refinement counts.

## tndm awareness

Show which tickets differ between the current branch/worktree and another git ref.

```sh
tndm awareness --against <REF> [OPTIONS]

Required:
  --against <REF>   Git ref to compare against (branch name, tag, or commit SHA).

Options:
  --json            Output as structured JSON (recommended for agent consumption).
```

Examples:

```sh
# Compare against main branch
tndm awareness --against main --json

# Compare against a remote branch
tndm awareness --against origin/feature-auth --json

# Compare against a specific commit
tndm awareness --against abc1234 --json

# Human-readable summary
tndm awareness --against main
```

JSON output structure:

```json
{
  "added_current": [
    { "id": "TNDM-NEW01", "title": "...", "status": "in_progress", ... }
  ],
  "added_against": [
    { "id": "TNDM-NEW02", "title": "...", "status": "done", ... }
  ],
  "diverged": [
    {
      "id": "TNDM-SHARED",
      "current": { "status": "in_progress", "priority": "p1", ... },
      "against":  { "status": "done",        "priority": "p2", ... },
      "diff": {
        "status":   { "current": "in_progress", "against": "done" },
        "priority": { "current": "p1",           "against": "p2"  }
      }
    }
  ]
}
```

Field-level diffs are provided for: `status`, `priority`, `effort`, `title`, `type`, `depends_on`, `tags`.

## tndm fmt

Normalise ticket files to canonical TOML format.

```sh
tndm fmt [OPTIONS]

Options:
  --check   Verify format without writing changes. Exits non-zero if any file needs normalisation.
            Used in CI to enforce clean diffs.
```

Examples:

```sh
# Normalise all ticket files in place
tndm fmt

# CI check — fail if files are not canonical
tndm fmt --check
```

Also checks registered document fingerprints. `tndm fmt --check` fails when document
files have been edited without running `tndm ticket sync <ID>`.

## Field Enum Reference

### Status (`--status`)

| Value        | Meaning                                       |
|--------------|-----------------------------------------------|
| `todo`       | Work not yet started (default on create)      |
| `in_progress`| Work actively in progress                     |
| `blocked`    | Work is blocked by an external dependency     |
| `done`       | Work complete                                 |

### Priority (`--priority`)

| Value | Meaning             |
|-------|---------------------|
| `p0`  | Critical / on-fire  |
| `p1`  | High                |
| `p2`  | Medium (default)    |
| `p3`  | Low                 |
| `p4`  | Backlog / someday   |

### Type (`--type`)

| Value     | Meaning                   |
|-----------|---------------------------|
| `task`    | General work item (default)|
| `bug`     | Defect to fix             |
| `feature` | New capability            |
| `chore`   | Maintenance / housekeeping|
| `epic`    | Large multi-ticket effort |

### Effort (`--effort`)

| Value | Meaning           |
|-------|-------------------|
| `xs`  | Extra-small       |
| `s`   | Small             |
| `m`   | Medium            |
| `l`   | Large             |
| `xl`  | Extra-large       |

Effort is optional. Omit to leave unset; omitting on update leaves the existing value unchanged.

## Ticket File Structure

Each ticket is stored as a directory with registered documents:

```
.tndm/tickets/TNDM-XXXXXX/
├── meta.toml              # stable metadata: id, title, type, priority, effort, tags, depends_on, [[documents]]
├── state.toml             # volatile state: status, revision, updated_at, [document_fingerprints]
├── content.md             # default registered document (always present)
└── docs/
    ├── plan.md            # additional registered documents (created via `tndm ticket doc create`)
    └── archive.md
```

- `meta.toml` includes a `[[documents]]` table listing each registered file by name and path.
- `state.toml` includes `[document_fingerprints]` with SHA-256 hashes for each document.
- After editing a registered document file with an edit tool, run `tndm ticket sync <ID>`
  to refresh fingerprints. `tndm fmt --check` will fail on stale fingerprints.

## Repository Configuration

Optional config at `.tndm/config.toml`:

```toml
schema_version = 1

[id]
prefix = "TNDM"

[templates]
content = """
## Context

## Goal

## Open Questions

- [ ] Question or ambiguity 1
- [ ] Question or ambiguity 2

## Acceptance

- [ ] Observable outcome 1
- [ ] Observable outcome 2

## Ready When

- [ ] Scope is clear
- [ ] Dependencies are known
- [ ] Open questions are resolved or explicitly deferred
- [ ] Acceptance is specific enough for implementation
"""
```
