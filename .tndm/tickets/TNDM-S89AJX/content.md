# Overview

Align `supi-flow` apply guidance so agents still load the approved overview and full structured task manifest at apply start, but do **not** eagerly read every linked task detail doc. Task docs referenced by `detail_path` should be read only when the corresponding task becomes active.

## Scope

This is a guidance-only change. It updates agent-facing instructions and supporting documentation without changing `supi_flow_apply` runtime behavior, task-manifest structure, or ticket lifecycle behavior.

## Files to modify

- `plugins/supi-flow/skills/supi-flow-apply/SKILL.md`
  - Clarify the apply-start sequence: load overview + manifest first, defer task-detail-doc reads until the current task starts.
- `plugins/supi-flow/README.md`
  - Align the documented apply workflow and tool behavior wording with the lazy task-detail loading rule.
- `plugins/supi-flow/extensions/index.ts`
  - Update `supi_flow_apply` prompt guidance so the system-prompt-facing instruction matches the skill and README.
- `plugins/supi-flow/__tests__/resources.test.ts`
  - Add a focused registration assertion that preserves the new `supi_flow_apply` prompt-guidance contract.

## Implementation notes

- Keep the `supi_flow_apply` tool description centered on actual tool behavior: loading `content.md`, returning the structured task manifest, and handling flow-state transitions.
- Put the lazy task-detail loading rule in the places that instruct agent behavior: the apply skill, the README workflow guidance, and the registered prompt guidelines.
- In the apply skill, make the distinction explicit between:
  - reviewing the overview and full manifest before starting work
  - reading a task's `detail_path` only when that specific task begins
- In the README, keep the workflow wording short but unambiguous so it matches the skill.
- In the registration test, assert the exact guidance intent rather than relying only on tool-name presence.

## Verification strategy

- Manual doc review for the markdown guidance changes to confirm the wording consistently distinguishes upfront overview/manifest loading from per-task detail-doc loading.
- Targeted plugin verification after the TypeScript change:

```bash
cd plugins/supi-flow
RTK_DISABLED=1 pnpm exec vitest run __tests__/index.test.ts __tests__/resources.test.ts -v
pnpm exec tsc --noEmit
```

## Non-goals

- No change to `plugins/supi-flow/extensions/tools/flow-tools.ts`
- No change to `plugins/supi-flow/CLAUDE.md` unless contradictory wording is discovered during editing
- No change to task numbering, `detail_path` generation, or archive/closeout behavior
