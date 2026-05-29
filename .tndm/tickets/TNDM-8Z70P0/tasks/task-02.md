# Task 2: Add applyTaskMutation shared function to ticket-helpers

## Goal

Add `applyTaskMutation()` to `ticket-helpers.ts`. This function encapsulates the full task detail-doc lifecycle: ensure → write → sync → reload. It replaces the need for callers to manually call `ensureTaskDetailDoc`, `writeTaskDetailDoc`, `tndm("sync")`, and `loadTicket` in sequence.

## Files

- `extensions/tools/ticket-helpers.ts` — add function, deprecate `writeTaskDetailAndReload`
- `__tests__/ticket-helpers.test.ts` — add tests

## Change

Add to `ticket-helpers.ts`:

```typescript
/**
 * Apply a task detail mutation end-to-end.
 *
 * 1. Ensure the detail doc via tndm.
 * 2. Optionally apply a title edit to the manifest.
 * 3. Write the markdown body.
 * 4. Sync the ticket.
 * 5. Reload and return the updated ticket snapshot.
 *
 * Replaces writeTaskDetailAndReload — use this instead.
 */
export async function applyTaskMutation(
  id: string,
  taskNumber: number,
  title: string,
  detail: string,
  signal?: AbortSignal,
  applyTitleEdit?: boolean,
): Promise<Record<string, unknown>> {
  const detailResult = await ensureTaskDetailDoc(id, taskNumber, signal);

  if (applyTitleEdit) {
    await tndmJson<Record<string, unknown>>(
      ["ticket", "task", "edit", id, String(taskNumber), "--title", title],
      signal,
    );
  }

  await writeTaskDetailDoc(detailResult.path, taskNumber, title, detail);
  await tndm(["ticket", "sync", id], signal);
  return loadTicket(id, signal);
}
```

Mark `writeTaskDetailAndReload` with a `@deprecated Use applyTaskMutation instead` JSDoc tag. Do NOT remove it yet — it's still imported in `tndm-cli.ts` and `tndm-cli-tool.test.ts` until later tasks remove those references.

Add to `__tests__/ticket-helpers.test.ts`:

1. Test: `applyTaskMutation` calls ensure → write → sync → reload in order
2. Test: does NOT call task edit when `applyTitleEdit` is false
3. Test: calls task edit when `applyTitleEdit` is true
4. Test: passes signal to every internal call
5. Test: returns the reloaded ticket

Follow the existing test patterns in the file (mock `tndm`, `tndmJson`, `writeTaskDetailDoc`).

## Verification

- `pnpm exec vitest run __tests__/ticket-helpers.test.ts` — all existing + new tests pass
- `pnpm exec tsc --noEmit` — zero type errors
