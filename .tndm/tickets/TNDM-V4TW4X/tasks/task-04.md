# Task 4: Queue task-detail and archive markdown writes

## Goal
Make every markdown file write performed by `supi-flow` participate in PI's file-mutation queue, using the real path returned by `tndm` for task-detail docs and `archive.md`, while keeping the current file contents and sync behavior intact.

## Files
- `plugins/supi-flow/extensions/tools/doc-writes.ts`
- `plugins/supi-flow/extensions/tools/tndm-cli.ts`
- `plugins/supi-flow/extensions/tools/flow-tools.ts`
- `plugins/supi-flow/__tests__/tndm-cli-tool.test.ts`
- `plugins/supi-flow/__tests__/flow-tools.test.ts`

## Test strategy
**Test-driven.** Preserve the existing file-content assertions while refactoring the write path to an async queued helper.

## Change to make
1. Add `plugins/supi-flow/extensions/tools/doc-writes.ts` with one async helper that:
   - takes the real target path returned by `tndm`
   - wraps the full `mkdir` + `writeFile` window in `withFileMutationQueue()`
   - writes markdown with the same content shape the current tests expect
2. Update `plugins/supi-flow/extensions/tools/tndm-cli.ts` so `task_add` and `task_edit` await the shared helper instead of calling sync filesystem APIs directly.
3. Update `plugins/supi-flow/extensions/tools/flow-tools.ts` so task-detail writes and `executeFlowClose()` archive writes also await the shared helper.
4. Remove the old sync write helpers/imports once the queued helper covers all markdown writes.
5. Finish by running the focused plugin validation sweep for all files touched by this change set.

## RED
- Re-run the existing task-detail and archive-write tests in `plugins/supi-flow/__tests__/tndm-cli-tool.test.ts` and `plugins/supi-flow/__tests__/flow-tools.test.ts` after switching their expectations to the async path only if needed.
- If coverage is missing for the archive path after the helper extraction, add one assertion that the written `archive.md` content still matches the current format.
- Watch the focused tests fail before changing the implementation.

## GREEN
- Implement `plugins/supi-flow/extensions/tools/doc-writes.ts` and wire both tool modules to it.
- Keep the written markdown text stable so existing content assertions remain valid.
- Make the focused flow/tool tests pass after the async refactor.

## REFACTOR
- Remove duplicated sync write logic from `plugins/supi-flow/extensions/tools/tndm-cli.ts` and `plugins/supi-flow/extensions/tools/flow-tools.ts`.
- Keep queueing scoped to the real returned path; do not invent new path-normalization behavior for unrelated inputs.

## Verification
```sh
cd plugins/supi-flow
pnpm exec vitest run __tests__/flow-tools.test.ts __tests__/tndm-cli-tool.test.ts
pnpm exec tsc --noEmit
pnpm exec vitest run __tests__/index.test.ts __tests__/resources.test.ts __tests__/cli.test.ts __tests__/tndm-cli-tool.test.ts __tests__/flow-tools.test.ts
```
Expected result: focused file-write tests stay green after the async queue refactor, TypeScript passes, and the full targeted verification sweep succeeds.
