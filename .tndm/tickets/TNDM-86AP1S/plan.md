- [x] **Task 1**: Remove slop-detect automation scripts and references directory
  - Delete `plugins/supi-flow/skills/supi-flow-slop-detect/scripts/slop-scan.ts`
  - Delete `plugins/supi-flow/skills/supi-flow-slop-detect/scripts/slop-scan-structural.ts`
  - Delete `plugins/supi-flow/skills/supi-flow-slop-detect/scripts/slop-scan-vocab.ts`
  - Delete `plugins/supi-flow/skills/supi-flow-slop-detect/scripts/slop-helpers.ts`
  - Delete `plugins/supi-flow/skills/supi-flow-slop-detect/references/vocabulary.json`
  - Delete empty `scripts/` and `references/` directories
  - Verification: `fd . plugins/supi-flow/skills/supi-flow-slop-detect/scripts/ 2>/dev/null; fd . plugins/supi-flow/skills/supi-flow-slop-detect/references/ 2>/dev/null` — should return nothing

- [x] **Task 2**: Update slop-detect SKILL.md to remove script references, keep manual workflow
  - File: `plugins/supi-flow/skills/supi-flow-slop-detect/SKILL.md`
  - Remove the "Scan workflow" step numbers that reference automated scanning
  - Keep the principles, document profiles, and vocabulary tiers — these are still useful manual guidance
  - Rewrite step 4 (run scan) to be a manual agent-driven check instead of calling scripts
  - Verification: read the updated SKILL.md and confirm it reads as manual guidance with no script invocations

- [x] **Task 3**: Remove doc_create and sync from supi_tndm_cli tool surface
  - File: `plugins/supi-flow/extensions/tools/tndm-cli.ts`
  - Remove `"doc_create"` and `"sync"` from `actionEnum` array
  - Remove `name` parameter from schema (only used by doc_create)
  - Remove `doc_create` and `sync` cases from the `executeTndmCli` switch
  - Remove the `addOptionalFlags` call for `name` in doc_create handling
  - Verification: `pnpm exec tsc --noEmit` passes

- [x] **Task 4**: Update README.md and CLAUDE.md to remove references to removed features
  - File: `plugins/supi-flow/README.md`
  - File: `plugins/supi-flow/CLAUDE.md`
  - Update README: remove slop-detect script references, update tndm-cli action list, remove supi-coding-retro from prompt templates (bonus cleanup from analysis)
  - Update CLAUDE.md: remove slop-detect script references, update tndm-cli action list
  - Verification: `rg "doc_create|sync" plugins/supi-flow/README.md plugins/supi-flow/CLAUDE.md` returns nothing (for these actions)

- [x] **Task 5**: Run full test suite and type check
  - Command: `cd plugins/supi-flow && pnpm exec tsc --noEmit`
  - Command: `cd plugins/supi-flow && pnpm exec vitest run`
  - Verification: All tests pass, zero type errors
