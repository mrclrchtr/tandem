supi_flow_plan writes to docs/plan.md (via doc registry) but supi_flow_complete_task reads from content.md — tasks can never be checked off. supi_flow_close also writes verification results to content.md instead of a separate archive.md. Tool descriptions in index.ts are stale and reference content.md. Additionally, ~/.cargo/bin/tndm is missing the doc subcommand.

Fix: align all tools to the three-file layout (content.md, docs/plan.md, docs/archive.md), update descriptions, rebuild binary.

## Verification Results

- Rust CLI: `tndm ticket doc create` now creates flat `{name}.md` at ticket root (verified with live test)
- `executeFlowStart`: writes context to `brainstorm.md` via doc registry (no `--content` flag)
- `executeFlowCompleteTask`: reads from `plan.md` instead of `content.md`
- `executeFlowClose`: writes verification results to `archive.md` via doc registry
- Tool descriptions in index.ts updated to reference correct files
- Skill SKILL.md files updated: `content.md` → `brainstorm.md` / `plan.md` / `archive.md`
- Binary: `tndm` installed from current source, `tndm ticket doc --help` succeeds
- TypeScript: `pnpm exec tsc --noEmit` — no errors
- Vitest: 28 tests pass, 0 fail
- Rust: 140 tests pass (16 suites)
