# Archive

- Removed `/supi-flow` and `/supi-flow-status` command registrations from `extensions/index.ts`
- Removed `checkTndmVersion` function, `FLOW_VERSION` export, and `session_start` event handler
- Removed unused imports (`readFileSync`, `dirname`, `join`, `fileURLToPath`) and `tndmJson`/`tndmVersion` imports from `extensions/index.ts`
- Deleted `__tests__/index.test.ts` (all tests were for removed features)
- Removed `session_start` handler assertion from `__tests__/resources.test.ts`
- Removed Commands table from `README.md`
- TypeScript: `pnpm exec tsc --noEmit` — no errors
- Tests: `pnpm exec vitest run` — 28 passed, 0 failed
