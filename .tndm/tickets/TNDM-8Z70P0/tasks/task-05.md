# Task 5: Rewrite tndm-cli.ts as action dispatch table

## Goal

Rewrite `tndm-cli.ts` so `executeTndmCli` is a dispatch table mapping action names to handler functions imported from `tndm-ticket-actions.ts` and `tndm-task-actions.ts`. Remove all inline action handler bodies.

## Files

- `extensions/tools/tndm-cli.ts` — rewrite dispatch
- `extensions/tools/tool-specs.ts` — no changes (still imports `executeTndmCli`)
- `__tests__/tndm-cli-tool.test.ts` — remove or replace with delegation tests

## Change

Replace the 11-action `switch` with:

```typescript
import { handleCreate, handleUpdate, handleShow, handleList, handleAwareness } from "./tndm-ticket-actions.js";
import { handleTaskAdd, handleTaskEdit, handleTaskRemove, handleTaskComplete, handleTaskSet, handleTaskList } from "./tndm-task-actions.js";

export async function executeTndmCli(params: TndmCliParams, signal?: AbortSignal) {
  const handlers: Record<string, (p: TndmCliParams, s?: AbortSignal) => Promise<ToolResult>> = {
    create: handleCreate,
    update: handleUpdate,
    show: handleShow,
    list: handleList,
    awareness: handleAwareness,
    task_add: handleTaskAdd,
    task_edit: handleTaskEdit,
    task_remove: handleTaskRemove,
    task_complete: handleTaskComplete,
    task_set: handleTaskSet,
    task_list: handleTaskList,
  };

  const handler = handlers[params.action];
  if (!handler) throw new Error(`supi_tndm_cli: unknown action "${params.action}"`);
  return handler(params, signal);
}
```

Remove: `addOptionalFlags`, `formatContent` (moved in Task 3), all 11 case bodies. Keep: `actionEnum`, `supi_tndm_cli_params`, `TndmCliParams` type, and the JSDoc comment about action→subcommand mapping.

### Test file: `__tests__/tndm-cli-tool.test.ts`

Since all action tests moved to `tndm-ticket-actions.test.ts` and `tndm-task-actions.test.ts`, this file can either:
- Be deleted (recommended — no remaining unique test coverage)
- Or contain a lightweight delegation test verifying the dispatch table routes to correct handlers

Preferred: delete `__tests__/tndm-cli-tool.test.ts`. Update `resources.test.ts` if it imports from it (it doesn't — it tests registration only).

## Verification

- `pnpm exec vitest run` — all tests pass, 0 failures
- `pnpm exec tsc --noEmit` — zero type errors
