# Archive

Implemented version mismatch warning in supi-flow:

**Changes:**
- `src/cli.ts` — added `tndmVersion()`: runs `tndm --version`, parses semver with regex `/tndm\s+(\d+\.\d+\.\d+)/`, returns string or null (never throws)
- `src/index.ts` — added `FLOW_VERSION` read from `package.json` at module init. `session_start` handler calls `checkTndmVersion()` on startup/reload. Warns via `ctx.ui.notify()` when `tndm` version differs from supi-flow version. Silent when they match or when tndm is not installed.

**Fresh verification (all run in this session):**
- `pnpm exec tsc --noEmit` — no errors
- `pnpm exec vitest run __tests__/cli.test.ts` — 14/14 pass (3 new: parse version, null on bad output, null on ENOENT)
- `pnpm exec vitest run __tests__/index.test.ts` — 7/7 pass (5 new: skip non-startup, warn on mismatch startup, warn on mismatch reload, silent on match, silent when tndm missing)
- `pnpm exec vitest run` — 39/39 pass
