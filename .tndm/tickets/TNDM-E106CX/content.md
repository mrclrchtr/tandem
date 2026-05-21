## Overview

Fix `supi-flow` task helper parsing so it matches the real current TNDM JSON contracts observed in end-to-end validation. The helpers should accept the real top-level ticket envelope shape with a top-level `tasks` array, while remaining tolerant of the older nested mock shape used in existing tests. This should restore `supi_flow_task add` and protect the lower-level `supi_tndm_cli` task-detail path from the same bug.

## Files

- `plugins/supi-flow/extensions/tools/flow-tools.ts` — fix task extraction/title lookup for top-level ticket envelopes.
- `plugins/supi-flow/extensions/tools/tndm-cli.ts` — apply the same helper fix for lower-level task add/edit flows.
- `plugins/supi-flow/__tests__/flow-tools.test.ts` — regressions using the real top-level task envelope shape.
- `plugins/supi-flow/__tests__/tndm-cli-tool.test.ts` — regressions for lower-level wrapper behavior with the real top-level task envelope shape.

## Verification

- `cd plugins/supi-flow && pnpm exec tsc --noEmit && pnpm exec vitest run __tests__/flow-tools.test.ts __tests__/tndm-cli-tool.test.ts`
- run a fresh end-to-end scratch validation of `supi_flow_task add` against the reloaded extension after the code/tests are green
