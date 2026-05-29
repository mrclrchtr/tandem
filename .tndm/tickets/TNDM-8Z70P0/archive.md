# Archive

## Verification Evidence

### Fresh runs (2026-05-29)

**Typecheck:**
```
cd plugins/supi-flow && pnpm exec tsc --noEmit
# TypeScript: No errors found
```

**Full test suite:**
```
cd plugins/supi-flow && pnpm exec vitest run
# PASS (95) FAIL (0) skipped (6)
```

### Task-by-task verification

| # | Task | Verification |
|---|------|-------------|
| 1 | registerTypedTool adapter + drop as-never | tsc --noEmit zero errors; resources.test.ts 5/5 pass |
| 2 | applyTaskMutation shared function | ticket-helpers.test.ts 15 pass (10 existing + 5 new) |
| 3 | Extract ticket action handlers | tndm-ticket-actions.test.ts 12/12 pass |
| 4 | Extract task action handlers with applyTaskMutation | tndm-task-actions.test.ts 19/19 pass |
| 5 | tndm-cli.ts dispatch table | tndm-cli-tool.test.ts 2/2 pass (dispatch + fallback) |
| 6 | Replace writeTaskDetailAndReload in flow-tools | flow-tools.test.ts 20/20 pass |
| 7 | .tndm naming comment | Comment-only change, tsc --noEmit zero errors |
| 8 | Full test suite + typecheck | 95/95 pass, 0 failures |

### Post-review fixes

1. `import type` for supi_tndm_cli_params — breaks circular runtime dependency
2. ToolResult type consolidated in ticket-helpers.ts — single source of truth
3. Dispatch smoke tests added — routing + unknown-action fallback

### Files changed (source)

- `extensions/tools/tool-specs.ts` — added registerTypedTool<T>() adapter, typedExecute helper
- `extensions/tools/tndm-cli.ts` — rewritten as dispatch table
- `extensions/tools/tndm-ticket-actions.ts` — new: 5 ticket action handlers
- `extensions/tools/tndm-task-actions.ts` — new: 6 task action handlers using applyTaskMutation
- `extensions/tools/ticket-helpers.ts` — added applyTaskMutation, formatContent, ToolResult; removed writeTaskDetailAndReload
- `extensions/tools/flow-tools.ts` — delegates to applyTaskMutation
- `extensions/index.ts` — uses registerTypedTool adapter

### Files changed (tests)

- `__tests__/tndm-cli-tool.test.ts` — rewritten as dispatch smoke tests
- `__tests__/tndm-ticket-actions.test.ts` — new: ticket action handler tests
- `__tests__/tndm-task-actions.test.ts` — new: task action handler tests
- `__tests__/flow-tools.test.ts` — updated mocks for applyTaskMutation
- `__tests__/ticket-helpers.test.ts` — added applyTaskMutation tests, removed writeTaskDetailAndReload tests

### CLAUDE.md

Updated file structure to reflect new files and their responsibilities.
