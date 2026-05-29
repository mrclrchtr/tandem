# Task 3: Run final supi-flow verification and review the diff for lost guidance

# Goal

Confirm that the prompt-contract cleanup is complete, type-safe, and did not silently drop required guidance.

## Files

- Inspect the final diff for:
  - `plugins/supi-flow/__tests__/resources.test.ts`
  - `plugins/supi-flow/extensions/tools/tool-specs.ts`
  - `plugins/supi-flow/extensions/tools/tndm-cli.ts`
  - `plugins/supi-flow/extensions/tools/flow-tools.ts`
  - any intentionally touched `plugins/supi-flow/skills/*/SKILL.md` file

## Change

- No new implementation is expected in this task.
- Run the focused supi-flow verification sweep.
- Review the final diff and confirm that any guidance removed from always-on tool metadata still exists in the appropriate skill file when it was intentionally relocated.

## Verification

1. Run:
   ```sh
   cd plugins/supi-flow
   pnpm exec tsc --noEmit
   pnpm exec vitest run __tests__/index.test.ts __tests__/resources.test.ts
   git diff -- plugins/supi-flow
   ```
2. Expected result:
   - TypeScript type-check passes.
   - The targeted tests pass.
   - The diff shows only the intended prompt-contract/test updates plus any minimal skill clarifications needed to preserve moved guidance.

## Test strategy

- Test-exempt final verification task.
