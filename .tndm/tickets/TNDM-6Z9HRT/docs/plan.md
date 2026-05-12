## Implementation Plan

Target ticket file layout:

```
.tndm/tickets/TNDM-XXXXXX/
├── brainstorm.md     ← supi_flow_start
├── plan.md           ← supi_flow_plan → read by supi_flow_complete_task
├── archive.md        ← supi_flow_close
├── meta.toml
└── state.toml
```

### Tasks

- [x] **Task 1**: Change doc path derivation in Rust CLI from `docs/{name}.md` to `{name}.md`
  - File: `crates/tandem-cli/src/main.rs`
  - Change: `let rel_path = format!("docs/{name}.md");` → `let rel_path = format!("{name}.md");`
  - Also adjust `fs::create_dir_all(parent)` — no longer needed for flat files, remove or guard
  - Verification: `cargo build -p tandem-cli && ./tndm-dev ticket doc create TNDM-TEST plan` creates `plan.md` not `docs/plan.md`

- [x] **Task 2**: Fix `executeFlowStart` to write context to `brainstorm.md` instead of `content.md`
  - File: `plugins/supi-flow/src/tools/flow-tools.ts`
  - Change: after ticket creation, call `tndm ticket doc create <id> brainstorm`, write context there, sync
  - Remove `--content` flag usage
  - Verification: `cd plugins/supi-flow && pnpm exec tsc --noEmit && pnpm exec vitest run`

- [x] **Task 3**: Fix `executeFlowCompleteTask` to read from `plan.md` instead of `content.md`
  - File: `plugins/supi-flow/src/tools/flow-tools.ts`
  - Change: resolve `plan.md` path from ticket dir instead of `content_path`
  - Keep `tndm ticket sync` after write
  - Verification: `cd plugins/supi-flow && pnpm exec tsc --noEmit && pnpm exec vitest run`

- [x] **Task 4**: Fix `executeFlowClose` to write verification results to `archive.md`
  - File: `plugins/supi-flow/src/tools/flow-tools.ts`
  - Change: call `tndm ticket doc create <id> archive`, write results there, sync
  - Remove append-to-content.md logic
  - Verification: `cd plugins/supi-flow && pnpm exec tsc --noEmit && pnpm exec vitest run`

- [x] **Task 5**: Update tool descriptions in `index.ts`
  - File: `plugins/supi-flow/src/index.ts`
  - `supi_flow_plan`: `"content.md"` → `"plan.md"`
  - `supi_flow_complete_task`: `"content.md"` → `"plan.md"`
  - `supi_flow_close`: `"content"` → `"archive.md"`
  - `supi_flow_start`: mention `brainstorm.md`
  - Verification: `cd plugins/supi-flow && pnpm exec tsc --noEmit`

- [x] **Task 6**: Update skill SKILL.md references
  - File: `plugins/supi-flow/skills/supi-flow-apply/SKILL.md` — `"content.md"` → `"plan.md"`
  - File: `plugins/supi-flow/skills/supi-flow-archive/SKILL.md` — `"content.md"` → `"archive.md"`, `"store verification results in content.md"` → `"store verification results in archive.md"`
  - Verification: `rg 'content\.md' plugins/supi-flow/skills/` returns no matches

- [x] **Task 7**: Handle stale `tndm` binary
  - The installed `~/.cargo/bin/tndm` lacks the `doc` subcommand
  - Command: `cargo install --path crates/tandem-cli --bin tndm --force`
  - Verification: `tndm ticket doc --help` succeeds
