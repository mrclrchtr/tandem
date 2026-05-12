
- [x] **Task 1**: Create `scripts/check-tndm.mjs` — postinstall check script
  - File: `plugins/supi-flow/scripts/check-tndm.mjs`
  - Runs `tndm --version` via `execFile`; on ENOENT, prints a clear warning with install instructions
  - Exits 0 always (must not break `npm install`)
  - Test-exempt: standalone script, no test harness. Manual verification: `node scripts/check-tndm.mjs`

- [x] **Task 2**: Update `package.json` to wire up postinstall and include `scripts/`
  - File: `plugins/supi-flow/package.json`
  - Add `"postinstall": "node scripts/check-tndm.mjs"` to `scripts`
  - Add `"scripts/"` to `files` array
  - Test-exempt: config change. Verification: `pnpm install` runs the script

- [x] **Task 3**: Improve runtime error in `cli.ts` when `tndm` is not found
  - File: `plugins/supi-flow/src/cli.ts`
  - In `tndm()` function, catch ENOENT from `run()` and throw a clear message with install instructions
  - TDD: add test in `__tests__/cli.test.ts` for the ENOENT error case
  - Verification: `pnpm exec vitest run __tests__/cli.test.ts`
