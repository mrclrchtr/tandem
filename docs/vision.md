# Vision

## What is tandem?

**tandem** is a git-aware ticket coordination system for AI agents in a monorepo.

It stores ticket state in the repository, works across branches and git worktrees, and helps agents understand what
other agents or parallel work contexts have changed. It is meant to be operable by agents through a deterministic
interface and usable by humans through a CLI for oversight.

CLI command: **`tndm`**

## For whom?

The primary user is an AI agent operating alongside other agents in the same repository. Agents create, update, and
query tickets. They use awareness commands to discover what other agents (on other branches or worktrees) have changed.

Humans use `tndm` for oversight: reviewing ticket state, checking awareness reports, and resolving conflicts. Humans
are not the primary ticket creators or consumers — agents are.

## Core workflow

1. **Agent A** picks up work and creates a ticket: `tndm ticket create "Refactor auth module"`. It gets `TNDM-A1B2C3`.
2. Agent A works on `branch-a`, updating the ticket's status to `in_progress`.
3. **Agent B** is working on `branch-b` on a related area. Before starting, it runs:
   `tndm awareness --against branch-a`
4. The awareness report (JSON) tells Agent B that `TNDM-A1B2C3` was added on `branch-a` with status `in_progress` and
   touches the auth module.
5. Agent B adjusts its plan to avoid conflicting changes.
6. Later, both branches merge. The deterministic TOML format and `tndm fmt --check` catch any formatting drift in CI.

## V1 success criteria

V1 is useful when an agent can:

- Create and manage tickets entirely through the CLI.
- Use lightweight current-state signals such as `definition:*` tags together with structured ticket content.
- Discover what tickets exist or changed on another branch or worktree via `tndm awareness --against <ref>`.
- Consume structured JSON output to make autonomous decisions about conflicts or coordination.
- Trust that ticket files produce clean diffs and pass `tndm fmt --check` in CI.

## V1 scope

V1 is the minimum useful ticket coordination system:

- Strict ticket schema with validated fields.
- Git-aware: compare ticket state across branches and worktrees.
- Structured awareness output (JSON) distinguishing added, removed, and diverged tickets with field-level diffs.
- Deterministic on-disk format (`tndm fmt`) for clean version control.
- No central service, no background process, no LLM dependency.

## Possible extensions (non-V1)

- **Semantic conflict detection:** Three-way merge-base comparison with conflict categories (state, priority,
  dependency, blocking conflicts). Currently, awareness uses two-way snapshot comparison.
- **Issue tracker adapters:** Optional integrations with GitHub/Linear/etc. to mirror ticket metadata. Repo-local
  ticket files remain the system of record.
- **Worktree lifecycle helpers:** Commands that create/manage per-ticket worktrees with naming, cleanup, and
  deterministic per-ticket runtime configuration (e.g. ports).
- **Phase-oriented workflows:** Commands that structure work into phases (analyze, plan, implement) and persist outputs
  as machine-readable artifacts associated with tickets.
- **Recap/summarization plugin:** A layer that turns structured awareness output into a concise recap for an agent,
  without making the core model depend on an LLM.
- **Editor UI:** VS Code extension surfacing tickets, awareness changes, and cross-worktree context switching.
- **Ticket decomposition + parallelism:** Splitting tickets into children with dependencies and orchestrating parallel
  execution across worktrees/agents.
- **Automation + CI hooks:** Pre-commit/pre-push/CI integrations running `tndm fmt --check` and awareness checks,
  blocking merges on specific conflicts.

## Reference points

See `docs/references.md` for detailed competitive analysis. Key references:

- [Taskwarrior](https://github.com/GothenburgBitFactory/taskwarrior) — CLI UX and local-first task management.
- [Beads](https://github.com/steveyegge/beads) — Distributed git-backed graph issue tracker for agents.
- [ticket](https://github.com/wedow/ticket) — Git-backed markdown issue tracker for agents.
- [bodega](https://github.com/bjia56/bodega) — Git-native issue tracker, closest external reference in spirit.
- [iloom](https://github.com/iloom-ai/iloom-cli) — AI development control plane with worktree isolation patterns.

## Related docs

- Architecture overview: `docs/architecture.md`
- Design decisions: `docs/decisions.md`
- Competitive analysis: `docs/references.md`
