## Overview

Internal refactoring of supi-flow plugin addressing 5 architectural issues. No tool-visible changes. All 67 existing tests must remain green.

### Bottom-up execution order

1. **Type adapter** (#2) — `registerTypedTool` confines all `as never` casts to one boundary
2. **Shared task lifecycle** (#1) — `applyTaskMutation` replaces duplicated ensure→write→sync→reload logic
3. **File split** (#3) — extract ticket and task action handlers from monolithic `tndm-cli.ts`
4. **Overlap cleanup** (#4) — `flow-tools.ts` delegates to `applyTaskMutation`
5. **Comment** (#5) — explain `.tndm` naming in `findRepoRoot`

### Files

| File | Change |
|------|--------|
| `extensions/tools/ticket-helpers.ts` | Add `applyTaskMutation()`, deprecate `writeTaskDetailAndReload` |
| `extensions/tools/tool-specs.ts` | Add `registerTypedTool<T>()`, drop `as never` in execute wrappers |
| `extensions/tools/tndm-ticket-actions.ts` | **New** — create, update, show, list, awareness handlers |
| `extensions/tools/tndm-task-actions.ts` | **New** — task_add, task_edit, task_remove, task_complete, task_set, task_list handlers |
| `extensions/tools/tndm-cli.ts` | Replace 11-action switch with dispatch table, remove handler bodies |
| `extensions/tools/flow-tools.ts` | Replace `writeTaskDetailAndReload` calls with `applyTaskMutation` |
| `__tests__/tndm-cli-tool.test.ts` | Split into ticket-actions and task-actions test files |
| `__tests__/tndm-ticket-actions.test.ts` | **New** — tests for ticket action handlers |
| `__tests__/tndm-task-actions.test.ts` | **New** — tests for task action handlers with `applyTaskMutation` delegation |
| `__tests__/flow-tools.test.ts` | Update mocks for `applyTaskMutation` |
| `__tests__/ticket-helpers.test.ts` | Add `applyTaskMutation` tests |

### Verification gates

Each task has its own verification. Final task runs full `tsc --noEmit` + `vitest run` as integration gate.
