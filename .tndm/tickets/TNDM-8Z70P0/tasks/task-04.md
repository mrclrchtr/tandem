# Task 4: Extract task action handlers to tndm-task-actions.ts using applyTaskMutation

## Goal

Create `extensions/tools/tndm-task-actions.ts` containing 6 handler functions: `handleTaskAdd`, `handleTaskEdit`, `handleTaskRemove`, `handleTaskComplete`, `handleTaskSet`, `handleTaskList`. `handleTaskAdd` and `handleTaskEdit` must use `applyTaskMutation` from `ticket-helpers.ts` instead of the deprecated `writeTaskDetailAndReload`.

## Files

- `extensions/tools/tndm-task-actions.ts` — **new file**
- `__tests__/tndm-task-actions.test.ts` — **new file**

## Change

### New file: `tndm-task-actions.ts`

Extract from `tndm-cli.ts` the 6 task action cases. Key differences from current code:

- `handleTaskAdd`: calls `tndmJson("task add")` → `extractLatestTaskNumber` → `applyTaskMutation(id, taskNumber, title, detail, signal)` (no `applyTitleEdit` for add)
- `handleTaskEdit`: if detail provided → `loadTicket` for title extraction → `applyTaskMutation(id, taskNumber, title, detail, signal, applyTitleEdit)`. If no detail → `tndmJson("task edit")` directly
- Other 4 handlers: direct CLI wrappers, unchanged logic

Import `applyTaskMutation` from `ticket-helpers.js`. Import `formatContent` from wherever it was placed in Task 3 (either `ticket-helpers.ts` or `tndm-ticket-actions.ts`).

### Test file: `__tests__/tndm-task-actions.test.ts`

Port from `tndm-cli-tool.test.ts`:
- `task_add` without detail test
- `task_add` with detail test (verifies `applyTaskMutation` delegation)
- `task_edit` detail-only test (verifies `applyTaskMutation` delegation with `applyTitleEdit=false`)

Mock `cli.ts` (`tndm`, `tndmJson`) and mock `ticket-helpers.applyTaskMutation` — same pattern as existing tests mock `writeTaskDetailAndReload`.

## Verification

- `pnpm exec vitest run __tests__/tndm-task-actions.test.ts` — all tests pass
- `pnpm exec tsc --noEmit` — zero type errors
