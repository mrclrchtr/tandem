# Task 1: Update apply skill and README guidance for lazy task-detail loading

## Goal
Align the human-readable apply guidance so it is explicit that agents review the approved overview and full task manifest at apply start, but defer task-detail-doc reads until the active task begins.

## Files
- `plugins/supi-flow/skills/supi-flow-apply/SKILL.md`
- `plugins/supi-flow/README.md`

## Changes
1. In `plugins/supi-flow/skills/supi-flow-apply/SKILL.md`, update Step 1 so `supi_flow_apply` is described as loading the overview and structured task manifest up front while leaving linked `detail_path` docs unread at that stage.
2. In the same skill, update Step 2 so “Read the whole plan before starting” clearly means the approved overview plus the full task manifest, not every linked task-detail doc.
3. In Step 3 of the skill, keep the existing requirement to read a task’s `detail_path`, but make it explicit that this happens only when that specific task becomes active.
4. In `plugins/supi-flow/README.md`, update the apply workflow bullets, flow-phase summary, and any nearby tool wording that currently implies or could be read as eager task-detail loading so they match the skill’s lazy-loading rule.

## Test status
Test-exempt. This is a docs-and-guidance-only change with no practical automated harness beyond reviewing the rendered wording.

## Verification
Run:

```bash
git diff -- plugins/supi-flow/skills/supi-flow-apply/SKILL.md plugins/supi-flow/README.md
```

Expected result:
- the diff says apply start loads the approved overview and task manifest up front
- the diff does **not** instruct the agent to read every task detail doc at the beginning
- the diff says a linked `detail_path` doc is read only when the current task starts
