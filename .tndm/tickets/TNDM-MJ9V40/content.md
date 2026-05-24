# supi-flow Refactor — Deduplicate task-detail chain, fix signal gaps, add integration tests, clean up docs

## Design decision summary

- One shared helper `writeTaskDetailAndReload()` in `ticket-helpers.ts` replaces 4 duplicate chains
- `applyTitleEdit: boolean` parameter — no raw CLI args in the helper's public API
- Flow tag constants extracted to `ticket-helpers.ts`
- `_repoRoot` module cache removed entirely (memoization not worth the invalidation risk)
- Test mock proxy (`stripTrailingUndefined`) removed — all assertions updated with explicit `, undefined`
- Integration tests in separate file, env-var gated (`TNDM_INTEGRATION_TEST=1`)
- Doc dedup: only trim overlapping content from `CLAUDE.md` (skills section already covered by skill files)
- Linting de-scoped to follow-up

## File map

| File | Change |
|---|---|
| `extensions/tools/ticket-helpers.ts` | ADD: `writeTaskDetailAndReload()`, `loadTaskList()`, tag constants. REMOVE: `_repoRoot` cache, `_resetRepoRootCache` |
| `extensions/tools/tndm-cli.ts` | Use helper for `task_add`/`task_edit` detail. Fix signal gaps. Collapse dead branch |
| `extensions/tools/flow-tools.ts` | Use helper for `add`/`edit`. Replace hard-coded tags with constants. Fix signal gaps. Import `loadTaskList` |
| `__tests__/ticket-helpers.test.ts` | NEW tests for helper, constants, `loadTaskList`. Remove cache-behavior test |
| `__tests__/tndm-cli-tool.test.ts` | Remove mock proxy. Update all assertions |
| `__tests__/flow-tools.test.ts` | Remove mock proxy. Update all assertions |
| `__tests__/integration.test.ts` | NEW: integration tests (env-var gated) |
| `CLAUDE.md` | Trim skill conventions section |
