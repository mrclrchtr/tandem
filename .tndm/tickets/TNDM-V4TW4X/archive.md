# Archive

# Archive: Optimize supi-flow tool architecture and runtime behavior

## Changes made

### New files
- `plugins/supi-flow/extensions/tools/tool-specs.ts` — single source of truth for all 7 public tool definitions (name, label, description, snippets, guidelines, execution mode, params, execute binding)
- `plugins/supi-flow/extensions/tools/doc-writes.ts` — shared async helper wrapping `withFileMutationQueue()` for task-detail and archive markdown writes

### Modified files
- `plugins/supi-flow/extensions/index.ts` — refactored from 7 inline `pi.registerTool()` calls to a loop over shared tool specs; preserves version-check wiring
- `plugins/supi-flow/extensions/cli.ts` — `tndm()`, `tndmJson()`, `tndmVersion()` now accept an optional `AbortSignal` and forward it to `execFile`
- `plugins/supi-flow/extensions/tools/tndm-cli.ts` — `executeTndmCli()` accepts `AbortSignal`; all CLIs calls are signal-aware; large model-facing JSON output is truncated via `truncateHead`/`formatSize` while full payloads remain in `details`; old sync `writeTaskDetailDoc` replaced with shared queued helper
- `plugins/supi-flow/extensions/tools/flow-tools.ts` — all 6 flow execute functions accept `AbortSignal` and thread it through CLIs calls; sync `writeFileSync(archive.md)` replaced with shared queued `writeArchiveDoc()`; task-detail writes use shared queued helper
- `plugins/supi-flow/__tests__/cli.test.ts` — added signal-forwarding test
- `plugins/supi-flow/__tests__/resources.test.ts` — added sequential execution-mode assertion for `supi_tndm_cli`
- `plugins/supi-flow/__tests__/tndm-cli-tool.test.ts` — added truncation test
- `plugins/supi-flow/__tests__/flow-tools.test.ts` — added Proxy mock wrappers to handle trailing-undefined signal arg transparently

### Doc updates
- `plugins/supi-flow/CLAUDE.md` — file structure updated to list tool-specs.ts, doc-writes.ts, and tndm-cli-tool.test.ts
- `plugins/supi-flow/package.json` — description corrected from "6" to "7" custom tools

## Verification evidence

### TypeScript compilation
```sh
cd plugins/supi-flow && pnpm exec tsc --noEmit
```
Result: No errors found (2026-05-24)

### Full test suite
```sh
cd plugins/supi-flow && pnpm exec vitest run
```
Result: PASS (51) — all 51 tests pass across 5 test files:

- `__tests__/index.test.ts` — version-check session_start registration intact
- `__tests__/resources.test.ts` — 7 tools registered, sequential mode on supi_tndm_cli, apply guidance preserved
- `__tests__/cli.test.ts` — tndm/tndmJson/tndmVersion parsing, ENOENT handling, signal forwarding (12 tests)
- `__tests__/tndm-cli-tool.test.ts` — list envelope, task_add/ task_edit detail docs, truncation (5 tests)
- `__tests__/flow-tools.test.ts` — flow start/plan/apply/task/complete/close with queued doc writes (29 tests)

### Intact public contract
- All 7 tool names, labels, descriptions, prompt snippets, and prompt guidelines preserved
- Version-check behavior unchanged
- Tool return shapes preserved: `content` + `details`
- All existing workflow semantics (brainstorm → plan → apply → archive) unchanged
