# Overview

Reduce the always-on prompt surface for `plugins/supi-flow/` without losing guidance coverage. Keep all 7 tools active and keep every `promptSnippet`, but rewrite the always-on tool contract so it only carries routing and hard guardrails. Detailed phase procedure should remain in the existing phase skills instead of the always-on tool metadata.

## Scope

- Prompt-contract change only inside `plugins/supi-flow/`.
- No tool availability, lifecycle behavior, or runtime logic changes.
- No measurement or instrumentation work.

## Files to modify

- `plugins/supi-flow/extensions/tools/tool-specs.ts`
  - Shorten each tool's `description`, `promptSnippet`, and `promptGuidelines` to routing + guardrail language while preserving the core lifecycle and safety distinctions.
- `plugins/supi-flow/extensions/tools/tndm-cli.ts`
  - Compress TypeBox parameter descriptions for `supi_tndm_cli`, keeping only the distinctions the model actually needs (for example: `tags` replaces, `add_tags`/`remove_tags` mutate, `task_number` is 1-based).
- `plugins/supi-flow/extensions/tools/flow-tools.ts`
  - Compress flow-tool parameter descriptions while keeping document-role distinctions explicit (`context`, `plan_content`, `detail`, `verification_results`).
- `plugins/supi-flow/__tests__/resources.test.ts`
  - Tighten tests around the semantic prompt contract so future cleanup does not accidentally drop critical guidance.
- Optional only if a removed always-on rule has no surviving home:
  - relevant `plugins/supi-flow/skills/*/SKILL.md` file(s)
  - Add the smallest clarification needed so the detailed procedure still exists on demand.

## Design rules

- Keep all seven tools active.
- Keep every `promptSnippet`.
- Always-on tool text should answer only:
  1. what the tool does,
  2. when to use it,
  3. which hard precondition or guardrail must not be forgotten.
- Remove repetitive procedural wording from always-on tool metadata.
- Preserve information coverage even when wording is shortened or relocated.
- Prefer the existing phase skills as the home for detailed procedure.
- Do not change runtime behavior, tool names, or tool availability.

## Guidance that must remain represented

- `supi_tndm_cli` is the direct structured `tndm` wrapper and should be used instead of running `tndm` via bash.
- `supi_flow_start` is for non-trivial work and must not be used when the user explicitly requests direct implementation.
- `supi_flow_plan` stores the approved overview; structured tasks are authored separately.
- `supi_flow_apply` is the implementation entrypoint that loads the overview and task manifest and enforces apply lifecycle expectations.
- `supi_flow_task` is the normal task-authoring path over low-level task escape hatches.
- `supi_flow_complete_task` is used after verification and takes the task number.
- `supi_flow_close` is the final closeout step and requires verification evidence.

## Verification strategy

- `cd plugins/supi-flow && pnpm exec vitest run __tests__/resources.test.ts`
- `cd plugins/supi-flow && pnpm exec vitest run __tests__/index.test.ts __tests__/resources.test.ts`
- `cd plugins/supi-flow && pnpm exec tsc --noEmit`
- Manual review that any guidance removed from always-on tool metadata still exists in the appropriate skill file if it was intentionally relocated.

## Non-goals

- No dynamic tool activation.
- No tool removal or snippet removal.
- No runtime behavior changes.
- No measurement or instrumentation work.
- No broad README or CLAUDE cleanup unless it is required to preserve moved guidance.
