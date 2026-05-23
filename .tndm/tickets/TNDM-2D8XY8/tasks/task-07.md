# Task 7: supi-flow: remove clear_detail, always ensure detail doc on task add

## Goal

Remove the `clear_detail` path from `supi_flow_task`. When `detail` is not provided on add, still call `detail ensure` so the task always gets a detail doc. When `detail` is provided, write full content.

## Change

In `flow-tools.ts` (`supsiFlowTaskParams`):
- Remove `clear_detail: Type.Optional(Type.Boolean(...))` from the schema.
- Remove the `if (params.detail !== undefined && params.clear_detail)` validation.

In `executeFlowTask`:
- **Add case**: After calling `tndm task add`, always call `ensureTaskDetailDoc`. If `params.detail` is provided, write it; otherwise the minimal template from ensure is fine. Sync after writing.
- **Edit case**: Remove the `else if (params.clear_detail)` branch entirely — unlinking is no longer a thing. When `detail` is provided, ensure + write. When not provided, skip detail doc operations.

In test files:
- `flow-tools.test.ts`: Remove tests for `clear_detail` behavior. Add/update test verifying task add always creates a detail doc even without `detail` param.
- `tndm-cli-tool.test.ts`: Update the "keeps headline-only tasks manifest-only" test — task_add should now also call `detail ensure`.
