# Task 3: Update flow-tools.ts to import helpers from ticket-helpers.ts

In `extensions/tools/flow-tools.ts`:

- Remove local functions: `extractLatestTaskNumber`, `extractTaskTitle`, `extractTasks`, `filterFlowTasks`, `loadTicket`, `ensureTaskDetailDoc`, `unwrapTicket`, `extractTicketStatus`, `extractTicketTags`, `extractContentPath`, `readRequiredTicketContent`, `resolveTicketPath`, `findRepoRoot`
- Import from `./ticket-helpers.js`: all of the above
- `readRequiredTicketContent` is now async — `executeFlowApply` is already `async`, so just add `await` at the call site
- Remove now-unused imports: `existsSync`, `isAbsolute`, `resolve`, `dirname`, `join`, `readFileSync` from `node:fs` / `node:path` (verify which are no longer needed)
- Remove the local `FlowTaskListEntry` type definition (now in ticket-helpers)

No behavioral changes. Task orchestration stays in this file.

**Verification**: `pnpm exec tsc --noEmit` passes; `pnpm exec vitest run __tests__/flow-tools.test.ts` passes.
