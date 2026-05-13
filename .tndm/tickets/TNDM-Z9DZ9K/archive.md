# Archive

- Removed `/supi-flow` and `/supi-flow-status` command registrations from `extensions/index.ts`
- Removed Commands table from `README.md`
- Kept `checkTndmVersion` function, `FLOW_VERSION` export, and `session_start` event handler intact
- Kept `__tests__/index.test.ts` and `__tests__/resources.test.ts` unchanged (version-check tests remain valid)
- TypeScript: `pnpm exec tsc --noEmit` — no errors
- Tests: `pnpm exec vitest run` — 28 passed, 0 failed
