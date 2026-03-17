---
name: ticket
description: >
  This skill should be used when an agent or user asks to "create a ticket", "track this work",
  "make a tndm ticket", "update ticket TNDM-X", "mark ticket as done", "mark ticket as
  in_progress", "show ticket TNDM-X", "list tickets", "list open tickets", "what tickets are
  open", "add a tag to ticket", "set priority on ticket", "TNDM-" followed by a ticket ID,
  "ticket status", "ticket blocked", or any ticket lifecycle operation.
version: 0.1.0
argument-hint: create <title> | update <ID> [--status <s>] [--priority <p>] | show <ID> | list
---

# tndm Ticket Operations

Manage the full lifecycle of tndm tickets: create, update, show, and list.

## Create a Ticket

Create a ticket before starting any development task.

```sh
tndm ticket create "<title>" --json
```

Immediately update status to `in_progress`:

```sh
tndm ticket update <ID> --status in_progress --json
```

With optional content body:

```sh
tndm ticket create "Implement OAuth flow" --content-file /tmp/description.md --json
```

## Update a Ticket

Update any field on an existing ticket.

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

# Replace content body
tndm ticket update TNDM-XXXXXX --content-file /tmp/updated-notes.md

# Combine multiple fields
tndm ticket update TNDM-XXXXXX --status done --priority p1 --json
```

## Show a Ticket

```sh
tndm ticket show TNDM-XXXXXX
tndm ticket show TNDM-XXXXXX --json
```

## List All Tickets

```sh
tndm ticket list
tndm ticket list --json

# Useful jq filters
tndm ticket list --json | jq '[.[] | select(.status == "in_progress")]'
tndm ticket list --json | jq '[.[] | select(.status == "blocked")]'
tndm ticket list --json | jq '[.[] | select(.priority == "p0" or .priority == "p1")]'
```

## Field Reference

| Flag           | Values                                          |
|----------------|-------------------------------------------------|
| `--status`     | `todo` `in_progress` `blocked` `done`           |
| `--priority`   | `p0` `p1` `p2` `p3` `p4`  (p0 = critical)      |
| `--type`       | `task` `bug` `feature` `chore` `epic`           |
| `--tags`       | comma-separated strings                         |
| `--depends-on` | comma-separated ticket IDs                      |

## After Every Mutation

Run `tndm fmt` after every create or update to keep files in canonical format:

```sh
tndm fmt
```

CI enforces this via `tndm fmt --check` — files not normalised will fail the check.
