# Task 3: Extract ticket action handlers to tndm-ticket-actions.ts

## Goal

Create `extensions/tools/tndm-ticket-actions.ts` containing 5 handler functions extracted from `tndm-cli.ts`: `handleCreate`, `handleUpdate`, `handleShow`, `handleList`, `handleAwareness`. Each handler accepts typed params and returns `{ content, details }`.

## Files

- `extensions/tools/tndm-ticket-actions.ts` — **new file**
- `__tests__/tndm-ticket-actions.test.ts` — **new file**

## Change

### New file: `tndm-ticket-actions.ts`

Extract from `tndm-cli.ts`:

- `addOptionalFlags` helper (private, local to this file)
- `formatContent` helper (private, local to this file — also needed by task actions; export it)
- 5 handler functions, each with explicit params matching the action's requirements

Each handler signature:
```typescript
import type { TndmCliParams } from "./tndm-cli.js"; // or define locally if cleaner

export async function handleCreate(params: PickRequired<TndmCliParams, "title"> & TndmCliParams, signal?: AbortSignal): Promise<ToolResult>
export async function handleUpdate(params: PickRequired<TndmCliParams, "id"> & TndmCliParams, signal?: AbortSignal): Promise<ToolResult>
// etc.
```

Move `formatContent` to a shared location or export it from here — it's needed by task actions too. Best approach: move `formatContent` to `ticket-helpers.ts` since it's output formatting shared by both action groups.

### Test file: `__tests__/tndm-ticket-actions.test.ts`

Port relevant tests from `tndm-cli-tool.test.ts`:
- List envelope handling test (already exists)
- Truncation test (already exists)
- Add tests for create, update, show, awareness action handlers

Mock `cli.ts` (`tndm`, `tndmJson`) — same pattern as existing tests.

## Verification

- `pnpm exec vitest run __tests__/tndm-ticket-actions.test.ts` — all tests pass
- `pnpm exec tsc --noEmit` — zero type errors
