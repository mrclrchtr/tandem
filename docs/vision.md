# Goal

**tandem** is a git-aware ticket coordination system for AI agents in a monorepo.

It stores ticket state in the repository, works across branches and git worktrees, and helps agents understand what
other agents or parallel work contexts have changed. It is meant to be operable by agents through a deterministic
interface and usable by humans through a CLI.

CLI command: **`tndm`**

Repository architecture summary: `docs/architecture.md`.

# Decisions

## Product scope

- The core product is a coordination layer, not just a passive planning format.
- The core feature is a ticket system for AI agents.
- The main requirement is that agents can tell what other agents or parallel branches/worktrees are doing.
- Exploration, conversation forks, and alternative implementations are relevant, but they are not the core feature.

## Architecture direction

- The system is git-aware.
- It is designed to work inside a monorepo.
- It must support git worktrees.
- It should detect that changes happened on other branches/worktrees, including remote changes.
- It should not depend on a central web service as the main architecture.
- Repo-local ticket files are the system of record. Optional adapters may integrate with external issue trackers
  (GitHub/Linear/etc.), but those should not be required for core operation.

## Storage model

- Ticket state is stored in the repository.
- Storage model: one directory per ticket.
- Proposed structure:
    - `.tndm/tickets/TICKET-123/ticket.<format>`
    - `.tndm/tickets/TICKET-123/state.<format>`
    - `.tndm/tickets/TICKET-123/notes.md`
- The split between stable metadata and volatile state is intentional to reduce Git friction.

## File format + determinism

- Ticket metadata and state are stored as structured data files using a single repo-wide format (`<format>`).
- The exact on-disk format is an implementation choice and should be selected based on tooling quality in the chosen
  implementation language and ecosystem.
- The CLI is the canonical writer/formatter for these files.
- The system should provide `tndm fmt` and `tndm fmt --check` to enforce stable serialization (ordering, whitespace,
  encoding, timestamp representation) and minimize churn in diffs.
- Freeform text belongs in `notes.md`, not in the structured data files.

## Awareness model

- V1 must provide awareness of other branches/worktrees.
- This awareness is a core requirement, not an optional extra.
- Awareness is a command-bound function.
- The exact commands that trigger awareness checks are intentionally left open for now.
- The baseline behavior is deterministic:
    - identify relevant ticket changes elsewhere
    - expose those changes to the requesting agent in a structured form
- Awareness may surface local, uncommitted ticket changes as early hints, but should distinguish them from changes
  observed on Git refs (since uncommitted state is machine-local and non-reproducible).
- Structured awareness output should distinguish at least:
    - direct change: the same ticket changed elsewhere
    - dependency change: a dependency of the current ticket changed elsewhere
    - related change: optional later extension
- An optional plugin may later summarize those changes for an agent.

## LLM integration

- LLM integration is not part of the required core.
- Optional direction: a plugin that summarizes structured changes for the requesting agent.
- The deterministic ticket/change model must exist independently of any LLM component.

## Ticket model

- The ticket model should be strictly validated.
- Current direction is closer to a richer ticket schema, but without an assignee field.
- Exact fields still need implementation-level review.
- `updated_at` is a tool-managed field in `state.<format>` and part of the core model.
- `updated_at` is load-bearing for freshness, awareness, and change comparison.
- Because wall clocks can skew across machines/worktrees, the system should avoid relying on `updated_at` as the only
  ordering/comparison signal and may additionally use monotonic/tool-derived signals (e.g. a per-ticket revision and/or
  the Git commit graph) where needed.

## Conflict handling

- The system should support semantic conflict detection, not only raw Git conflict handling.
- V1 semantic conflicts should be defined narrowly.
- When comparing divergent refs, semantic conflict detection should use the Git merge-base of the compared refs as the
  baseline (three-way comparison), rather than comparing only “latest” states by timestamp.
- Initial semantic conflict categories:
    - state conflict: the same ticket status changed differently on different branches
    - priority conflict: the same ticket priority diverged
    - dependency conflict: the same ticket dependencies diverged
    - blocking conflict: one branch blocks a ticket while another continues it

## Automation behavior

- The system should perform automatic checks in the context of relevant commands.
- It should not require a background service to notice useful context.
- The current direction is contextual automation rather than a permanently running process.

## Branch/worktree information in tickets

- Tickets may carry light branch/worktree-related metadata.
- Branch/worktree context should be present where useful, but there is no decision to make branch-specific attempts the
  main domain model.

## Interfaces

- The product must support AI-first usage and also be usable by humans.
- The interface direction is:
    - deterministic CLI for humans and agents
    - machine-readable output for agents
    - possible future adapter layers if needed
- Name: **tandem**
- CLI command: **`tndm`**

## V1 scope

- V1 should stay small.
- The focus is on the minimum useful ticket coordination system with:
    - strict schema
    - Git awareness
    - worktree awareness
    - branch/remote change awareness
    - semantic visibility into what changed elsewhere

## Possible extensions (non-V1)

- **Issue tracker adapters:** Optional integrations with GitHub/Linear/etc. to mirror ticket metadata, link refs, and/or
  publish derived artifacts (summaries, plans) into tracker comments. Repo-local ticket files remain the system of
  record.
- **Worktree lifecycle helpers:** Optional commands that create/manage per-ticket worktrees and standardize isolation
  ergonomics (naming, cleanup, and deterministic per-ticket runtime configuration such as ports).
- **Phase-oriented workflows:** Optional higher-level commands that structure work into phases (e.g. analyze → plan →
  implement) and persist their outputs into deterministic, machine-readable artifacts associated with tickets.
- **Recap/summarization plugin:** Optional layer that turns structured awareness output and ticket diffs into a concise
  recap for a human or agent, without making the core model depend on an LLM.
- **Editor UI:** Optional VS Code (or other IDE) extension that surfaces active tickets, awareness changes, conflicts,
  and recaps, and makes cross-worktree context switching easier.
- **Ticket decomposition + parallelism:** Optional support for splitting a ticket into child tickets with dependencies
  and orchestrating parallel execution across worktrees/agents, while keeping the underlying ticket model and awareness
  outputs deterministic.
- **Automation + CI hooks:** Optional pre-commit/pre-push/CI integrations that run `tndm fmt --check` and awareness
  checks, and can block merges on specific semantic conflicts.

## Reference points

### [Taskwarrior](https://github.com/GothenburgBitFactory/taskwarrior)

- Taskwarrior is a command line task list management utility with a large ecosystem of tools, hooks, and extensions.
- It is a useful reference for CLI UX, query/report ergonomics, and local-first task management workflows.
- It is not the tandem baseline because it is not git-aware and does not target multi-branch/worktree coordination,
  repo-local system-of-record state, or semantic conflict detection.

### [Beads](https://github.com/steveyegge/beads)

- Beads is a distributed, git-backed graph issue tracker for AI agents.
- It centers on a Dolt-powered architecture with version-controlled SQL, branching, and built-in sync.
- It already includes dependency-aware tracking, JSON output, claiming workflow, and auto-ready task detection.
- It is a useful reference for task graph behavior and machine-readable CLI design.
- It is not the tandem baseline because tandem is currently defined around repo-local ticket files and branch/worktree
  awareness as a first-class feature.

### [ticket](https://github.com/wedow/ticket)

- `ticket` is a git-backed issue tracker for AI agents.
- It stores tickets as markdown files with YAML frontmatter in `.tickets/`.
- It supports dependency tracking, ready/blocked views, JSON query output, and a plugin system.
- It is closer to tandem than Beads because it is repo-native, file-based, and lightweight.
- It is still not the tandem baseline as-is because tandem currently aims for:
    - per-ticket directories instead of single markdown files
    - stricter separation of stable ticket metadata and volatile state
    - branch/worktree awareness as a core feature rather than a later add-on
    - semantic awareness and conflict categories as part of the design

### [bodega](https://github.com/bjia56/bodega)

- `bodega` is a git-native issue tracker for developers and AI agents.
- It positions itself as a hybrid of Beads and `ticket`, keeping a file-first workflow while using Git tracking in a
  dedicated branch.
- It includes dependency-aware commands such as ready, blocked, tree, cycle, and machine-readable query output.
- It also includes explicit sync-oriented commands such as push and sync.
- It is currently the closest external reference to tandem in spirit because it combines Git-native tickets, AI-oriented
  CLI usage, and coordination concerns.
- It is still not the tandem baseline as-is because tandem currently aims to make branch/worktree awareness and
  structured relevance output the primary feature rather than a side effect of ticket sync.

### [iloom](https://github.com/iloom-ai/iloom-cli)

- `iloom` positions itself as an “AI development control plane”, delivered as a CLI (`il`) and a VS Code extension.
- It orchestrates multi-agent workflows around an external issue tracker (GitHub/Linear), and persists agent output
  (analysis, plans, decisions, risks) into issue comments rather than repo-local ticket files.
- It creates per-task isolated environments (git worktrees, per-task runtime isolation such as ports, and optional DB
  branching) to make parallel work and context-switching practical.
- It is a useful reference for:
    - worktree-based isolation patterns for parallel agent work
    - structured, phase-oriented workflows (analyze → plan → implement) with human-in-the-loop review
    - machine-readable artifacts and recap/summarization UX
- It is not the tandem baseline because tandem is currently defined around repo-local ticket state, deterministic CLI
  semantics, and branch/worktree awareness over that state (rather than making an external issue tracker the primary
  datastore).

## Things intentionally not over-fixed yet

- Exact ticket field list
- Final status state machine
- Exact commands that trigger awareness checks
- Exact API surface beyond the CLI
