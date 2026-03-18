---
name: ticket
description: >
  This skill MUST be used when an agent or user mentions "ticket", "tndm", "TNDM-*", "track this
  work", "create a ticket", "update ticket", "mark ticket as done", "mark ticket as in_progress",
  "show ticket", "list tickets", "what tickets are open", "add a tag to ticket", "set priority on
  ticket", "ticket status", "ticket blocked", or any ticket lifecycle operation. Also use when
  starting any development task that should be tracked, or when a conversation references a ticket
  ID (e.g. "fix TNDM-XXXXXX").
version: 0.2.0
argument-hint: create <title> | update <ID> [--status <s>] [--priority <p>] | show <ID> | list
---

# tndm Ticket Operations

tndm is a git-aware ticket coordination system for AI agents in a monorepo. Ticket state is stored
in the repository — no central service required. Other agents rely on ticket status to coordinate,
so keeping it current is essential.

## Workflow Protocol

Follow this for every development task:

### 1. Create a ticket before starting work

```sh
tndm ticket create "Brief title describing the task"
```

Note the returned ticket ID (format: `TNDM-XXXXXX`). Immediately update status:

```sh
tndm ticket update <ID> --status in_progress
```

### 2. Keep status current as work progresses

```sh
# When blocked — document the reason via heredoc (do not create temporary files):
tndm ticket update <ID> --status blocked <<'EOF'
Blocked: waiting for PR #42 review
EOF

# When unblocked and resuming:
tndm ticket update <ID> --status in_progress
```

### 3. When work is complete — mark done

```sh
tndm ticket update <ID> --status done
```

### 4. Commit ticket changes immediately

Ticket creation and status updates are coordination signals — other agents can only see them once
committed. Always commit `.tndm/` changes right away.

## Working on an Existing Ticket

When asked to fix, work on, or continue a ticket (e.g. "fix TNDM-XXXXXX"):

1. Show the ticket to understand it: `tndm ticket show <ID>`
2. Set status to `in_progress`: `tndm ticket update <ID> --status in_progress`
3. Commit the status change immediately
4. Do the work
5. Set status to `done`: `tndm ticket update <ID> --status done`
6. Commit the status change

## Commands

### Create

```sh
tndm ticket create "<title>"
```

With optional content body (use a heredoc — do **not** create temporary files):

```sh
tndm ticket create "Implement OAuth flow" <<'EOF'
## Description

Add OAuth 2.0 authorization code flow.

## Acceptance

- Users can sign in with Google
EOF
```

### Update

```sh
# Change status
tndm ticket update TNDM-XXXXXX --status in_progress
tndm ticket update TNDM-XXXXXX --status blocked
tndm ticket update TNDM-XXXXXX --status done

# Change priority and type
tndm ticket update TNDM-XXXXXX --priority p1 --type bug

# Set tags (replaces existing list)
tndm ticket update TNDM-XXXXXX --tags auth,security

# Clear tags
tndm ticket update TNDM-XXXXXX --tags ""

# Set dependencies
tndm ticket update TNDM-XXXXXX --depends-on TNDM-AAAAAA,TNDM-BBBBBB

# Replace content body (use a heredoc — do not create temporary files)
tndm ticket update TNDM-XXXXXX <<'EOF'
## Notes

Updated design after review feedback.
EOF

# Combine multiple fields
tndm ticket update TNDM-XXXXXX --status done --priority p1
```

### Show

```sh
tndm ticket show TNDM-XXXXXX
tndm ticket show TNDM-XXXXXX --json
```

### List

By default, done tickets are hidden. Use `--all` to include them.

```sh
tndm ticket list
tndm ticket list --all
tndm ticket list --json

# Useful jq filters
tndm ticket list --all --json | jq '[.tickets[] | select(.status == "done")]'
tndm ticket list --json | jq '[.tickets[] | select(.status == "in_progress")]'
tndm ticket list --json | jq '[.tickets[] | select(.status == "blocked")]'
tndm ticket list --json | jq '[.tickets[] | select(.priority == "p0" or .priority == "p1")]'
```

## Field Reference

| Flag           | Values                                          |
|----------------|-------------------------------------------------|
| `--status`     | `todo` `in_progress` `blocked` `done`           |
| `--priority`   | `p0` `p1` `p2` `p3` `p4`  (p0 = critical)      |
| `--type`       | `task` `bug` `feature` `chore` `epic`           |
| `--tags`       | comma-separated strings                         |
| `--depends-on` | comma-separated ticket IDs                      |

Use `--json` on read commands (`show`, `list`) when parsing output. Mutations (`create`, `update`)
print just the ticket ID by default — skip `--json` to save tokens.

## Additional Resources

For complete command syntax, all flags, enum values, ticket file layout, and repository
configuration options, see:

- **`references/command-reference.md`** — full flag reference, all subcommands, enum values,
  example invocations, ticket file structure, and `.tndm/config.toml` options
