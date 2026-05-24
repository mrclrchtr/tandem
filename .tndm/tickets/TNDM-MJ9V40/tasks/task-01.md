# Task 1: Create shared helper + tag constants + loadTaskList in ticket-helpers.ts

## Goal

Add `writeTaskDetailAndReload()`, flow tag constants, and `loadTaskList()` to `ticket-helpers.ts`.

## Files

- `extensions/tools/ticket-helpers.ts` — add exports
- `__tests__/ticket-helpers.test.ts` — add new tests

## Changes

### 1. `writeTaskDetailAndReload(id, taskNumber, title, detail, signal?, applyTitleEdit?)`

Encapsulates the 5-step chain:
1. `ensureTaskDetailDoc(id, taskNumber, signal)` — register doc path
2. If `applyTitleEdit` is true: `tndmJson(["ticket", "task", "edit", id, String(taskNumber), "--title", title], signal)` — apply title change to manifest
3. `writeTaskDetailDoc(path, taskNumber, title, detail)` — write markdown
4. `tndm(["ticket", "sync", id])` — register file
5. `return loadTicket(id, signal)` — return fresh snapshot

Imports from `./doc-writes.js` (writeTaskDetailDoc) and `../cli.js` (tndm, tndmJson). Keep existing import of `ensureTaskDetailDoc` and `loadTicket` from same file.

### 2. Flow tag constants

```ts
export const FLOW_TAGS_ALL = "flow:brainstorm,flow:planned,flow:applying,flow:done";
export const FLOW_TAG_BRAINSTORM = "flow:brainstorm";
export const FLOW_TAG_PLANNED = "flow:planned";
export const FLOW_TAG_APPLYING = "flow:applying";
export const FLOW_TAG_DONE = "flow:done";
```

### 3. `loadTaskList(id, signal?)`

Move from `flow-tools.ts` to `ticket-helpers.ts` and export it. Uses `tndmJson` and `filterFlowTasks` (already in this module).

## Verification (TDD)

**RED tests first:**

1. `writeTaskDetailAndReload` — mock `ensureTaskDetailDoc`, `writeTaskDetailDoc`, `tndm`, `tndmJson`, `loadTicket`. Verify call sequence:
   - `ensureTaskDetailDoc` called with correct id/taskNumber
   - If `applyTitleEdit: true`: `tndmJson` called with title edit args
   - If `applyTitleEdit: false/undefined`: `tndmJson` NOT called for edit
   - `writeTaskDetailDoc` called with path, number, title, detail
   - `tndm` called with `["ticket", "sync", id]` (plus signal)
   - `loadTicket` called and its return value is the function's return value
   - Signal passed through to every internal call

2. Tag constants — literal value checks (trivial assertions).

3. `loadTaskList` — mock `tndmJson` returning arrays and non-arrays:
   - When array: passes through `filterFlowTasks`, returns filtered entries
   - When non-array: throws
   - When empty array: returns `[]`

**GREEN:** implement the exports.

## Dependencies

This task is the foundation for tasks 2 and 3.
