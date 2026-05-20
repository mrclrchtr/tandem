# Archive

## Verification Results

### Fresh regression suite (all passing)
- `cargo test --workspace --locked`: 167 passed, 1 ignored
- `cd plugins/supi-flow && pnpm exec tsc --noEmit`: No errors
- `cd plugins/supi-flow && pnpm exec vitest run`: 33 passed, 0 failed

### End-to-end workflow verification
A fresh test ticket (TNDM-690P8K) was created and verified through the full lifecycle:

#### 1. Overview-first `supi_flow_plan` ✓
- Accepted pure overview markdown with zero task blocks
- Stored it in `content.md` without parsing for `**Task N**`
- Tags transitioned to `flow:planned`

#### 2. Headline-only tasks (manifest-only) ✓
- `supi_tndm_cli {action:"task_add"}` with only a title creates task in `state.toml`
- No `tasks/` directory created
- No `detail_path` in manifest

#### 3. Detailed tasks (with linked doc) ✓
- `supi_tndm_cli {action:"task_add", task_detail}` creates task + links canonical `tasks/task-XX.md`
- `detail_path` linked in `state.toml`
- Document registered in `meta.toml` (once, no duplicates)
- File fingerprints computed and stored

#### 4. Non-destructive clear ✓
- `supi_tndm_cli {action:"task_edit", task_clear_detail:true}` removes `detail_path` from state
- Task doc file and meta registration preserved

#### 5. Idempotent re-attach ✓
- Re-attaching detail after clear works correctly
- Content updated, no duplicate document registrations
- `detail_path` restored in manifest

#### 6. CLI lifecycle primitives ✓
- `tndm ticket task detail ensure`: creates/links canonical doc, returns path
- `tndm ticket task detail clear`: detaches link non-destructively
- Path validation rejects absolute paths and `..` traversal

#### 7. Closeout ✓
- `supi_flow_close` sets status=done, tags=flow:done
- `archive.md` written with evidence

### Documentation accuracy
- `docs/decisions.md` — task detail lifecycle rules match CLI
- `plugins/supi-flow/README.md` — tool descriptions match new overview-first model
- `plugins/supi-flow/CLAUDE.md` — conventions match architecture
- `plugins/supi-flow/skills/supi-flow-plan/SKILL.md` — overview-first guidance correct
- `plugins/supi-flow/skills/supi-flow-apply/SKILL.md` — linked task doc guidance correct
