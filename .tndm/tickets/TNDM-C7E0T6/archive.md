# Archive

## Verification Results

**Test suite:** `pnpm exec vitest run` — 30/30 passed (4 test files)
**TypeScript:** `pnpm exec tsc --noEmit` — No errors

### Changes made

**`extensions/tools/flow-tools.ts`:**
1. `executeFlowPlan`: Changed from single `ticket update --add-tags flow:planned --remove-tags flow:brainstorm` call to two sequential calls: (1) `--remove-tags flow:brainstorm,flow:planned,flow:applying,flow:done` to clear any existing flow-state, then (2) `--add-tags flow:planned` to set the correct one.
2. `executeFlowClose`: Same pattern — (1) clear all flow-state tags, then (2) set `--status done --add-tags flow:done`.

**`__tests__/flow-tools.test.ts`:**
1. Updated `executeFlowPlan` test to expect two separate update calls (remove-all, then add).
2. Updated `executeFlowClose` "updates status and tags" test to expect two separate update calls (remove-all, then status+add).

### Root cause
Both functions assumed a specific starting flow-state tag (`flow:brainstorm` for plan, `flow:applying,flow:planned` for close) rather than cleaning up ALL flow-state tags. When re-running `supi_flow_plan` during the apply phase (ticket had `flow:applying`), the `flow:applying` tag persisted alongside `flow:planned`. The two-call approach (remove-all first, then add target) works around the CLI's add-before-remove ordering and is future-proof against new flow-state tags.
