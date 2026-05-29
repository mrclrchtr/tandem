# Task 2: Rewrite supi-flow tool metadata and schema descriptions to routing + guardrails only

# Goal

Shrink the always-on `supi-flow` prompt surface without losing guidance coverage by rewriting tool metadata and parameter descriptions as routing + hard guardrail text only.

## Files

- `plugins/supi-flow/extensions/tools/tool-specs.ts`
- `plugins/supi-flow/extensions/tools/tndm-cli.ts`
- `plugins/supi-flow/extensions/tools/flow-tools.ts`
- Optional only if a removed always-on rule has no surviving on-demand home:
  - relevant `plugins/supi-flow/skills/*/SKILL.md`

## Change

- In `plugins/supi-flow/extensions/tools/tool-specs.ts`:
  - shorten every tool `description`
  - keep every `promptSnippet`, but shorten each one to the smallest clear routing phrase
  - trim `promptGuidelines` down to routing and hard guardrails only
- Preserve the important distinctions while removing procedural detail from the always-on metadata:
  - `supi_tndm_cli` is the direct `tndm` wrapper over `bash`
  - `supi_flow_start` is for non-trivial work and must not be used when the user explicitly wants direct implementation
  - `supi_flow_plan` stores the overview and leaves task authoring separate
  - `supi_flow_apply` is the apply entrypoint and keeps lifecycle semantics, but detailed phase procedure should live in the apply skill
  - `supi_flow_task` remains the preferred normal task-authoring path
  - `supi_flow_complete_task` stays tied to verified task completion by number
  - `supi_flow_close` stays tied to final closeout with verification evidence
- In `plugins/supi-flow/extensions/tools/tndm-cli.ts` and `plugins/supi-flow/extensions/tools/flow-tools.ts`:
  - compress TypeBox field descriptions wherever the field name already carries meaning
  - keep extra wording only where it prevents ambiguity, especially for:
    - `tags` vs `add_tags` / `remove_tags`
    - `task_number` being 1-based
    - `content` vs `plan_content` vs `detail` vs `verification_results`
- Only touch a skill file if a removed always-on rule no longer has a clear on-demand home there; if so, add the smallest clarification needed instead of re-expanding the always-on tool metadata.

## Verification

1. Run:
   ```sh
   cd plugins/supi-flow
   pnpm exec vitest run __tests__/resources.test.ts
   ```
2. Expected result after the implementation: the updated registration test passes with the compact contract and preserved semantic distinctions.

## Test strategy

- Test-driven via the registration contract added in Task 1.
