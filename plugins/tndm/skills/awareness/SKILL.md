---
name: awareness
description: >
  This skill MUST be used when an agent or user asks to "check awareness", "check what changed
  on branch X", "what tickets are on main", "compare tickets between branches", "check for ticket
  conflicts", "what did other agents work on", "tndm awareness", "before I start working on X check
  the branch", "what's in flight on another branch", "coordination", "worktree", "branch
  coordination", or "run awareness against <ref>". Use when starting work that may overlap with
  another branch or worktree.
version: 0.2.0
argument-hint: <branch-or-ref>
---

# tndm Awareness Check

Run an awareness check to discover how tickets differ between the current branch/worktree and
another git ref before starting work that could conflict.

## When to Run

Run awareness before:
- Starting a feature that may touch files another agent is working on
- Branching off a branch that has active in-progress tickets
- Merging or rebasing onto a branch with open work
- Resuming work after another agent may have progressed

## Command

```sh
tndm awareness --against <branch-or-ref> --json
```

Where `<branch-or-ref>` is the git ref to compare against: a branch name (`main`,
`origin/feature-auth`), a tag, or a commit SHA.

## Interpreting the Output

```json
{
  "added_current":  [ ... ],
  "added_against":  [ ... ],
  "diverged":       [ ... ]
}
```

### `added_current`

Tickets that exist only on the current branch — work this agent has started that the other ref
does not know about yet. No immediate action needed, but note these for merge context.

### `added_against`

Tickets that exist only on the reference branch — work in flight on that branch not yet visible
here. Review these to understand what the other agent is doing and whether it overlaps with the
planned work.

### `diverged`

Tickets that exist on both refs but with differing field values. Each entry includes a `diff`
object with field-level changes:

```json
{
  "id": "TNDM-XXXXXX",
  "current": { "status": "in_progress", "priority": "p1" },
  "against":  { "status": "done",        "priority": "p2" },
  "diff": {
    "status":   { "current": "in_progress", "against": "done" },
    "priority": { "current": "p1",           "against": "p2"  }
  }
}
```

## Actions Based on Results

| Situation | Action |
|---|---|
| `added_against` contains `in_progress` tickets touching the same area | Coordinate before proceeding — create a dependency or adjust scope |
| `diverged` has a ticket where `against.status == "done"` but `current.status != "done"` | The other branch already finished this — check whether to adopt its resolution |
| `diverged` has conflicting `depends_on` | Reconcile dependency chains before merging |
| All arrays empty | No ticket conflicts — safe to proceed |

## After Awareness — Commit Any Ticket Updates

If you update tickets based on awareness results, commit `.tndm/` changes immediately so other
agents can see them.

## Example

```sh
# Check what the main branch has before branching off
tndm awareness --against main --json

# Check a remote feature branch
tndm awareness --against origin/feature-payments --json
```

After reviewing the output, update any affected tickets and adjust the work plan to avoid
duplicate or conflicting efforts.
