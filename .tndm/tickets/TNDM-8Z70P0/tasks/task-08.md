# Task 8: Final verification: full test suite + typecheck

## Goal

Run the full test suite and type-check as the integration gate. Confirm all 67+ tests pass and zero type errors.

## Verification

```sh
cd plugins/supi-flow
pnpm exec tsc --noEmit
pnpm exec vitest run
```

Expected: `TypeScript: No errors found` and `PASS (67+) FAIL (0)`.

Also confirm:
- All 7 tools still registered (`__tests__/resources.test.ts` passes)
- `flow-tools.test.ts` passes (executeFlowStart, executeFlowPlan, executeFlowApply, executeFlowTask, executeFlowCompleteTask, executeFlowClose)
- `ticket-helpers.test.ts` passes (findRepoRoot, resolveTicketPath, flow tag constants, loadTaskList, applyTaskMutation)
- `tndm-ticket-actions.test.ts` passes (create, update, show, list, awareness, truncation)
- `tndm-task-actions.test.ts` passes (task_add, task_edit with applyTaskMutation delegation)
- `cli.test.ts` passes (tndm, tndmJson, tndmVersion, signal handling)
