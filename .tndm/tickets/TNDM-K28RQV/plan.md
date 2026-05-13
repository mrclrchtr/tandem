## Implementation Plan

**Goal**: Remove auto-commit behavior from `supi_flow_close` and update the archive skill to give the user control over committing.

---

- [x] **Task 1**: Remove `gitAddCommit` call from `executeFlowClose`
  - File: `plugins/supi-flow/extensions/tools/flow-tools.ts`
  - Change: Delete the `import { gitAddCommit }` and the `gitAddCommit` invocation block inside `executeFlowClose`. Update the return object to remove `commitHash`.
  - Verification: `pnpm exec tsc --noEmit`

- [x] **Task 2**: Update tool registration description for `supi_flow_close`
  - File: `plugins/supi-flow/extensions/index.ts`
  - Change: Remove "and auto-commits .tndm/ changes" from the tool description. Update prompt guidelines if needed.
  - Verification: `pnpm exec tsc --noEmit`

- [x] **Task 3**: Update tests for `executeFlowClose`
  - File: `plugins/supi-flow/__tests__/flow-tools.test.ts`
  - Change: Remove `gitAddCommit` mock setup and the "commits after close" test case. Remove any unused imports.
  - Verification: `pnpm exec vitest run -- flow-tools.test.ts`

- [x] **Task 4**: Rewrite archive skill close-out steps
  - File: `plugins/supi-flow/skills/supi-flow-archive/SKILL.md`
  - Change: Update Steps 5–6. Step 5 calls `supi_flow_close` without expecting auto-commit. Step 6 checks `git status`, asks the user whether to commit everything together or end the flow, and handles the response.
  - Verification: Read `SKILL.md` and confirm it matches the intended behavior.

- [x] **Task 5**: Update README tool description
  - File: `plugins/supi-flow/README.md`
  - Change: Update the `supi_flow_close` description and the mermaid diagram label to remove "auto-commit" language.
  - Verification: `rg "auto-commit" plugins/supi-flow/README.md` should return no matches.

- [x] **Task 6**: Final test run
  - Verification: `pnpm exec vitest run` passes in `plugins/supi-flow/`.
