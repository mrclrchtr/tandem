# Archive

- Removed `/supi-flow` and `/supi-flow-status` commands from `extensions/index.ts`
- Removed `checkTndmVersion`, `FLOW_VERSION`, `session_start` handler, and all related imports from `extensions/index.ts`
- Rewrote `__tests__/index.test.ts` to only verify no commands are registered
- Updated `__tests__/resources.test.ts` to assert `session_start` is NOT registered
- Deleted `prompts/supi-coding-retro.md` and removed `prompts/` from `package.json` `files`
- Removed Commands and Prompt templates tables from `README.md`
- `pnpm exec tsc --noEmit` passes
- `pnpm exec vitest run` passes (30/30)
