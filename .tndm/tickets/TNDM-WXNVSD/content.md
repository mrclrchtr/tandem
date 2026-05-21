## Overview

Clarify the new single-task authoring workflow so it is correct for both fresh plans and replans. The docs should explain that repeated `supi_flow_task { operation: "add" }` calls are only the empty-ticket path. When a ticket already has tasks, the agent must first inspect the current manifest, then reconcile it by editing matching tasks, removing stale tasks, and adding only genuinely new tasks. The numbering guidance must also stop implying that every plan starts at task 1.

## Files

- `plugins/supi-flow/skills/supi-flow-plan/SKILL.md` — add the explicit reconcile/replan flow and fix numbering guidance.
- `plugins/supi-flow/README.md` — mirror the same common-path guidance in user-facing docs.
- `plugins/supi-flow/CLAUDE.md` — keep the maintainer guidance aligned with the skill/README wording.

## Verification

Use targeted grep/readback verification to confirm the docs now mention:
- list current tasks before replanning
- edit/remove/add as the reconcile path
- task numbers start at 1 only on empty tickets, while adds on existing tickets return the next available number
