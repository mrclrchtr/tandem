- [x] **Task 1**: Add `tndmVersion()` to `cli.ts`
  - File: `plugins/supi-flow/src/cli.ts`
  - New exported function that runs `tndm --version`, parses version with regex, returns string or null
  - TDD: add test in `__tests__/cli.test.ts` for success and failure cases
  - Verification: `pnpm exec vitest run __tests__/cli.test.ts`

- [x] **Task 2**: Read supi-flow version at init in `index.ts`
  - File: `plugins/supi-flow/src/index.ts`
  - Read `package.json` once at module level, cache version as `FLOW_VERSION` constant
  - Use existing `baseDir` + `readFileSync` (add import)
  - Verification: `pnpm exec tsc --noEmit`

- [x] **Task 3**: Add `session_start` handler with version mismatch warning
  - File: `plugins/supi-flow/src/index.ts`
  - On `reason: "startup" | "reload"`, call `tndmVersion()`, compare with `FLOW_VERSION`
  - If mismatch, `ctx.ui.notify("warning")` with install instructions
  - TDD: add test in `__tests__/index.test.ts`
  - Verification: `pnpm exec vitest run __tests__/index.test.ts`

- [x] **Task 4**: Full regression
  - Verification: `pnpm exec tsc --noEmit && pnpm exec vitest run`
