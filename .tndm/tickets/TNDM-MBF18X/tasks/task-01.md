# Task 1: Define the compact always-on prompt contract in registration tests before changing tool copy

# Goal

Define the slimmer always-on `supi-flow` prompt contract in tests before editing the tool-facing copy.

## Files

- `plugins/supi-flow/__tests__/resources.test.ts`

## Change

- Extend the registered-tool test surface so the test can inspect:
  - tool `description`
  - `promptSnippet`
  - `promptGuidelines`
  - selected TypeBox parameter descriptions
- Add focused assertions for the intended compact contract. Use exact or near-exact checks only for text that is meant to stay stable after this cleanup.
- Cover at least these guarantees:
  - all 7 tools still register and keep `promptSnippet`
  - `supi_tndm_cli` still routes direct `tndm` work away from `bash`
  - `supi_flow_start` still blocks explicit direct-implementation requests
  - `supi_flow_plan` still separates overview persistence from task authoring
  - `supi_flow_task` remains the normal task-authoring path
  - `supi_flow_complete_task` still uses the task number
  - `supi_flow_close` still requires verification-evidence semantics
  - compressed parameter descriptions still preserve the required distinctions for `tags`, `add_tags`, `remove_tags`, `task_number`, `plan_content`, and `verification_results`
- Keep the assertions aligned to the new compact contract instead of the current verbose copy.

## Verification

1. Run:
   ```sh
   cd plugins/supi-flow
   pnpm exec vitest run __tests__/resources.test.ts
   ```
2. Expected result before the implementation task: the new compact-contract assertions fail because the current tool metadata and parameter descriptions have not been updated yet.

## Test strategy

- Test-driven. Write the failing registration assertions first and confirm they fail for the intended prompt-contract mismatch before changing implementation.
