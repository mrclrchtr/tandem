# Task 5: Run full test suite — verify zero regressions

## Goal

After tasks 1-4, run the full unit test suite and verify zero failures. Tasks 2 and 3 already clean up their test files — this is the cross-cutting safety check.

## Verification

```sh
cd plugins/supi-flow
pnpm exec tsc --noEmit && pnpm exec vitest run
```

Expected: all tests pass, zero failures, zero warnings.

If any test fails:
1. Check if it's a stale assertion from an unexpected source file
2. Fix it — do NOT skip or mute it
3. Re-run full suite

## Dependencies

Depends on Tasks 1-4 being complete. This is the integration gate before moving to Tasks 6-7.
