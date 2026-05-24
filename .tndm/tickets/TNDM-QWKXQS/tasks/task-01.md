# Task 1: Create ticket-helpers.ts with all shared utility functions

Create `extensions/tools/ticket-helpers.ts` exporting these functions extracted from `tndm-cli.ts` and `flow-tools.ts`:

- `loadTicket(id, signal?)` → `tndmJson<Record<string, unknown>>(["ticket", "show", id], signal)`
- `ensureTaskDetailDoc(id, taskNumber, signal?)` → `tndmJson<{ path: string }>(["ticket", "task", "detail", "ensure", id, String(taskNumber)], signal)`
- `extractTasks(result)` → unwrap ticket envelope, return filtered task array
- `filterFlowTasks(tasks)` → canonical version from flow-tools (includes `detail_path` derivation via `task-XX.md` convention)
- `extractLatestTaskNumber(result)` → max task number from extracted tasks
- `extractTaskTitle(result, taskNumber)` → title of matching task
- `unwrapTicket(result)` → extract `result.ticket` if present, else return result
- `extractTicketStatus(result)` → status string from unwrapped ticket
- `extractTicketTags(result)` → tags array from unwrapped ticket
- `extractContentPath(result)` → content_path from unwrapped ticket
- `findRepoRoot(startDir?)` → walk up from startDir finding `.git` or `.tndm`; **memoize** in module-level `let _repoRoot: string | null = null`
- `resolveTicketPath(ticketPath)` → resolve relative paths against `findRepoRoot()`, pass through absolute paths
- `readRequiredTicketContent(ticketId, contentPath, toolName)` → **async** — uses `readFile` from `node:fs/promises` instead of `readFileSync`; throws if missing or blank

Define `FlowTaskListEntry` type in this file (moved from flow-tools.ts).

**Verification**: `pnpm exec tsc --noEmit` passes (file compiles cleanly).
