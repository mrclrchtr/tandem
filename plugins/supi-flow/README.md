# supi-flow

> **PI-only workflow package for spec-driven changes backed by TNDM tickets.**

`supi-flow` adds a lightweight workflow on top of tandem's `tndm` CLI:
**brainstorm → plan → apply → archive**.

It is published as `@mrclrchtr/supi-flow` and ships inside the tandem repository at
`plugins/supi-flow/`.

Use it when a change needs:

- an approved design before implementation
- a durable TNDM ticket for non-trivial work
- explicit task-by-task verification during implementation
- archived verification evidence at closeout

Trivial changes can still be implemented directly without a ticket.

## What ships in the package

`supi-flow` uses PI's conventional package directories, so PI auto-discovers the resources in:

- `extensions/`
- `skills/`
- `prompts/`

Current package contents:

- **7 custom tools**
  - `supi_tndm_cli`
  - `supi_flow_start`
  - `supi_flow_plan`
  - `supi_flow_apply`
  - `supi_flow_task`
  - `supi_flow_complete_task`
  - `supi_flow_close`
- **5 skills**
  - `supi-flow-brainstorm`
  - `supi-flow-plan`
  - `supi-flow-apply`
  - `supi-flow-archive`
  - `supi-flow-debug`
- **1 prompt template**
  - `/supi-coding-retro`
- **Startup/reload version check**
  - on PI session start and reload, the extension compares `tndm --version` with the package version and warns when they do not match

This package does **not** rely on a `pi` manifest in `package.json`; it uses PI's conventional directory discovery.

## Installation and loading

### Install from npm

```bash
pi install npm:@mrclrchtr/supi-flow
```

### Install from a local checkout

From the tandem repository root:

```bash
pi install ./plugins/supi-flow
```

Or add the package root to PI settings:

```json
{
  "packages": ["./plugins/supi-flow"]
}
```

### Important: prefer the package root, not just the extension entrypoint

If you load only `plugins/supi-flow/extensions/index.ts`, PI gets the extension entrypoint,
but not the package-style resource loading for the bundled skills and prompt template.

Use the **package root** when you want the full package:

- extension tools
- auto-discovered skills
- auto-discovered prompt template

### Dependency: `tndm`

`supi-flow` wraps the tandem CLI and expects `tndm` to be installed and on your `PATH`.
Keep `tndm` and `@mrclrchtr/supi-flow` on matching release versions so the startup/reload
version check stays quiet.

See the tandem project README for CLI install options:
<https://github.com/mrclrchtr/tandem>

## How you use it in PI

This package is primarily used through:

- **skills** via `/skill:<name>`
- **tools** invoked by the model
- **prompt template** `/supi-coding-retro`

There is no custom `/supi-flow` slash command registered by the extension.

### Typical workflow

1. **Brainstorm** — `/skill:supi-flow-brainstorm`
   - clarify intent
   - inspect the codebase
   - compare approaches
   - approve a design before editing
   - decide whether the change is trivial or non-trivial

2. **Plan** — `/skill:supi-flow-plan TNDM-XXXXXX`
   - store the approved overview in `content.md`
   - author executable tasks one at a time in `state.toml` via `supi_flow_task`
   - when revising an existing ticket, list the current tasks first and reconcile them with edit/remove/add instead of blindly appending new ones
   - keep tasks concrete, ordered, and verifiable

3. **Apply** — `/skill:supi-flow-apply TNDM-XXXXXX`
   - start with `supi_flow_apply` to load the approved overview and task manifest
   - transition planned tickets into `flow:applying`
   - resume already-applying tickets with their current `in_progress` or `blocked` status intact
   - execute tasks in order
   - run fresh verification for each task
   - check off tasks with `supi_flow_complete_task`

4. **Debug when blocked** — `/skill:supi-flow-debug`
   - use the root-cause workflow instead of guessing

5. **Archive** — `/skill:supi-flow-archive TNDM-XXXXXX`
   - re-verify the completed change
   - update living docs if needed
   - close the ticket with required archived verification evidence

## Flow phases

| Phase | Main skill | Ticket behavior |
|---|---|---|
| Brainstorm | `supi-flow-brainstorm` | creates or refines the change definition; non-trivial work starts with `supi_flow_start` |
| Plan | `supi-flow-plan` | stores the approved overview in `content.md`, then authors structured tasks one at a time via `supi_flow_task` |
| Apply | `supi-flow-apply` | starts with `supi_flow_apply`, loads the approved overview plus task manifest, transitions to `flow:applying` when needed, preserves the current `in_progress` or `blocked` status for already-applying tickets, then executes tasks and verifies each step fresh |
| Archive | `supi-flow-archive` | verifies the final result, writes `archive.md`, and closes the ticket |

Flow state is tracked with TNDM status/tag combinations:

| Flow phase | Status | Tags |
|---|---|---|
| Brainstorm | `todo` | `flow:brainstorm` |
| Plan written | `todo` | `flow:planned` |
| Implementing | `in_progress` | `flow:applying` |
| Paused during apply | `blocked` | `flow:applying` |
| Done | `done` | `flow:done` |

## Ticket document model

`supi-flow` uses tandem's registered document model with an overview-first workflow:

| Artifact | Role |
|---|---|
| `content.md` | Canonical approved overview / design / plan prose |
| `state.toml` tasks | Structured execution manifest used during apply |
| `tasks/task-XX.md` | Canonical task detail doc — every task gets one automatically at creation time |
| `archive.md` | Final verification evidence written during archive/closeout |

Key rules from the current implementation:

- `content.md` is **overview-first** and may contain zero tasks.
- Executable tasks live in `state.toml`, not in checklist blocks parsed from markdown.
- Every task gets a canonical `tasks/task-XX.md` detail doc automatically at creation time.
- The common plan-time task-authoring path is `supi_flow_task`; low-level `task_*` actions remain available as escape hatches.
- When revising a ticket that already has tasks, list the current manifest first and reconcile it with `edit` / `remove` / `add` operations instead of treating replanning as repeated append-only adds.
- Older tickets may still contain a legacy brainstorm sidecar document, but new flow work should not depend on it.

## Tools

The extension registers seven custom tools.

| Tool | What it does |
|---|---|
| `supi_tndm_cli` | Structured wrapper around `tndm` for ticket create/update/show/list/awareness plus lower-level task add/list/complete/remove/edit/set actions |
| `supi_flow_start` | Creates a ticket with `status=todo` and tag `flow:brainstorm`, optionally persisting initial context into `content.md` |
| `supi_flow_plan` | Stores the approved overview in `content.md` and replaces flow-state tags with `flow:planned` |
| `supi_flow_apply` | Loads the approved overview from `content.md`, returns the structured task manifest, transitions `flow:planned` tickets into `status=in_progress` with `flow:applying`, and preserves the current `in_progress` or `blocked` status for already-applying tickets |
| `supi_flow_task` | Adds, edits, or removes one structured task at a time and optionally manages the canonical `tasks/task-XX.md` detail doc; use it to reconcile existing task manifests during replans as well as to create new ones |
| `supi_flow_complete_task` | Marks one numbered task as done in the structured task manifest |
| `supi_flow_close` | Requires verification evidence, refuses to close unless the ticket is in `flow:applying` with a non-empty all-done structured task list, writes `archive.md`, syncs documents, and closes the ticket with `status=done` and `flow:done` |

### `supi_tndm_cli` at a glance

`supi_tndm_cli` is intentionally thinner than the flow skills. Use `supi_flow_task` for the normal plan-time path to author tasks one at a time. Reach for `supi_tndm_cli` when you need direct ticket operations or lower-level task repair.

Current action groups:

- **ticket actions** — `create`, `update`, `show`, `list`, `awareness`
- **task actions** — `task_add`, `task_list`, `task_complete`, `task_remove`, `task_edit`, `task_set`
  - these are lower-level escape hatches; normal plan-time task authoring should prefer `supi_flow_task`

Task-detail behavior worth knowing:

- every `task_add` automatically creates the canonical `tasks/task-XX.md` detail doc via the Rust CLI
- when `task_detail` is provided, the tool writes the full markdown body and runs `tndm ticket sync`
- `task_edit` can also write updated task detail docs and clear file lists

## Skills

The package ships five skills under `skills/`.

| Skill | Use it for |
|---|---|
| `supi-flow-brainstorm` | Clarify intent, inspect context, compare approaches, and get approval before implementation |
| `supi-flow-plan` | Turn the approved design into an executable plan with exact files and verification |
| `supi-flow-apply` | Execute the approved plan task by task with fresh verification gates |
| `supi-flow-archive` | Re-verify the completed change, update living docs, and close the ticket |
| `supi-flow-debug` | Systematic root-cause debugging when verification fails or the cause is unclear |

## Prompt template

| Prompt | Purpose |
|---|---|
| `/supi-coding-retro` | Retrospective on setup, architecture, tooling, workflow, and conventions |

## Package layout

```text
plugins/supi-flow/
├── extensions/
│   ├── index.ts
│   ├── cli.ts
│   └── tools/
├── skills/
│   ├── supi-flow-brainstorm/
│   ├── supi-flow-plan/
│   ├── supi-flow-apply/
│   ├── supi-flow-archive/
│   └── supi-flow-debug/
├── prompts/
│   └── supi-coding-retro.md
└── __tests__/
```

## Development

```bash
cd plugins/supi-flow
pnpm install

# Type-check
pnpm exec tsc --noEmit

# Run the full test suite
pnpm exec vitest run
```

Useful targeted checks from the current package guidance:

```bash
# Version-check / tool registration behavior
pnpm exec vitest run __tests__/index.test.ts __tests__/resources.test.ts

# CLI wrapper behavior
pnpm exec vitest run __tests__/cli.test.ts

# Flow tools
pnpm exec vitest run __tests__/flow-tools.test.ts

# TNDM CLI tool behavior
pnpm exec vitest run __tests__/tndm-cli-tool.test.ts
```

## Validation notes while developing

- After changing `extensions/`, `skills/`, or `prompts/`, use `/reload` or restart PI before validating behavior.
- Use the plugin source and tests as the source of truth for README claims.
- Keep runtime PI packages in `peerDependencies` with `"*"` ranges and non-PI runtime deps in `dependencies`.
- Do not add `resources_discover` for `skills/` or `prompts/`; PI already auto-discovers them from the conventional directories.
