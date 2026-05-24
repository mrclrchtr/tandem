# Task 4: Remove _repoRoot module cache from ticket-helpers.ts

## Goal

Remove the module-level `_repoRoot` cache and its `_resetRepoRootCache` export. `findRepoRoot()` always walks the directory tree (at most 10-15 `existsSync` checks — negligible overhead). The cached value is process-lifetime and risks staleness if PI ever changes directories across sessions.

## Files

- `extensions/tools/ticket-helpers.ts` — remove cache, remove reset export
- `__tests__/ticket-helpers.test.ts` — update caching test

## Changes

### ticket-helpers.ts

1. Remove `let _repoRoot: string | null = null;`
2. Remove the `if (_repoRoot !== null) return _repoRoot;` guard
3. Remove `export function _resetRepoRootCache(): void { _repoRoot = null; }`
4. `findRepoRoot` body now starts directly with `let current = resolve(startDir);`

### Test updates

1. Remove `_resetRepoRootCache` imports and calls from `ticket-helpers.test.ts`
2. Update the caching test: instead of checking for stale cache, validate that two calls to `findRepoRoot` return the same correct path (both walk independently, both find the same root)

## Verification (TDD)

RED: Remove the cache test (it tests memoization, which no longer exists). Update the remaining `findRepoRoot` tests.
GREEN: Remove cache from source.
Verify: `pnpm exec tsc --noEmit && pnpm exec vitest run __tests__/ticket-helpers.test.ts`

## Dependencies

Independent. Can run alongside Task 1 or after.
