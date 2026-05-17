# Archive

All 4 findings fixed and verified:

1. **Empty task title validation** — `handle_task_add` now rejects empty titles with `anyhow::bail!("task title must not be empty")`. Verified by new integration test `task_add_rejects_empty_title`.

2. **Empty plan rejection** — `executeFlowPlan` now throws if no `**Task N**:` lines are found in plan_content, preventing silent task clearing. Verified by new vitest `rejects empty plan_content instead of silently clearing tasks`.

3. **Atomic tag transitions** — Both `executeFlowPlan` and `executeFlowClose` now combine `--remove-tags` and `--add-tags` (and `--status` for close) into a single `tndm` call. Verified by updated vitest expectations.

4. **Clearing optional task fields** — `handle_task_edit` now treats empty strings as intent to clear: `task.file = if value.trim().is_empty() { None } else { Some(value) }`. Verified by new integration test `task_edit_clears_optional_fields`.

Test results:
- `cargo test -p tandem-core -p tandem-storage`: 90 passed, 1 ignored
- `cargo test -p tandem-cli`: 63 passed (including 2 new)
- `pnpm exec tsc --noEmit`: No errors
- `pnpm exec vitest run __tests__/flow-tools.test.ts`: 12 passed
