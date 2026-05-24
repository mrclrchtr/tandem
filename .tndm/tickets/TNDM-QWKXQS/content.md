## Overview

Refactor the supi-flow plugin internals to eliminate ~150 lines of duplicated code between `tndm-cli.ts` and `flow-tools.ts`, fix synchronous I/O, improve error robustness, and add missing test coverage.

### Phase 1 — Structural + correctness

Extract shared utility functions into a new `ticket-helpers.ts` module:
- `loadTicket`, `ensureTaskDetailDoc` — CLI wrappers duplicated in both files
- `extractTasks`, `filterFlowTasks`, `extractLatestTaskNumber`, `extractTaskTitle` — JSON unwrapping duplicated in both files
- `unwrapTicket`, `extractTicketStatus`, `extractTicketTags`, `extractContentPath` — envelope helpers
- `findRepoRoot` (now memoized), `resolveTicketPath`, `readRequiredTicketContent` (now async)

Both tool files import from the shared module instead of defining locals. No behavioral changes.

### Phase 2 — Cosmetic + robustness + testing

- Rename misleading `_signal` → `signal` in `tool-specs.ts` (7 wrappers)
- Replace fragile `message.includes("not found")` with targeted regex in `executeFlowCompleteTask`
- Unit test `resolveTicketPath` and `findRepoRoot` (including cache behavior)
- Add `pnpm install --frozen-lockfile` to CI to prevent lockfile drift

### Non-goals

- Changing tool behavior or API surface
- Touching skill files, prompts, `doc-writes.ts`, `cli.ts`, or `index.ts`
- Modifying the tandem Rust CLI
- Extracting task_add/task_edit orchestration (leaf helpers only)
