This plan modifies `executeFlowStart` in `plugins/supi-flow/src/tools/flow-tools.ts` to look up the ticket's file path after creation and include it in the response text and details. The two existing tests need updating to mock `tndmJson` (which currently isn't mocked for `executeFlowStart` tests).

- [x] **Task 1**: Update `executeFlowStart` to look up ticket path and include it in output
  - File: `plugins/supi-flow/src/tools/flow-tools.ts`
  - Change: After the optional context update, call `tndmJson(["ticket", "show", ticketId, "--json"])` to get `content_path`, derive the ticket directory, and include it in the response text (`Created ticket TNDM-XXXXXX at .tndm/tickets/TNDM-XXXXXX/ ...`) and `details` (add `ticketPath` field).
  - Verification: `pnpm exec tsc --noEmit`

- [x] **Task 2**: Update tests to mock `tndmJson` for `executeFlowStart` tests and assert path in output
  - File: `plugins/supi-flow/__tests__/flow-tools.test.ts`
  - Change: Add `tndmJson` mock setup (returning a temp path as `content_path`) in both `executeFlowStart` tests. Assert the path appears in `result.content[0].text` and `result.details.ticketPath`.
  - Verification: `pnpm exec vitest run __tests__/flow-tools.test.ts`
