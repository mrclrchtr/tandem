# Task 2: Update tndm-cli.ts to import helpers from ticket-helpers.ts

In `extensions/tools/tndm-cli.ts`:

- Remove local functions: `extractLatestTaskNumber`, `extractTaskTitle`, `extractTasks`, `loadTicket`, `ensureTaskDetailDoc`
- Import from `./ticket-helpers.js`: `loadTicket`, `ensureTaskDetailDoc`, `extractLatestTaskNumber`, `extractTaskTitle`, `filterFlowTasks`, `unwrapTicket`
- In `task_add` case: replace `extractTasks(result)` call with `filterFlowTasks(extractTasks(result))` or equivalent — ensure the code still compiles. The local `extractTasks` had a variant that unwrapped `result.tasks` directly or via `result.ticket.state.tasks`. After import, use the canonical `extractTasks` from helpers + `filterFlowTasks` if detail_path derivation is needed. However, for tndm-cli the detail_path derivation isn't strictly needed — the key is that the code compiles and behavior doesn't change.
- In `task_edit` case: same — ensure `extractTaskTitle` and `extractTasks` calls still work.

No behavioral changes. The task_add/task_edit orchestration stays in this file.

**Verification**: `pnpm exec tsc --noEmit` passes; `pnpm exec vitest run __tests__/tndm-cli-tool.test.ts` passes.
