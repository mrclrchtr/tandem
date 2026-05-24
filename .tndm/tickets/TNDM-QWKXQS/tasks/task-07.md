# Task 7: Create ticket-helpers.test.ts with resolveTicketPath and findRepoRoot tests

Create `__tests__/ticket-helpers.test.ts` with unit tests for `resolveTicketPath` and `findRepoRoot`.

The test file should:
- Mock `node:fs` (`existsSync`) and `node:path` (`resolve`, `dirname`, `join`) as needed, OR use real temp directories via `mkdtempSync`
- Prefer real temp directories (matches the pattern used in `flow-tools.test.ts` and `tndm-cli-tool.test.ts`)

Test cases:

1. **`findRepoRoot` finds root via `.git` directory** — create a temp dir with `.git` subdir, chdir into a nested subdir, verify it finds the root
2. **`findRepoRoot` finds root via `.tndm` directory** — create a temp dir with `.tndm` subdir (no `.git`), verify it finds the root
3. **`findRepoRoot` throws when no root found** — create a temp dir with neither `.git` nor `.tndm`, verify it throws
4. **`findRepoRoot` caches result** — call twice, verify the second call doesn't re-traverse (either by checking it returns same value or by spying on `existsSync`)

5. **`resolveTicketPath` resolves relative path** — create a repo-root temp dir with `.git`, create a file at `.tndm/tickets/TEST/content.md`, verify `resolveTicketPath(".tndm/tickets/TEST/content.md")` returns the absolute path
6. **`resolveTicketPath` passes through absolute paths** — verify `/absolute/path/to/file` is returned unchanged

Use `process.chdir` to set working directory for tests, and restore in `finally` or via `beforeEach`/`afterEach`.

**Verification**: `pnpm exec vitest run __tests__/ticket-helpers.test.ts` — all 6 test cases pass.
