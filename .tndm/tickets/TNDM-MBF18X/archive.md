# Archive

## Verification summary

Implemented the planned prompt-contract cleanup in `plugins/supi-flow/` by tightening always-on tool metadata to routing + guardrails, preserving all 7 tools and all `promptSnippet`s, compressing selected parameter descriptions, and locking the new contract in registration tests.

## Fresh verification evidence

### Task 1 — Define the compact always-on prompt contract in registration tests before changing tool copy
- Command: `cd plugins/supi-flow && RTK_DISABLED=1 pnpm exec vitest run __tests__/resources.test.ts -v`
- Result: passed fresh (`1` test file, `6` tests passed).
- Evidence: the registration test now inspects tool `description`, `promptSnippet`, `promptGuidelines`, and selected parameter descriptions, confirming the compact contract exists in the current tree.
- Historical TDD note from apply: this same test command was run before the metadata rewrite and failed as expected against the old verbose copy, then passed after the implementation.

### Task 2 — Rewrite supi-flow tool metadata and schema descriptions to routing + guardrails only
- Commands:
  - `cd plugins/supi-flow && RTK_DISABLED=1 pnpm exec vitest run __tests__/resources.test.ts -v`
  - `cd plugins/supi-flow && pnpm exec tsc --noEmit`
- Result: both passed fresh.
- Evidence: the compact registration contract passes against the rewritten metadata and compressed parameter descriptions, and TypeScript reports `No errors found`.

### Task 3 — Run final supi-flow verification and review the diff for lost guidance
- Command: `cd plugins/supi-flow && RTK_DISABLED=1 pnpm exec vitest run __tests__/index.test.ts __tests__/resources.test.ts -v`
- Result: passed fresh (`2` test files, `11` tests passed).
- Command: `git diff --stat -- plugins/supi-flow`
- Result: fresh diff shows only:
  - `plugins/supi-flow/__tests__/resources.test.ts`
  - `plugins/supi-flow/extensions/tools/tool-specs.ts`
  - `plugins/supi-flow/extensions/tools/tndm-cli.ts`
  - `plugins/supi-flow/extensions/tools/flow-tools.ts`
  - aggregate diffstat: `4 files changed, 155 insertions(+), 78 deletions(-)`.

## Docs / guidance review

- Reviewed README tool references and skill guidance anchors after the final diff.
- No doc update was required because the change does not alter tool names, workflow phases, file paths, or runtime behavior; it only shrinks always-on prompt copy and parameter descriptions.
- Verified that detailed guidance removed from always-on tool metadata still exists in the skill layer:
  - apply task-detail reading guidance remains in `plugins/supi-flow/skills/supi-flow-apply/SKILL.md`
  - overview-vs-task separation remains in `plugins/supi-flow/skills/supi-flow-plan/SKILL.md`
  - trivial/non-trivial and direct-implementation guidance remains in `plugins/supi-flow/skills/supi-flow-brainstorm/SKILL.md`

## Final state

- All three planned tasks are complete.
- Targeted tests and type-check pass.
- The diff matches the intended scope with no additional runtime or documentation changes required.
