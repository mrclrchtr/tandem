# Task 7: Doc dedup — trim overlapping skill conventions from CLAUDE.md

## Goal

Remove the "Skill conventions" section from `CLAUDE.md` — it duplicates content already present in the individual `skills/*/SKILL.md` files. Replace with a short pointer.

## Files

- `plugins/supi-flow/CLAUDE.md` — trim one section

## Changes

### Current "Skill conventions" section (~lines 127-146)

This section lists rules about:
- Skills live in `skills/<name>/SKILL.md` and are auto-discovered
- Skills reference tools with structured parameter examples
- `content.md` is canonical overview, tasks in `state.toml`
- Each task gets `tasks/task-XX.md` detail doc
- `supi_flow_plan` stores overview, task authoring separately
- `supi_flow_apply` transitions planned to applying
- `supi_flow_close` requires verification results
- `supi_tndm_cli` task_* actions are escape hatches

All of this already lives in the individual skill files and README.md. The CLAUDE.md section re-describes it.

### Replace with

```markdown
## Skill conventions

See `skills/*/SKILL.md` for per-phase agent instructions (brainstorm, plan, apply, archive, debug).
The workflow overview and tool descriptions live in `README.md`.
```

Everything else in CLAUDE.md stays: Purpose, PI guardrails, Relationship to tandem repo, File structure, Development commands, Coding conventions, Tool registration pattern, Verification shortcuts, When changing this plugin.

## Verification (Test-exempt — docs only)

1. `grep -c "content.md is the canonical" plugins/supi-flow/CLAUDE.md` returns 0
2. `grep -c "state.toml" plugins/supi-flow/CLAUDE.md` returns 0 (except if mentioned in file structure)
3. Visual check: CLAUDE.md still has Purpose, PI guardrails, File structure, Dev commands, Coding conventions, Tool registration, Verification shortcuts, When changing this plugin
4. Visual check: README.md still has full workflow description, tool table, phase table, document model

## Dependencies

Independent. Can run at any point.
