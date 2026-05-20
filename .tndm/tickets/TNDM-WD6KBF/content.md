## Problem

The new structured-task feature mixes three concepts in one authoring flow:

1. `content.md` as the ticket overview / approved design
2. `state.toml` as the executable task manifest
3. optional per-task markdown docs under `tasks/`

That makes the workflow harder to predict. In particular, the current shape exposes raw `detail_path` semantics too early, allows dangling or mismatched task-doc references, and encourages users to think of the plan parser as both overview storage and task materialization.

## Goal

Redesign the feature so it is simpler, easier, and more predictable:

- `content.md` is the overview / plan and may contain zero tasks.
- `state.toml` is the only place where tasks exist.
- Headline-only tasks live entirely in `state.toml`.
- Tasks with real implementation detail or notices may attach an optional task doc under `tasks/`.
- The workflow is hybrid: tools create and link task docs, then humans/agents can edit the markdown directly.
- Task docs cannot create tasks on their own.

## Recommended approach

Use an overview-first, manifest-first model.

### Source-of-truth rules

- `content.md` stores the approved overview / design.
- `state.toml.tasks` is the execution manifest.
- `tasks/task-XX.md` is an optional attachment referenced by a task.
- A task exists only if it exists in `state.toml`.
- Task docs enrich existing tasks; they never implicitly create tasks.

### Authoring rules

- If a task is just a headline, keep it in `state.toml` only.
- If a task has real implementation detail or notices, create and link a task doc.
- Normal users/agents should not have to reason about raw `detail_path` values.
- Tools should provide predictable helpers for creating/updating a task together with its optional detail doc.

### UX consequences

- `supi_flow_plan` should no longer be responsible for parsing `**Task N**` blocks into `state.toml`; it should focus on storing the approved overview in `content.md`.
- Task creation/editing becomes a separate, explicit workflow after the overview exists.
- Apply-phase behavior remains simple: read tasks from `state.toml`, then read a linked task doc only when one exists.
- Task/doc creation and linkage must be atomic so users never end up with dangling references or mismatched manifest/doc paths.

## Constraints / non-goals

- No backward compatibility is required for this new feature direction.
- Do not preserve the current mixed overview+task parser if it makes the UX harder to explain.
- Do not allow task docs to become a second source of truth.
- Avoid exposing low-level linkage details in the common path unless they are necessary for advanced/manual use.

## Acceptance

- The mental model can be explained as: overview in `content.md`, tasks in `state.toml`, optional detail in `tasks/`.
- The common workflow does not require users to manage raw `detail_path` manually.
- Creating or editing a detailed task cannot leave behind a broken task/doc link.
- The tool and skill guidance clearly separates overview planning from task execution manifest management.
- Apply-phase instructions remain deterministic about where task detail must be read from.

## Open questions resolved

- `content.md` is overview-first and may contain no tasks.
- Task docs are optional and only for tasks with real detail.
- The workflow is hybrid: tools create/link docs; humans and agents may edit markdown afterward.
- `state.toml` remains the sole source of truth for task existence.
