# Archive

Implemented improved runtime error when `tndm` is not found:
- `src/cli.ts`: `tndm()` catches ENOENT from `execFile` and throws a clear message with install instructions
- `__tests__/cli.test.ts`: added test for ENOENT error case
- Postinstall check was removed per user direction after reconsideration

Verification:
- `pnpm exec tsc --noEmit` — no errors
- `pnpm exec vitest run` — 31/31 pass
