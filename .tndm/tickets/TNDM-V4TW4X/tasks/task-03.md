# Task 3: Truncate large supi_tndm_cli model-facing output

## Goal
Prevent `supi_tndm_cli` from flooding model context with large JSON or raw stdout while preserving the full structured payload in `details` for UI/state consumers.

## Files
- `plugins/supi-flow/extensions/tools/tndm-cli.ts`
- `plugins/supi-flow/__tests__/tndm-cli-tool.test.ts`

## Test strategy
**Test-driven.** Add a truncation-focused tool test before changing the content formatting.

## Change to make
1. Import PI truncation helpers in `plugins/supi-flow/extensions/tools/tndm-cli.ts` and add one local formatter that converts raw text or JSON strings into bounded model-facing `content`.
2. Use the formatter for actions that currently stringify large results, including `show`, `list`, `awareness`, and the task actions that emit full ticket/task JSON.
3. Keep the complete parsed object in `details` exactly as needed for tests, rendering, and branch-aware state reconstruction.
4. Include a truncation notice in `content` when output exceeds the PI limits so the model knows it only received a partial view.
5. Do not truncate the structured `details` payload.

## RED
- Add a representative large-output case to `plugins/supi-flow/__tests__/tndm-cli-tool.test.ts` that feeds a big ticket/task payload through `executeTndmCli()` and asserts:
  - `content[0].text` contains a truncation notice
  - `details` still contains the full untruncated object
- Run the focused tool tests and watch the new truncation assertion fail before changing the implementation.

## GREEN
- Implement bounded content formatting in `plugins/supi-flow/extensions/tools/tndm-cli.ts`.
- Keep the existing result structure for current tests and callers.
- Make the new truncation test pass without regressing the existing task-detail tests.

## REFACTOR
- Centralize the truncation behavior inside `plugins/supi-flow/extensions/tools/tndm-cli.ts` instead of scattering per-action string handling.
- Keep action routing readable; do not rewrite the whole switch unless needed.

## Verification
```sh
cd plugins/supi-flow
pnpm exec vitest run __tests__/tndm-cli-tool.test.ts
```
Expected result: the new truncation coverage passes and the existing `supi_tndm_cli` behavior tests remain green.
