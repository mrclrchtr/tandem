# Archive

## Verification Summary

### Type check
- `pnpm exec tsc --noEmit` — **zero errors**

### Unit tests (all mocks)
- `pnpm exec vitest run` — **67 passed, 0 failed** (6 integration skipped without env var)
- Test files: `ticket-helpers.test.ts` (16), `tndm-cli-tool.test.ts` (5), `flow-tools.test.ts` (24), `cli.test.ts`, `resources.test.ts`, `index.test.ts`

### Integration tests (real tndm CLI)
- `TNDM_INTEGRATION_TEST=1 pnpm exec vitest run __tests__/integration.test.ts` — **6 passed, 0 failed**
- Verified: flow_start (ticket creation), task_add with detail (file on disk), task_list shape, task_edit (file content change), task_complete, flow_close (archive.md + done status)

### Changes verified per task

**Task 1** — `writeTaskDetailAndReload` helper extracted to `ticket-helpers.ts` with `applyTitleEdit` boolean parameter, flow tag constants added, `loadTaskList` moved from flow-tools.ts. 16 passing tests cover the helper, constants, and loadTaskList.

**Task 2** — `tndm-cli.ts` `task_add`/`task_edit` now use `writeTaskDetailAndReload`. Signal passed consistently in every `tndmJson`/`tndm` call. Dead `else if`/`else` collapsed. Mock proxy removed from `tndm-cli-tool.test.ts`. 5 passing tests.

**Task 3** — `flow-tools.ts` `executeFlowTask` add/edit use `writeTaskDetailAndReload`. Hard-coded tag strings replaced with `FLOW_TAG_*`/`FLOW_TAGS_ALL` constants. Signal gaps fixed. `loadTaskList` imported from ticket-helpers. Mock proxy removed from `flow-tools.test.ts`. 24 passing tests.

**Task 4** — `_repoRoot` module-level cache removed. `_resetRepoRootCache` export removed. `findRepoRoot` always walks. 16 passing tests (cache test updated to verify repeatability).

**Task 5** — Full suite: 67 pass, 0 fail. No regressions.

**Task 6** — Integration test file `__tests__/integration.test.ts` created, gated via `TNDM_INTEGRATION_TEST=1`. 6 tests exercise real `tndm` CLI end-to-end. Vitest config updated to pass env var to workers.

**Task 7** — CLAUDE.md "Skill conventions" section replaced with 2-line pointer to `skills/*/SKILL.md` and `README.md`. All other sections preserved.
