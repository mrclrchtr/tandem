## Implementation Plan

### File map
- `plugins/supi-flow/src/tools/flow-tools.ts` — make `content.md` the canonical durable ticket body, stop creating `brainstorm.md`, and resolve ticket documents from the registered `documents` metadata instead of inferred filenames.
- `plugins/supi-flow/src/index.ts` — align tool descriptions/guidelines with the new file model and replace `/supi-flow-status` session-history heuristics with a TNDM-backed active-flow summary.
- `plugins/supi-flow/__tests__/flow-tools.test.ts` — cover the new start/plan/complete/close behavior under the CLI-aligned document model.
- `plugins/supi-flow/__tests__/index.test.ts` — add command-level tests for `/supi-flow-status` and `/supi-flow` messaging.
- `plugins/supi-flow/skills/supi-flow-brainstorm/SKILL.md` — persist approved design to `content.md`, not `brainstorm.md`, and describe when tickets are required.
- `plugins/supi-flow/skills/supi-flow-plan/SKILL.md` — read the approved design from `content.md` and store executable tasks in `plan.md`.
- `plugins/supi-flow/skills/supi-flow-apply/SKILL.md` — read plan/doc paths accurately and describe status/tag transitions without assuming hidden file contents.
- `plugins/supi-flow/skills/supi-flow-archive/SKILL.md` — describe archive evidence in `archive.md` and final closeout against the updated model.
- `plugins/supi-flow/README.md` — document the canonical file model, phase flow, and `/supi-flow-status` behavior.
- `plugins/supi-flow/CLAUDE.md` — refresh maintainer guidance to match the new document responsibilities and verification expectations.
- `plugins/tndm/skills/ticket/references/command-reference.md` — fix CLI examples and file-layout guidance so the ticket document model matches current TNDM behavior.
- `docs/decisions.md` — update the document-registry design notes to describe the flat ticket-root document paths.
- `plugins/supi-flow/package.json` — bump the plugin version after behavior/docs changes.

- [x] **Task 1**: Write failing tests for the CLI-aligned document model and TNDM-backed status command
  - File: `plugins/supi-flow/__tests__/flow-tools.test.ts`
  - File: `plugins/supi-flow/__tests__/index.test.ts`
  - Change: Add RED coverage for these behaviors before changing production code: `supi_flow_start` seeds `content.md` instead of creating `brainstorm.md`; task completion resolves `plan.md` from registered ticket documents instead of deriving it from `content_path`; `/supi-flow-status` reports active flow tickets from TNDM data rather than scanning chat history.
  - Verification: `cd plugins/supi-flow && pnpm exec vitest run __tests__/flow-tools.test.ts __tests__/index.test.ts`
  - TDD: Required (watch the new assertions fail for the intended reasons before implementation).

- [x] **Task 2**: Implement the new ticket/document responsibilities and active-flow status lookup
  - File: `plugins/supi-flow/src/tools/flow-tools.ts`
  - File: `plugins/supi-flow/src/index.ts`
  - Change: Update `executeFlowStart` so known brainstorm context lands in `content.md` instead of a separate `brainstorm.md`; keep `content.md` as the durable approved-design summary while `plan.md` remains the executable checklist and `archive.md` remains verification evidence. Update task completion to read the registered `plan` document path from ticket metadata. Replace `/supi-flow-status` message scanning with a `tndm ticket list --json` query that surfaces active flow tickets (for example `flow:planned` / `flow:applying` and non-done statuses) and points the user at the next relevant step.
  - Verification: `cd plugins/supi-flow && pnpm exec vitest run __tests__/flow-tools.test.ts __tests__/index.test.ts`
  - TDD: Required (implement only after Task 1 is red).

- [x] **Task 3**: Align flow skills, plugin docs, and project docs to the single-source-of-truth model
  - File: `plugins/supi-flow/skills/supi-flow-brainstorm/SKILL.md`
  - File: `plugins/supi-flow/skills/supi-flow-plan/SKILL.md`
  - File: `plugins/supi-flow/skills/supi-flow-apply/SKILL.md`
  - File: `plugins/supi-flow/skills/supi-flow-archive/SKILL.md`
  - File: `plugins/supi-flow/README.md`
  - File: `plugins/supi-flow/CLAUDE.md`
  - File: `plugins/tndm/skills/ticket/references/command-reference.md`
  - File: `docs/decisions.md`
  - File: `plugins/supi-flow/package.json`
  - Change: Remove `brainstorm.md` guidance, make `content.md` the documented approved-design body, keep `plan.md` and `archive.md` as the only extra flow artifacts, document the TNDM-backed `/supi-flow-status` behavior, correct file-path examples that still mention `docs/<name>.md`, and bump the plugin version to reflect the behavior change.
  - Verification: `rg -n "brainstorm\.md|docs/<name>\.md|docs/plan\.md|read the design from brainstorm\.md|Store the implementation plan in the ticket's content\.md" plugins/supi-flow plugins/tndm docs`
  - Test exemption: Docs/config-only task; no reasonable automated harness for prose accuracy. Manual verification is the grep above returning no stale guidance.

- [x] **Task 4**: Run end-to-end plugin verification on the final model
  - File: `plugins/supi-flow/src/tools/flow-tools.ts`
  - File: `plugins/supi-flow/src/index.ts`
  - File: `plugins/supi-flow/__tests__/flow-tools.test.ts`
  - File: `plugins/supi-flow/__tests__/index.test.ts`
  - File: `plugins/supi-flow/README.md`
  - Change: Run the full targeted verification suite for the final code/docs state, fix any fallout, and confirm the published workflow now consistently describes `content.md` + `plan.md` + `archive.md` with TNDM-backed status reporting.
  - Verification: `cd plugins/supi-flow && pnpm exec tsc --noEmit && pnpm exec vitest run`
  - Test exemption: This is a verification-and-closeout task rather than new production code.

### Notes for execution
- Do not migrate or delete existing `brainstorm.md` files in old tickets unless the implementation proves that is necessary; the goal is to stop creating new ones and stop depending on them.
- Keep the phase model `brainstorm -> plan -> apply -> archive`; this change is about file ownership, agent ergonomics, and source-of-truth alignment, not a broader workflow-engine rewrite.
- `content.md` should act as the durable approved design / handoff summary, not as a duplicate task checklist; execution state should stay in `plan.md`, ticket status, and flow tags.
