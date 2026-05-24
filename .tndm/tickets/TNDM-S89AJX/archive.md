# Archive

## Verification summary

All planned tasks are complete and the final implementation matches the approved intent: apply still loads the approved overview and full task manifest up front, while linked task detail docs are now explicitly deferred until the active task begins.

## Task completion check

- `supi_tndm_cli { action: "task_list", id: "TNDM-S89AJX" }`
- Result: task 1 and task 2 are both `done`.

## Fresh verification evidence

### 1. Documentation and guidance diff review

Command:

```bash
git diff -- plugins/supi-flow/skills/supi-flow-apply/SKILL.md plugins/supi-flow/README.md plugins/supi-flow/extensions/index.ts plugins/supi-flow/__tests__/resources.test.ts
```

Result:
- `plugins/supi-flow/skills/supi-flow-apply/SKILL.md` now says to read the overview and full task manifest up front, not every linked task doc.
- `plugins/supi-flow/skills/supi-flow-apply/SKILL.md` now says to read `detail_path` only when the current task becomes active.
- `plugins/supi-flow/README.md` now documents the same apply behavior in the workflow bullets, phase summary, and tool table.
- `plugins/supi-flow/extensions/index.ts` now registers `supi_flow_apply` prompt guidance that reviews the overview and full task list up front and reads linked task detail docs only when the active task begins.
- `plugins/supi-flow/__tests__/resources.test.ts` now inspects the registered tool object and asserts that `supi_flow_apply` carries the lazy task-detail-loading guidance.

### 2. Targeted registration tests

Command:

```bash
cd plugins/supi-flow
RTK_DISABLED=1 pnpm exec vitest run __tests__/index.test.ts __tests__/resources.test.ts -v
```

Result:
- Passed fresh.
- 2 test files passed.
- 9 tests passed.

### 3. Type-check

Command:

```bash
cd plugins/supi-flow
pnpm exec tsc --noEmit
```

Result:
- Passed fresh.
- `TypeScript: No errors found`

## Doc-accuracy review

Fresh reads of the changed guidance confirm the final wording is internally consistent across:
- `plugins/supi-flow/skills/supi-flow-apply/SKILL.md`
- `plugins/supi-flow/README.md`
- `plugins/supi-flow/extensions/index.ts`

A targeted search for the prior eager-loading wording returned no remaining matches in `plugins/supi-flow/`.
