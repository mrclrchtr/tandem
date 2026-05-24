# Task 4: Verify Phase 1: tsc + all existing tests pass

Run the full test suite and type-check after Phase 1 changes:

```sh
cd plugins/supi-flow
pnpm exec tsc --noEmit
pnpm exec vitest run
```

All 51 existing tests must pass. No new TypeScript errors. Verify the test output shows `PASS (51) FAIL (0)`.

Additionally, do a quick sanity check: count lines of code across `tndm-cli.ts` + `flow-tools.ts` before and after. The combined line count should decrease by ~120-150 lines (the extracted duplication).

**Verification**: `pnpm exec tsc --noEmit` returns zero errors; `pnpm exec vitest run` shows all 51 tests passing.
