# Task 6: Replace writeTaskDetailAndReload with applyTaskMutation in flow-tools

## Goal

Replace `writeTaskDetailAndReload` calls in `flow-tools.ts` with `applyTaskMutation`, then remove the deprecated `writeTaskDetailAndReload` from `ticket-helpers.ts`.

## Files

- `extensions/tools/flow-tools.ts` — replace calls
- `extensions/tools/ticket-helpers.ts` — remove deprecated function
- `__tests__/flow-tools.test.ts` — update mocks

## Change

### `flow-tools.ts`

Replace `writeTaskDetailAndReload` import with `applyTaskMutation`.

In `executeFlowTask`:
- **add branch**: replace `writeTaskDetailAndReload(params.ticket_id, taskNumber, params.title, params.detail, signal)` with `applyTaskMutation(params.ticket_id, taskNumber, params.title, params.detail, signal)`
- **edit branch**: replace `writeTaskDetailAndReload(params.ticket_id, params.task_number, taskTitle, params.detail, signal, applyTitleEdit)` with `applyTaskMutation(params.ticket_id, params.task_number, taskTitle, params.detail, signal, applyTitleEdit)`

The function signatures are identical — this is a straight find-and-replace of the function name.

### `ticket-helpers.ts`

Remove the `writeTaskDetailAndReload` function body and its export. Remove the `ensureTaskDetailDoc` import (it's only used by `writeTaskDetailAndReload` and `applyTaskMutation` — keep it for `applyTaskMutation`).

Wait — `ensureTaskDetailDoc` is used by both. Keep the import. Just remove `writeTaskDetailAndReload`.

### `flow-tools.test.ts`

Replace all mock references:
- `vi.mocked(helpers.writeTaskDetailAndReload)` → `vi.mocked(helpers.applyTaskMutation)`
- `helpers.writeTaskDetailAndReload` → `helpers.applyTaskMutation`

The mock behavior is identical — just rename the mocked function.

## Verification

- `pnpm exec vitest run __tests__/flow-tools.test.ts` — all tests pass after mock rename
- `pnpm exec vitest run __tests__/ticket-helpers.test.ts` — tests still pass (only `applyTaskMutation` tests remain)
- `pnpm exec tsc --noEmit` — zero type errors
