## Overview

Fix the three approved review findings without changing the broader ticket/task model.

### Scope
- Prevent stale canonical task detail docs from surviving task removal or task-set replacement in ways that let old detail content bind to a different task later.
- Make `supi_tndm_cli` return the final post-mutation task state after any task-detail ensure/clear follow-up work.
- Make `tndm ticket doc create --path ...` reject requests whose requested path disagrees with an already-registered document name.

### Files
- `crates/tandem-cli/src/cli/ticket.rs` — prune canonical task detail docs and fingerprints when tasks are removed or replaced.
- `crates/tandem-cli/src/cli/doc.rs` — validate `doc create` name/path combinations consistently.
- `crates/tandem-cli/tests/ticket_cli_tests.rs` — CLI regressions for remove/set/doc-path behavior.
- `plugins/supi-flow/extensions/tools/tndm-cli.ts` — return post-follow-up final state for `task_add` / `task_edit`.
- `plugins/supi-flow/__tests__/tndm-cli-tool.test.ts` — wrapper regressions for final-state behavior.

### Verification
- `cargo test -p tandem-cli --test ticket_cli_tests`
- `cd plugins/supi-flow && pnpm exec vitest run __tests__/tndm-cli-tool.test.ts`

## Notes
Prefer the smallest change that restores correct lifecycle behavior and keeps the CLI/plugin contract deterministic.
