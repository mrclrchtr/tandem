# tndm Command Reference

Complete syntax for all `tndm` subcommands and flags.

## tndm ticket create

Create a new ticket. Prints the created ticket (text or JSON).

```sh
tndm ticket create <TITLE> [OPTIONS]

Options:
  --id <ID>               Explicit ticket ID (e.g. TNDM-A1B2C3). Auto-generated if omitted.
  --content <BODY>        Inline content body.
  --content-file <PATH>   Load ticket body from a markdown file.
  --json                  Output the created ticket as JSON.

Content can also be piped via stdin (heredoc recommended for agents).
--content, --content-file, and stdin are mutually exclusive.
```

Examples:

```sh
# Minimal — auto-generates ID, prints the ticket ID
tndm ticket create "Refactor auth module"

# With explicit ID
tndm ticket create "Fix login redirect" --id TNDM-FIX001

# With content via heredoc (preferred for agents — no temp files needed)
tndm ticket create "Implement OAuth flow" <<'EOF'
## Description

Add OAuth 2.0 authorization code flow.
EOF

# With content from file (when content already exists on disk)
tndm ticket create "Implement OAuth flow" --content-file /tmp/ticket-body.md
```

Output (JSON):

```json
{
  "id": "TNDM-A1B2C3",
  "title": "Refactor auth module",
  "type": "task",
  "status": "todo",
  "priority": "p2",
  "tags": [],
  "depends_on": [],
  "created_at": "2026-03-17T10:00:00Z"
}
```

## tndm ticket update

Update fields on an existing ticket.

```sh
tndm ticket update <ID> [OPTIONS]

Options:
  --status <STATUS>         New status. Values: todo | in_progress | blocked | done
  --priority <PRIORITY>     New priority. Values: p0 | p1 | p2 | p3 | p4
  --title <TITLE>           Replace the ticket title.
  --type <TYPE>             Set type. Values: task | bug | feature | chore | epic
  --tags <TAGS>             Comma-separated tags (replaces the full list; empty string clears).
  --depends-on <IDS>        Comma-separated ticket IDs (replaces the full list).
  --content <BODY>          Inline content body replacing existing content.
  --content-file <PATH>     Replace ticket body with content from a markdown file.
  --json                    Output the updated ticket as JSON.

Content can also be piped via stdin (heredoc recommended for agents).
--content, --content-file, and stdin are mutually exclusive.
```

Examples:

```sh
# Mark in-progress immediately after creating
tndm ticket update TNDM-A1B2C3 --status in_progress

# Block with reason via heredoc (preferred for agents — no temp files needed)
tndm ticket update TNDM-A1B2C3 --status blocked <<'EOF'
Blocked: waiting for PR #42 review
EOF

# Set priority and type
tndm ticket update TNDM-A1B2C3 --priority p1 --type bug

# Add tags
tndm ticket update TNDM-A1B2C3 --tags auth,security,p1

# Declare dependency
tndm ticket update TNDM-A1B2C3 --depends-on TNDM-B2C3D4

# Mark done
tndm ticket update TNDM-A1B2C3 --status done

# Replace content from file (when content already exists on disk)
tndm ticket update TNDM-A1B2C3 --content-file /tmp/blocker.md
```

## tndm ticket show

Display a single ticket by ID.

```sh
tndm ticket show <ID> [OPTIONS]

Options:
  --json    Output as JSON.
```

Examples:

```sh
tndm ticket show TNDM-A1B2C3
tndm ticket show TNDM-A1B2C3 --json
```

## tndm ticket list

List tickets in the repository. Done tickets are hidden by default.

```sh
tndm ticket list [OPTIONS]

Options:
  --all     Include tickets with status "done".
  --json    Output as JSON array.
```

Examples:

```sh
tndm ticket list
tndm ticket list --all
tndm ticket list --json

# Filter in-progress tickets with jq
tndm ticket list --json | jq '[.[] | select(.status == "in_progress")]'

# Show only blocked tickets
tndm ticket list --json | jq '[.[] | select(.status == "blocked")]'

# Show done tickets
tndm ticket list --all --json | jq '[.[] | select(.status == "done")]'
```

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

Field-level diffs are provided for: `status`, `priority`, `depends_on`.

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

Run `tndm fmt` after every `ticket create` or `ticket update` to ensure clean git diffs.

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

## Ticket File Structure

Each ticket is stored as a directory:

```
.tndm/tickets/TNDM-XXXXXX/
├── meta.toml     # stable metadata: id, title, type, priority, tags, depends_on
├── state.toml    # volatile state: status, revision, updated_at
└── content.md    # freeform markdown body (optional — set via heredoc, --content, or --content-file)
```

## Repository Configuration

Optional config at `.tndm/config.toml`:

```toml
id_prefix = "TNDM"          # prefix for auto-generated IDs
content_template = "..."    # default markdown template for new tickets
```
