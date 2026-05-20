# supi-flow

PI extension for spec-driven workflow with TNDM ticket coordination (optional for trivial changes).

## Flow

```mermaid
flowchart TD
    START(["Start a change"]) --> BRAIN
    BRAIN["/skill:supi-flow-brainstorm
         HARD-GATE: no code yet
         Explore, design, approve
         Classify trivial vs non-trivial"]
    BRAIN --> APPROVED{Design approved?}
    APPROVED -->|"No"| BRAIN
    APPROVED -->|"Yes"| TRIVIAL{Trivial change?}

    TRIVIAL -->|"Yes (skip ticket)"| LIGHT["Implement directly
         No ticket needed"]
    TRIVIAL -->|"No"| PLAN

    PLAN["/skill:supi-flow-plan [ID]
         Bite-sized tasks
         Exact file paths
         No placeholders
         TDD: red-green-refactor
         Stores plan via supi_flow_plan"]
    PLAN --> APPROVE2{"Plan approved?"}
    APPROVE2 -->|"No"| PLAN
    APPROVE2 -->|"Yes"| APPLY

    APPLY["/skill:supi-flow-apply [ID]
         Iron Law: fresh verify each task
         TDD gate: test-first or delete
         Sets flow:applying at start
         Checks off tasks via supi_flow_complete_task"]
    APPLY --> BLOCKED{"Verification
         failed?"}

    BLOCKED -->|"Yes"| DEBUG["/skill:supi-flow-debug
         4-phase systematic debugging
         3-fix → question architecture"]
    DEBUG --> FIXED{Fixed?}
    FIXED -->|"Yes"| APPLY
    FIXED -->|"No"| USER["Talk to user
         before fix #4"]

    BLOCKED -->|"No"| DONE{"All tasks
         done?"}
    DONE -->|"No"| APPLY
    DONE -->|"Yes"| ARCHIVE

    ARCHIVE["/skill:supi-flow-archive [ID]
         Fresh verification (gate function)
         Update living documentation
         Quality gate checklist"]
    ARCHIVE --> QGATE{"Quality gate
         passes?"}
    QGATE -->|"No"| ARCHIVE
    QGATE -->|"Yes"| CLOSE

    CLOSE["supi_flow_close
         Sets status=done, flow:done
         Writes archive.md"]

    classDef phase fill:#e8f5e9,stroke:#4caf50,stroke-width:2
    classDef decision fill:#e3f2fd,stroke:#2196f3
    classDef entry fill:#e8e8e8,stroke:#666
    classDef blocker fill:#ffebee,stroke:#f44336

    class BRAIN,PLAN,APPLY,ARCHIVE,CLOSE phase
    class APPROVED,APPROVE2,BLOCKED,FIXED,DONE,TRIVIAL decision
    class START entry
    class USER blocker
    class LIGHT entry
```

Non-trivial flows require a TNDM ticket created by `supi_flow_start`. Trivial changes can be implemented directly without a ticket.

## Skills

Five skills ship under `skills/`:

| Skill | Trigger | Purpose |
|---|---|---|
| `supi-flow-brainstorm` | `/supi-flow-brainstorm` | Explore intent and design, classify trivial vs non-trivial, create ticket if needed |
| `supi-flow-plan` | `/supi-flow-plan [ID]` | Create bite-sized implementation plan |
| `supi-flow-apply` | `/supi-flow-apply` | Execute plan task by task |
| `supi-flow-archive` | `/supi-flow-archive` | Verify, update docs, close out |
| `supi-flow-debug` | Loaded on demand when blocked | Root-cause debugging protocol |

## Tools

Five custom tools registered by the extension:

| Tool | Purpose |
|---|---|
| `supi_tndm_cli` | Thin wrapper around the `tndm` CLI with action enum (create/update/show/list/awareness) |
| `supi_flow_start` | Create a ticket with status=todo, tag=flow:brainstorm, and optional design context in `content.md` |
| `supi_flow_plan` | Store the approved overview / plan in `content.md` and move the ticket to `flow:planned` |
| `supi_flow_complete_task` | Check off a numbered task in the structured task manifest stored in `state.toml` |
| `supi_flow_close` | Mark done and write verification results to `archive.md` |

Tools should be used instead of calling `tndm` via bash. The agent invokes them with structured parameters.

## Ticket documents

`supi-flow` uses TNDM's registered document model with one canonical ticket body plus execution-time attachments:

- `content.md`: approved overview / design / plan prose
- structured tasks in `state.toml`: execution manifest used during `/supi-flow-apply`
- optional task docs in `tasks/`: linked task detail for tasks that need more than a headline/files/verification/notes
- `archive.md`: final verification evidence written during `/supi-flow-archive`

Older tickets may still contain a legacy brainstorm sidecar document, but new flow work should not create or depend on it.

## Overview-first workflow

`content.md` is overview-first and may contain zero tasks. After the overview is approved and stored, create the executable task list separately in `state.toml`.

Use headline-only tasks when the title is enough. If a task needs real implementation detail or notices, attach an optional `tasks/task-XX.md` task doc after the task already exists in the manifest.

## Prompt templates

| Prompt | Description |
|---|---|
| `/supi-coding-retro` | Retrospective on project setup, architecture, tooling, workflows, and conventions |

## Ticket flow phase tracking

Flow phases map to TNDM statuses and tags:

| Flow phase | Status | Tags |
|---|---|---|
| Brainstorm | `todo` | `flow:brainstorm` |
| Plan written | `todo` | `flow:planned` |
| Implementing | `in_progress` | `flow:applying` |
| Done | `done` | `flow:done` |

## Dependencies

- **tndm CLI** (`tandem-cli`): required (all ticket operations shell out to `tndm`)

  ```sh
  brew install mrclrchtr/tap/tandem-cli
  ```

- **pi**: discovers bundled skills and prompt templates automatically from the package

## PI package

This extension is published as a [`pi-package`](https://pi.dev/packages) — listed in the PI package gallery. Install directly:

```bash
pi install npm:@mrclrchtr/supi-flow
```

## Installation

The extension is auto-discovered when the plugin directory is in pi's extension search path:

```bash
# Option 1: symlink
ln -s "$(pwd)/plugins/supi-flow" ~/.pi/agent/extensions/supi-flow

# Option 2: settings.json
# Add to ~/.pi/agent/settings.json:
# { "extensions": ["./plugins/supi-flow/extensions/index.ts"] }
```

## Development

```bash
cd plugins/supi-flow
pnpm install

# Type-check
pnpm exec tsc --noEmit

# Run tests
pnpm exec vitest run
```
