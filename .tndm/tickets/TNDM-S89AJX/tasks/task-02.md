# Task 2: Update supi_flow_apply prompt guidance and cover it with a registration test

## Goal
Make the registered `supi_flow_apply` prompt guidance match the lazy task-detail loading rule, and lock that behavior in with a focused registration test.

## Files
- `plugins/supi-flow/extensions/index.ts`
- `plugins/supi-flow/__tests__/resources.test.ts`

## TDD flow
1. In `plugins/supi-flow/__tests__/resources.test.ts`, expand the test harness so it can inspect the full registered tool object for `supi_flow_apply`, not just the tool names.
2. Add a focused assertion that `supi_flow_apply` prompt guidance tells the agent to load the approved overview and task manifest at the beginning of apply while deferring linked task-detail-doc reads until the active task starts.
3. Run the targeted test first and confirm it fails because the old prompt guidance is still too vague.
4. Update the `supi_flow_apply` `promptGuidelines` entry in `plugins/supi-flow/extensions/index.ts` so it expresses the new rule without changing the tool’s runtime behavior or description beyond what clarity requires.
5. Re-run the targeted tests and then `pnpm exec tsc --noEmit`.

## Verification
Run:

```bash
cd plugins/supi-flow
RTK_DISABLED=1 pnpm exec vitest run __tests__/index.test.ts __tests__/resources.test.ts -v
pnpm exec tsc --noEmit
```

Expected result:
- the targeted Vitest run passes, including the new `supi_flow_apply` registration assertion
- TypeScript type-checking passes with no errors
