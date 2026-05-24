# Task 1: Centralize supi-flow tool specs and registration

## Goal
Create a single source of truth for all seven public `supi-flow` tools and make `plugins/supi-flow/extensions/index.ts` a thin PI adapter without changing public tool names, version-check behavior, or the existing flow-skill guidance.

## Files
- `plugins/supi-flow/extensions/index.ts`
- `plugins/supi-flow/extensions/tools/tool-specs.ts`
- `plugins/supi-flow/__tests__/resources.test.ts`
- `plugins/supi-flow/__tests__/index.test.ts`

## Test strategy
**Test-driven.** Tighten the registration tests before refactoring implementation.

## Change to make
1. Add `plugins/supi-flow/extensions/tools/tool-specs.ts` as the single ordered definition list for the seven tools.
2. Put each tool's `name`, `label`, `description`, `promptSnippet`, `promptGuidelines`, `executionMode`, parameter schema, and execute binding in that module.
3. Refactor `plugins/supi-flow/extensions/index.ts` so it keeps only the startup/reload version-check wiring plus a loop that registers tools from the shared definitions.
4. Preserve the existing public guidance strings unless a later runtime-hardening task requires a narrowly scoped wording change.
5. Keep `supi_tndm_cli` sequential in the shared metadata so ordered mutation behavior is encoded in one place.

## RED
- Extend `plugins/supi-flow/__tests__/resources.test.ts` so it asserts:
  - all 7 tools still register
  - `supi_tndm_cli` is registered with sequential execution mode
  - `supi_flow_apply` guidance still mentions deferring task-detail reads until task start
- Extend `plugins/supi-flow/__tests__/index.test.ts` only as needed to keep the version-check session-start registration stable after the refactor.
- Run the focused tests and watch them fail for the expected missing-metadata reasons before changing the implementation.

## GREEN
- Implement `plugins/supi-flow/extensions/tools/tool-specs.ts`.
- Refactor `plugins/supi-flow/extensions/index.ts` to register from the shared definitions and keep `checkTndmVersion()` intact.
- Make the tests pass without changing the public tool surface.

## REFACTOR
- Remove duplicated inline tool-registration literals from `plugins/supi-flow/extensions/index.ts`.
- Keep the PI adapter thin; do not move CLI or flow business logic out of `plugins/supi-flow/extensions/tools/*.ts`.

## Verification
```sh
cd plugins/supi-flow
pnpm exec vitest run __tests__/index.test.ts __tests__/resources.test.ts
```
Expected result: both test files pass, tool registration remains at 7 tools, and `supi_tndm_cli` is locked to sequential execution via the shared spec module.
