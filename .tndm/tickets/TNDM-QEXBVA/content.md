## Overview

Introduce a new high-level `supi_flow_task` tool for plan-time task authoring in `plugins/supi-flow/`, while keeping the current low-level `supi_tndm_cli` task actions as escape hatches. The new tool should manage exactly one task per call (`add`, `edit`, `remove`), hide raw `task_json` and `detail_path` from the common path, and automatically handle canonical task detail docs when detail markdown is supplied.

## Implementation shape

- `plugins/supi-flow/extensions/tools/flow-tools.ts`
  - add the new `supi_flow_task` schema and execute function
  - support single-task `add | edit | remove`
  - on add/edit, bridge optional detail markdown through the canonical task-detail lifecycle
- `plugins/supi-flow/extensions/index.ts`
  - register the new tool with accurate description and prompt guidance
  - keep `supi_flow_complete_task` as the apply-phase completion primitive
- `plugins/supi-flow/extensions/tools/tndm-cli.ts`
  - align direct wrapper behavior with current `tndm` JSON contracts, especially `ticket list --json`
- `plugins/supi-flow/__tests__/flow-tools.test.ts`, `plugins/supi-flow/__tests__/tndm-cli-tool.test.ts`, `plugins/supi-flow/__tests__/resources.test.ts`
  - add regression coverage for the new tool and wrapper hardening
- `plugins/supi-flow/skills/supi-flow-plan/SKILL.md`
  - make the new tool the common plan-time task-authoring path after `supi_flow_plan`
- `plugins/supi-flow/skills/supi-flow-brainstorm/SKILL.md`, `plugins/supi-flow/README.md`, `plugins/supi-flow/CLAUDE.md`, `README.md`, `CLAUDE.md`
  - reflect the 6-tool model and the overview-first + manifest-first task workflow
- `plugins/supi-flow/package.json`
  - bump the plugin version for the added tool

## Key rules

- `content.md` remains the approved overview
- `state.toml` remains the only source of truth for task existence
- `tasks/task-XX.md` stays optional and canonical
- no bulk flow-authoring abstraction in this pass
- no renumber/reorder abstraction in this pass

## Verification strategy

Use focused plugin tests while implementing, then finish with:
- `cd plugins/supi-flow && pnpm exec tsc --noEmit`
- `cd plugins/supi-flow && pnpm exec vitest run`
