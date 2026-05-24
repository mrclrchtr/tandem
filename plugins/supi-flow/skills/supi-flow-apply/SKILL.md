---
name: supi-flow-apply
description: Execute an approved implementation plan task by task — verify each step fresh, stop when blocked, and ask instead of guessing.
---

# Execute implementation plan

## The Iron Law

```
EVERY TASK GETS FRESH VERIFICATION BEFORE MARKING DONE
```

If you haven't run the verification command for this task, you cannot check it off. Previous runs do not count. "Should pass" is not evidence.

## Step 1: Load the plan

Call `supi_flow_apply { ticket_id: "<ID>" }`. Read the returned overview and full task manifest.
If a task has a linked `detail_path`, note it — read that doc only when the task becomes active.

If the tool reports an error (missing overview, empty manifest, invalid lifecycle, blocked),
stop and resolve it before editing. If no plan is available, ask which change to implement.

## Step 2: Review the plan

Read the whole plan before starting. Raise concerns before editing if anything is unclear,
incomplete, or outdated. Do not start until those concerns are resolved.

## Step 3: Execute tasks

For each unchecked task, in order:

1. Announce which task you are working on.
2. **Read the task detail doc first.** Every task has a `detail_path` (e.g. `tasks/task-01.md`). Read that file before touching any code. The detail doc is the authoritative task specification — do not rely on memory or the plan overview alone.
3. Follow the task as written. Treat the detail doc as part of the task definition.
4. Run the verification for that task and read the result carefully.
5. If verification passes: call `supi_flow_complete_task { ticket_id: "<ID>", task_number: <N> }` to check the task off in the ticket.
6. If verification fails: stop, diagnose, fix, and re-verify before moving on.
7. Record what actually passed.

Do not skip failed checks. Do not collapse several tasks into one vague batch.

### TDD by default, not always

For testable code, write the test first:

1. Write a failing test for the right reason.
2. Write the minimal code to make it pass.
3. Re-run and confirm it passes.
4. Run any broader regression checks the plan calls for.

No production code without a failing test first.

### Test-exempt work

For docs-only, config-only, trivial changes, or integration work with no reasonable harness:

1. Run the manual verification from the plan. If the plan omitted one, add the smallest
   concrete verification you can justify.
2. Confirm actual output matches the claim and note the exemption when reporting completion.

If uncertain whether TDD is practical, ask instead of guessing.

The hard rule is not "TDD in every case." The hard rule is: **no unverified changes**.

## Step 4: When blocked or stuck

If verification fails and you don't understand why, load `/skill:supi-flow-debug`
before attempting random fixes.

Stop and ask the user when:

- A verification fails repeatedly and you still don't understand the cause
- You've tried 3 fixes and none worked
- A dependency is missing
- The plan has a gap, is unclear, or scope changed
- A task is missing verification
- You're unsure whether to use TDD or a test exemption

Do not guess. Do not force through blockers.

## Rationalization prevention

| Excuse | Reality |
|--------|---------|
| "Should pass now" | Run the verification again. Fresh. |
| "I'm confident" | Confidence is not evidence. |
| "Just this once" | Verification still applies. |
| "Previous run passed" | Code changed. Run it again. |
| "This task is too simple to need TDD" | TDD is preferred for testable code. If it is not practical, verify it manually and say why. |
| "I know what the task says" | Task detail docs evolve between plan and apply. Read them fresh. |

## When all tasks are done

- If a ticket exists: summarize what passed in conversation, but do not mark it done yet — `/skill:supi-flow-archive` handles durable verification evidence in `archive.md` and final closeout.
- Announce: `Implementation complete. Run /skill:supi-flow-archive TNDM-XXXXXX to verify, update docs, and close out.`
