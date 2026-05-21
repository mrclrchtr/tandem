# Task 1: Add `supi_flow_apply` and register it as the apply-phase entrypoint

## Goal
Introduce a dedicated `supi_flow_apply { ticket_id }` tool that becomes the supported apply-start entrypoint.

## Required behavior
- Add `supiFlowApplyParams` and `executeFlowApply` in `plugins/supi-flow/extensions/tools/flow-tools.ts`.
- Load the ticket via `ticket show`, read the approved overview from `content.md`, and load the structured task manifest via `ticket task list`.
- If the ticket is currently `flow:planned`, transition it to `status=in_progress` with `flow:applying`.
- If the ticket is already `flow:applying`, treat the call as an idempotent re-entry and return the same context without changing state again.
- Reject invalid apply starts: blank/missing overview, empty task manifest, or tickets already closed / outside the planned-applying lifecycle.
- Return enough context for the skill to continue without another discovery round: transition outcome, overview markdown, content path, and numbered task manifest including `detail_path` when present.

## Registration expectations
- Register the new tool in `plugins/supi-flow/extensions/index.ts` with prompt metadata that explicitly names `supi_flow_apply`.
- Mark mutating flow tools as `executionMode: "sequential"` so PI does not race ticket mutations in one assistant response.
- Update registration tests to expect the additional tool and the new total count.
