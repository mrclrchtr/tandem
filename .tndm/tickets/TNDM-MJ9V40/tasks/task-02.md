# Task 2: Refactor tndm-cli.ts to use shared helper, fix signal gaps, collapse dead branch

## Goal

Replace the 2 duplicate detail-doc chains in `tndm-cli.ts` (`task_add` and `task_edit`) with calls to `writeTaskDetailAndReload`. Pass `signal` in every `tndmJson`/`tndm` call. Collapse the identical `else if`/`else` branches in `task_edit`.

## Files

- `extensions/tools/tndm-cli.ts` — refactor `task_add` and `task_edit` cases
- `__tests__/tndm-cli-tool.test.ts` — update mocks and assertions, remove stripTrailingUndefined proxy

## Changes

### task_add case

Replace lines ~266-273 (the entire `if (params.task_detail !== undefined)` block):

```ts
if (params.task_detail !== undefined) {
    const taskNumber = extractLatestTaskNumber(result);
    finalResult = await writeTaskDetailAndReload(
        params.id, taskNumber, params.task_title, params.task_detail, signal,
    );
}
```

Remove imports that are now unused: `writeTaskDetailDoc` (if only used here), `ensureTaskDetailDoc` (if only used here). Keep `extractLatestTaskNumber`.

### task_edit case

Replace lines ~326-341 (the entire `if (params.task_detail !== undefined)` block):

```ts
if (params.task_detail !== undefined) {
    const applyTitleEdit = hasManifestFieldChanges;
    const taskTitle = params.task_title ?? `Task ${params.task_number}`;
    finalResult = await writeTaskDetailAndReload(
        params.id, params.task_number, taskTitle, params.task_detail, signal, applyTitleEdit,
    );
}
```

Note: when `applyTitleEdit` is true and `params.task_title` is provided, the helper applies the title edit. When `params.task_title` is not provided, we fall back to `Task N` as before.

Collapse the dead branches (currently ~lines 335-339) to the fallback:

```ts
if (!params.task_detail) {
    finalResult = await tndmJson<Record<string, unknown>>(args, signal);
}
```

Actually, simplify the entire final return block:

```ts
// After detail handling above (or no detail), produce result
if (!finalResult) {
    finalResult = await tndmJson<Record<string, unknown>>(args, signal);
}
```

This replaces the three branches (`if detail`, `else if hasManifestFieldChanges`, `else`) with one fallback.

### Signal passthrough

Audit every `tndmJson()` and `tndm()` call in the file. Ensure every call passes `signal` as second argument where available.

### Test updates

1. Remove the `stripTrailingUndefined` proxy from the `vi.mock` in `tndm-cli-tool.test.ts`
2. Add `vi.mock("../extensions/tools/ticket-helpers.js")` with mock for `writeTaskDetailAndReload`
3. Update all `toHaveBeenCalledWith` assertions to include `, undefined` as trailing signal argument
4. Update task_add/ task_edit test cases to expect `writeTaskDetailAndReload` calls instead of the old sequence
5. Keep truncation test (no changes needed)

## Verification (TDD)

RED: Update test expectations before changing source.
GREEN: Apply source changes.
Verify: `pnpm exec tsc --noEmit && pnpm exec vitest run __tests__/tndm-cli-tool.test.ts`

## Dependencies

Depends on Task 1 (helper + constants must exist first).
