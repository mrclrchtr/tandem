- [x] **Task 1**: Remove command registrations and handler code from `extensions/index.ts`
  - File: `plugins/supi-flow/extensions/index.ts`
  - Remove `pi.registerCommand("supi-flow-status", …)` block
  - Remove `pi.registerCommand("supi-flow", …)` block
  - Remove `checkTndmVersion` function and the `pi.on("session_start", …)` block
  - Remove `FLOW_VERSION` export and the `pkg` parsing block (no longer needed)
  - Remove unused imports: `readFileSync`, `dirname`, `join`, `fileURLToPath`
  - Keep tool registrations untouched
  - Verification: `pnpm exec tsc --noEmit`

- [x] **Task 2**: Remove command tests and version-check tests from `__tests__/index.test.ts`
  - File: `plugins/supi-flow/__tests__/index.test.ts`
  - Remove the entire `makePi` helper, `loadExtension` helper, `CommandHandler` and `RegisteredCommand` types
  - Remove the `supi-flow commands` describe block (all command tests)
  - Remove all `checkTndmVersion` tests and the `beforeAll`/`beforeEach` setup for them
  - Keep `resources.test.ts` untouched
  - Verification: `pnpm exec vitest run __tests__/index.test.ts` passes (file should be empty or nearly empty — if empty, delete it)

- [x] **Task 3**: Remove Commands section from README and update package.json files list if needed
  - File: `plugins/supi-flow/README.md`
  - Remove the Commands table and its heading
  - File: `plugins/supi-flow/package.json`
  - `files` array already does not include `__tests__/`; no change needed
  - Verification: visual diff of README is clean

- [x] **Task 4**: Run full test suite and type-check
  - Verification: `cd plugins/supi-flow && pnpm exec tsc --noEmit && pnpm exec vitest run`
  - All tests pass, no type errors
