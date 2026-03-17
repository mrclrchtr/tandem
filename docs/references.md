# Reference Points

Competitive analysis for tandem. See `docs/vision.md` for product goals.

## [Taskwarrior](https://github.com/GothenburgBitFactory/taskwarrior)

- Taskwarrior is a command line task list management utility with a large ecosystem of tools, hooks, and extensions.
- It is a useful reference for CLI UX, query/report ergonomics, and local-first task management workflows.
- It is not the tandem baseline because it is not git-aware and does not target multi-branch/worktree coordination,
  repo-local system-of-record state, or semantic conflict detection.

## [Beads](https://github.com/steveyegge/beads)

- Beads is a distributed, git-backed graph issue tracker for AI agents.
- It centers on a Dolt-powered architecture with version-controlled SQL, branching, and built-in sync.
- It already includes dependency-aware tracking, JSON output, claiming workflow, and auto-ready task detection.
- It is a useful reference for task graph behavior and machine-readable CLI design.
- It is not the tandem baseline because tandem is currently defined around repo-local ticket files and branch/worktree
  awareness as a first-class feature.

## [ticket](https://github.com/wedow/ticket)

- `ticket` is a git-backed issue tracker for AI agents.
- It stores tickets as markdown files with YAML frontmatter in `.tickets/`.
- It supports dependency tracking, ready/blocked views, JSON query output, and a plugin system.
- It is closer to tandem than Beads because it is repo-native, file-based, and lightweight.
- It is still not the tandem baseline as-is because tandem currently aims for:
    - per-ticket directories instead of single markdown files
    - stricter separation of stable ticket metadata and volatile state
    - branch/worktree awareness as a core feature rather than a later add-on
    - semantic awareness and conflict categories as part of the design

## [bodega](https://github.com/bjia56/bodega)

- `bodega` is a git-native issue tracker for developers and AI agents.
- It positions itself as a hybrid of Beads and `ticket`, keeping a file-first workflow while using Git tracking in a
  dedicated branch.
- It includes dependency-aware commands such as ready, blocked, tree, cycle, and machine-readable query output.
- It also includes explicit sync-oriented commands such as push and sync.
- It is currently the closest external reference to tandem in spirit because it combines Git-native tickets, AI-oriented
  CLI usage, and coordination concerns.
- It is still not the tandem baseline as-is because tandem currently aims to make branch/worktree awareness and
  structured relevance output the primary feature rather than a side effect of ticket sync.

## [iloom](https://github.com/iloom-ai/iloom-cli)

- `iloom` positions itself as an "AI development control plane", delivered as a CLI (`il`) and a VS Code extension.
- It orchestrates multi-agent workflows around an external issue tracker (GitHub/Linear), and persists agent output
  (analysis, plans, decisions, risks) into issue comments rather than repo-local ticket files.
- It creates per-task isolated environments (git worktrees, per-task runtime isolation such as ports, and optional DB
  branching) to make parallel work and context-switching practical.
- It is a useful reference for:
    - worktree-based isolation patterns for parallel agent work
    - structured, phase-oriented workflows (analyze, plan, implement) with human-in-the-loop review
    - machine-readable artifacts and recap/summarization UX
- It is not the tandem baseline because tandem is currently defined around repo-local ticket state, deterministic CLI
  semantics, and branch/worktree awareness over that state (rather than making an external issue tracker the primary
  datastore).
