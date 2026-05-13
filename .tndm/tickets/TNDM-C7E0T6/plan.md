- [x] **Task 1**: Fix executeFlowPlan — remove all flow-state tags before adding flow:planned
  - File: plugins/supi-flow/extensions/tools/flow-tools.ts
  - Change: Replace single update call (add flow:planned, remove flow:brainstorm) with two calls: (1) remove all flow-state tags, (2) add flow:planned
  - Verification: `pnpm exec vitest run __tests__/flow-tools.test.ts`

- [x] **Task 2**: Fix executeFlowClose — remove all flow-state tags before adding flow:done
  - File: plugins/supi-flow/extensions/tools/flow-tools.ts
  - Change: Replace single update call (add flow:done, remove flow:applying+flow:planned) with two calls: (1) remove all flow-state tags, (2) set status=done + add flow:done
  - Verification: `pnpm exec vitest run __tests__/flow-tools.test.ts`

- [x] **Task 3**: Update executeFlowPlan test to expect two update calls (remove-all, then add)
  - File: plugins/supi-flow/__tests__/flow-tools.test.ts
  - Change: Update the tag assertion to check remove-all-call and add-flow:planned-call separately
  - Verification: `pnpm exec vitest run __tests__/flow-tools.test.ts`

- [x] **Task 4**: Update executeFlowClose test to expect two update calls (remove-all, then status+add)
  - File: plugins/supi-flow/__tests__/flow-tools.test.ts
  - Change: Update the tag assertion to check remove-all-call and status+add-flow:done-call separately
  - Verification: `pnpm exec vitest run __tests__/flow-tools.test.ts`

- [x] **Task 5**: Run full test suite to confirm everything passes
  - File: plugins/supi-flow/
  - Verification: `pnpm exec vitest run`
