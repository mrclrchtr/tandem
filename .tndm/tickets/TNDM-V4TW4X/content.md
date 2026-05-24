# Implementation Plan

## Overview
Optimize `plugins/supi-flow` so its public PI tool surface is derived from one metadata module and its runtime behavior matches the key PI tool-guidance expectations for ordered mutation execution, abort-aware CLI calls, truncated model-facing output, and queued file writes. Keep the brainstorm/plan/apply/archive workflow semantics, tool names, and version-check behavior unchanged.

## Constraints
- Scope is limited to `plugins/supi-flow/`.
- Keep `extensions/index.ts` as a thin PI adapter with version-check wiring.
- Do not redesign the workflow or add unrelated UI/renderer work.
- No Rust `tndm` CLI changes unless implementation reveals a hard blocker.
- No README or skill-doc edits are expected unless the implementation forces a public contract wording change.

## File map
- `plugins/supi-flow/extensions/index.ts` — retain startup/reload version-check wiring and register tools from shared definitions instead of repeated inline blocks.
- `plugins/supi-flow/extensions/cli.ts` — make the `execFile` wrapper accept and forward `AbortSignal` while preserving current timeout and ENOENT handling.
- `plugins/supi-flow/extensions/tools/tool-specs.ts` — new single source of truth for the seven public tools: metadata, schema, execution mode, and execute binding.
- `plugins/supi-flow/extensions/tools/tndm-cli.ts` — keep the action router, add model-facing truncation, and adopt signal-aware CLI calls.
- `plugins/supi-flow/extensions/tools/flow-tools.ts` — keep flow semantics, thread signal-aware CLI calls, and replace direct markdown writes with queued writes.
- `plugins/supi-flow/extensions/tools/doc-writes.ts` — new shared helper for queue-aware task-detail and archive markdown writes on the real target path returned by `tndm`.
- `plugins/supi-flow/__tests__/resources.test.ts` — lock tool registration metadata that must survive the refactor.
- `plugins/supi-flow/__tests__/index.test.ts` — keep version-check registration behavior covered.
- `plugins/supi-flow/__tests__/cli.test.ts` — cover abort-aware CLI wrapper behavior alongside existing parsing/error cases.
- `plugins/supi-flow/__tests__/tndm-cli-tool.test.ts` — cover truncation behavior and task-detail write behavior after the async refactor.
- `plugins/supi-flow/__tests__/flow-tools.test.ts` — cover queued task-detail/archive writes after the async refactor.

## Ordered work
1. Centralize the seven tool definitions in `extensions/tools/tool-specs.ts` and refactor `extensions/index.ts` to register tools from that shared metadata while preserving current public guidance and execution modes.
2. Make `extensions/cli.ts` abort-aware and thread the PI tool signal through `extensions/tools/tndm-cli.ts` and `extensions/tools/flow-tools.ts` so every `tndm`/`tndmJson` call can be cancelled cleanly.
3. Add bounded truncation for large `supi_tndm_cli` model-facing output in `extensions/tools/tndm-cli.ts` while preserving the full structured payload in `details`.
4. Replace direct task-detail and archive markdown writes with a shared `withFileMutationQueue()` helper used by both tool modules, then run the focused plugin validation sweep.

## Verification strategy
Run focused checks during each task, then finish with:

```sh
cd plugins/supi-flow
pnpm exec tsc --noEmit
pnpm exec vitest run __tests__/index.test.ts __tests__/resources.test.ts __tests__/cli.test.ts __tests__/tndm-cli-tool.test.ts __tests__/flow-tools.test.ts
```

## Task authoring notes
- Use TDD for the code changes: write or tighten the relevant failing test before changing implementation.
- Preserve current public tool names and return-shape expectations unless a guidance-driven fix requires a narrowly scoped update.
- Keep structured state in `details`; only the model-facing `content` should be truncated.
- Queue writes on the actual path returned by `tndm`, not on a guessed relative path.

## Self-review
- Coverage: the plan maps metadata centralization, ordered execution metadata, abort-aware CLI calls, truncation, queued writes, and focused verification to explicit work.
- Placeholder scan: no vague or deferred tasks remain.
- Consistency: file paths and responsibilities align with the approved design.
- Right-sized detail: the plan is explicit enough to execute without turning into a rewrite spec.
