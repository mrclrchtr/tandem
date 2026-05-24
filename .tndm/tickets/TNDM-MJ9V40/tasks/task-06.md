# Task 6: Add integration tests against real tndm CLI

## Goal

New file `__tests__/integration.test.ts` exercises the extension tools against the real `tndm` CLI in a temp repo. Gated by `TNDM_INTEGRATION_TEST=1` env var. Catches Rust-side JSON output contract changes that unit tests cannot detect.

## Files

- `__tests__/integration.test.ts` — new file

## Changes

### Test structure

```ts
import { describe, expect, it, beforeAll, afterAll } from "vitest";
import { mkdtempSync, rmSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { execFileSync } from "node:child_process";

// Gate: skip if env var not set
const describeIntegration = process.env.TNDM_INTEGRATION_TEST
    ? describe.sequential
    : describe.skip;

let repoRoot: string;
let ticketId: string;

describeIntegration("supi-flow integration", () => {
    beforeAll(() => {
        repoRoot = mkdtempSync(join(tmpdir(), "supi-flow-int-"));
        process.chdir(repoRoot);
        execFileSync("git", ["init"]);
        execFileSync("tndm", ["init"]);
    });

    afterAll(() => {
        rmSync(repoRoot, { recursive: true, force: true });
    });

    // Import tool functions dynamically (after repo setup)

    it("creates a ticket via executeFlowStart", async () => { ... });
    it("adds task with detail via executeTndmCli and verifies file on disk", async () => { ... });
    it("lists tasks and verifies shape", async () => { ... });
    it("edits task detail via executeFlowTask", async () => { ... });
    it("completes task and verifies status", async () => { ... });
    it("closes flow with verification results and verifies archive.md", async () => { ... });
});
```

### Tests

Each test:
1. Calls the real tool execute function (not mocked)
2. Verifies return value shape (matches expected contract)
3. For file-related operations, also reads the file from disk

Key assertions:
- `executeFlowStart` returns `{ ticketId, status: "todo", tags: "flow:brainstorm" }`
- `executeTndmCli task_add` with detail creates `tasks/task-01.md` on disk with the detail content
- `executeTndmCli task_list` returns array of `{ number, title, status, detail_path }`
- `executeFlowTask edit` with detail updates the file content
- `executeFlowCompleteTask` returns `{ completed: true }` and subsequent task_list shows `status: "done"`
- `executeFlowClose` writes `archive.md` on disk

No mocking of `cli.ts` or `ticket-helpers.ts` — purely integration.

## Verification (Test-exempt — harness integration)

Manual: `TNDM_INTEGRATION_TEST=1 pnpm exec vitest run __tests__/integration.test.ts`
Requires `tndm` on PATH and `git` available.

## Dependencies

Independent of Tasks 1-4 but should run after them to confirm the refactored code works end-to-end.
