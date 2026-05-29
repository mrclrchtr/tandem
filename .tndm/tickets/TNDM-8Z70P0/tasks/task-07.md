# Task 7: Add .tndm naming comment to findRepoRoot

## Goal

Add a comment in `findRepoRoot` explaining why the directory is named `.tndm` (matching the CLI binary name) rather than `.tandem` (matching the repo/product name).

## Files

- `extensions/tools/ticket-helpers.ts`

## Change

In the JSDoc for `findRepoRoot`, add a sentence:

```typescript
/**
 * Walk up from startDir looking for `.git` or `.tndm`.
 *
 * `.tndm` is the on-disk directory name matching the `tndm` CLI binary,
 * following the convention of other git-aware tools (e.g., `.github`).
 */
export function findRepoRoot(startDir = process.cwd()): string {
```

## Verification

- `pnpm exec tsc --noEmit` — zero type errors (comment-only change)
