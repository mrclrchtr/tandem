# Task 2: Make tndm wrapper calls abort-aware

## Goal
Allow long-running `tndm` subprocess calls to honor the PI tool abort signal and thread that signal through both tool-execution modules without changing current error messages or return shapes.

## Files
- `plugins/supi-flow/extensions/cli.ts`
- `plugins/supi-flow/extensions/tools/tndm-cli.ts`
- `plugins/supi-flow/extensions/tools/flow-tools.ts`
- `plugins/supi-flow/__tests__/cli.test.ts`

## Test strategy
**Test-driven.** Add CLI-wrapper coverage before changing the subprocess wrapper.

## Change to make
1. Update `plugins/supi-flow/extensions/cli.ts` so the internal `run()` helper accepts an optional `AbortSignal` and forwards it to `execFile`.
2. Keep the existing timeout behavior and the helpful ENOENT message intact.
3. Update `tndm()`, `tndmJson()`, and `tndmVersion()` to accept an optional signal and pass it through to `run()`.
4. Update the execute helpers in `plugins/supi-flow/extensions/tools/tndm-cli.ts` and `plugins/supi-flow/extensions/tools/flow-tools.ts` so every `tndm`/`tndmJson` call receives the tool's `signal` argument.
5. Do not create a second subprocess helper path; keep `plugins/supi-flow/extensions/cli.ts` as the single wrapper layer.

## RED
- Extend `plugins/supi-flow/__tests__/cli.test.ts` with a case that calls `tndm(..., signal)` and asserts `execFile` receives that signal in its options.
- Keep the existing ENOENT, empty-output, invalid-JSON, and version-parsing tests in place so the refactor cannot silently change behavior.
- Run the focused test file and watch the new signal-forwarding assertion fail before changing the implementation.

## GREEN
- Implement optional signal support in `plugins/supi-flow/extensions/cli.ts`.
- Thread the signal through all `tndm` call sites in `plugins/supi-flow/extensions/tools/tndm-cli.ts` and `plugins/supi-flow/extensions/tools/flow-tools.ts`.
- Keep the existing user-facing error text stable.

## REFACTOR
- Reuse the existing CLI wrapper signatures instead of adding parallel helper names.
- Keep the call-site changes mechanical and localized to the two tool modules.

## Verification
```sh
cd plugins/supi-flow
pnpm exec vitest run __tests__/cli.test.ts
pnpm exec tsc --noEmit
```
Expected result: CLI tests pass, the forwarded-signal assertion is green, and TypeScript confirms the updated helper signatures and call sites are wired correctly.
