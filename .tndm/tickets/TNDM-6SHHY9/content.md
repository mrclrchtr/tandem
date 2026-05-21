## Overview
Add a first-class `supi_flow_apply` tool to move planned tickets into the apply phase and load the approved execution context. At the same time, tighten core lifecycle enforcement in the existing flow tools and align package guidance with the approved policy choices.

## Scope
- Add `supi_flow_apply { ticket_id }` as a focused transition+context tool.
- Always load the approved overview from `content.md` before apply starts.
- Keep task execution in the `supi-flow-apply` skill; the new tool does not execute or complete tasks.
- Require nonblank archive evidence before `supi_flow_close` and refuse to close when structured tasks remain incomplete.
- Mark mutating flow tools as sequential so PI does not race ticket mutations inside one assistant response.
- Update bundled docs and skills to describe the non-trivial-ticket policy and the new apply entrypoint.

## Assumptions
- This pass stays inside `plugins/supi-flow/` plus repo-level documentation; it does not add new `tndm` CLI commands.
- Coordinated workspace/package version bumps are deferred to release work; this change updates behavior, tests, and documentation only.

## File map
- `plugins/supi-flow/extensions/tools/flow-tools.ts` — add `supi_flow_apply`, shared helpers for reading ticket/task state, and close guards.
- `plugins/supi-flow/extensions/index.ts` — register `supi_flow_apply`, update tool guidance, and set sequential execution on mutating flow tools.
- `plugins/supi-flow/__tests__/flow-tools.test.ts` — unit coverage for apply transition/context loading and close guards.
- `plugins/supi-flow/__tests__/resources.test.ts` — tool registration count and new tool presence.
- `plugins/supi-flow/README.md` — workflow and tool documentation.
- `plugins/supi-flow/CLAUDE.md` — plugin maintainer guidance.
- `plugins/supi-flow/skills/supi-flow-brainstorm/SKILL.md` — non-trivial ticket policy wording.
- `plugins/supi-flow/skills/supi-flow-apply/SKILL.md` — start apply via `supi_flow_apply`, then read task detail docs as needed.
- `plugins/supi-flow/skills/supi-flow-archive/SKILL.md` — evidence-required closeout guidance.
- `CLAUDE.md` — root repo description of the plugin tool surface.

## Verification strategy
- Add targeted Vitest coverage for the new apply behavior and close guards.
- Finish with `cd plugins/supi-flow && pnpm exec tsc --noEmit && pnpm exec vitest run`.
