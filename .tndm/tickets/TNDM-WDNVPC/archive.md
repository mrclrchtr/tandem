# Archive

## Verification Results

All tasks completed successfully.

### Files changed
- **plugins/supi-flow/extensions/index.ts**: Removed `pi.registerCommand("supi-flow-status", ...)` and `pi.registerCommand("supi-flow", ...)` blocks + unused `tndmJson` import
- **plugins/supi-flow/__tests__/index.test.ts**: Removed command tests (`makePi`, `loadExtension`, `CommandHandler`, `RegisteredCommand` types, and `describe("supi-flow commands", ...)` block) — kept all version-check tests
- **plugins/supi-flow/README.md**: Removed Commands table

### Verification
- Type-check: `pnpm exec tsc --noEmit` → no errors
- Tests: `pnpm exec vitest run` → 34/34 passed (5 in index.test.ts, 3 in resources.test.ts, 14 in cli.test.ts, 12 in flow-tools.test.ts)
