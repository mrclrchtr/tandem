# Task 3: Refactor flow-tools.ts to use shared helper, constants, fix signal gaps

## Goal

Replace the 2 duplicate detail-doc chains in `executeFlowTask` (`add` and `edit`) with calls to `writeTaskDetailAndReload`. Replace hard-coded tag strings with `FLOW_TAG_*` and `FLOW_TAGS_ALL` constants. Pass `signal` in every internal call. Import `loadTaskList` from `ticket-helpers.ts`.

## Files

- `extensions/tools/flow-tools.ts` — refactor `executeFlowTask` add/edit, `executeFlowStart`, `executeFlowPlan`, `executeFlowClose`
- `__tests__/flow-tools.test.ts` — update mocks and assertions, remove stripTrailingUndefined proxy

## Changes

### executeFlowTask — add case

Replace detail chain (~lines 266-273) with:

```ts
if (params.detail !== undefined) {
    finalResult = await writeTaskDetailAndReload(
        params.ticket_id, taskNumber, params.title, params.detail, signal,
    );
}
```

### executeFlowTask — edit case

Replace detail chain (~lines 310-324) with:

```ts
if (params.detail !== undefined) {
    const applyTitleEdit = params.title !== undefined && params.title.trim().length > 0;
    finalResult = await writeTaskDetailAndReload(
        params.ticket_id, params.task_number, taskTitle, params.detail, signal, applyTitleEdit,
    );
}
```

Where `taskTitle` is computed before this block from `params.title` (if provided) or extracted from the ticket. Since we no longer need `taskSnapshot` for title extraction in the `applyTitleEdit` path (the helper handles the edit), simplify: use `params.title ?? "Task N"`. If `applyTitleEdit` is false and `params.title` is not set, we still need the current title from the ticket — so keep `loadTicket` for that fallback, but it's only called when detail is provided AND title is not.

Actually, simplify further: extract `taskTitle` before the `if`:

```ts
let taskTitle: string;
if (params.title !== undefined && params.title.trim()) {
    taskTitle = params.title;
} else {
    const ticket = await loadTicket(params.ticket_id, signal);
    taskTitle = extractTaskTitle(ticket, params.task_number) ?? `Task ${params.task_number}`;
}
```

Then the detail block is clean.

### Tag constant replacement

- `executeFlowStart`: `"--tags", "flow:brainstorm"` → `"--tags", FLOW_TAG_BRAINSTORM`
- `executeFlowPlan`: `"--remove-tags", "flow:brainstorm,flow:planned,flow:applying,flow:done"` → `"--remove-tags", FLOW_TAGS_ALL` and `"--add-tags", "flow:planned"` → `"--add-tags", FLOW_TAG_PLANNED`
- `executeFlowClose`: tag remove → `FLOW_TAGS_ALL`, add → `FLOW_TAG_DONE`

### loadTaskList import

Remove local function definition. Add import from `"./ticket-helpers.js"`.

### Signal passthrough

Audit every `tndmJson()` and `tndm()` call. The existing gap in the edit fallback branch gets fixed when we unify the return path.

### Test updates

1. Remove `stripTrailingUndefined` proxy from `flow-tools.test.ts`
2. Add `vi.mock("../extensions/tools/ticket-helpers.js")` with mock for `writeTaskDetailAndReload` and `loadTaskList`
3. Update all assertions for signal and tag constants
4. Update add/edit tests to expect `writeTaskDetailAndReload` calls

## Verification (TDD)

RED: Update test expectations first.
GREEN: Apply source changes.
Verify: `pnpm exec tsc --noEmit && pnpm exec vitest run __tests__/flow-tools.test.ts`

## Dependencies

Depends on Task 1 (helper + constants + loadTaskList must exist first).
