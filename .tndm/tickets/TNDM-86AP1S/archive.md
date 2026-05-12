# Archive

## Verification Results

### Changes Made

1. **Removed slop-detect automation scripts** (~1,100 lines across 5 files):
   - Deleted `skills/supi-flow-slop-detect/scripts/slop-scan.ts`
   - Deleted `skills/supi-flow-slop-detect/scripts/slop-scan-structural.ts`
   - Deleted `skills/supi-flow-slop-detect/scripts/slop-scan-vocab.ts`
   - Deleted `skills/supi-flow-slop-detect/scripts/slop-helpers.ts`
   - Deleted `skills/supi-flow-slop-detect/references/vocabulary.json`
   - Updated `SKILL.md` to remove all script references while keeping the manual workflow, vocabulary tiers, structural patterns, and density scoring formula as agent guidance.

2. **Removed `doc_create` and `sync` from `supi_tndm_cli` tool surface**:
   - Removed `"doc_create"` and `"sync"` from `actionEnum` in `extensions/tools/tndm-cli.ts`
   - Removed `name` parameter from the tool schema (was only used by `doc_create`)
   - Removed both cases from the `executeTndmCli` switch statement
   - Added JSDoc note that these are internal operations used by flow tools
   - Flow tools continue to call `tndm()` directly for these operations, so functionality is unchanged.

### Verification

- `pnpm exec tsc --noEmit` — **0 errors**
- `pnpm exec vitest run` — **36 pass, 0 fail**
- Confirmed no remaining references to deleted scripts in any SKILL.md or docs file
- Confirmed no remaining `doc_create`/`sync` references in README.md or CLAUDE.md
