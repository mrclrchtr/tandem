---
name: supi-flow-archive
description: Verify implementation against the plan, update living documentation, and close out the change.
disable-model-invocation: true
---

# Archive and document

Use after `/skill:supi-flow-apply` when implementation is complete. This is a docs-first verification closeout, not a repository cleanup.

## The Iron Law

NO COMPLETION CLAIMS WITHOUT FRESH VERIFICATION EVIDENCE. Run the proof fresh, read the result, check the exit status.

## Step 1: Find the change

Read the ticket metadata and content, then list its tasks. Archive only runs when a ticket exists — trivial flows close out in conversation. If unclear, ask which change to archive.

## Step 2: Verify completion against the plan

For every planned task, run fresh verification:

1. Identify the command or evidence that proves completion.
2. Run it fresh.
3. Read the full result and exit code.
4. Confirm the evidence matches the claim.

Stop if: a task is incomplete and unexplained, tests weren't run fresh, the result diverges from approved intent, or claimed manual verification wasn't actually performed.

## Step 3: Update and verify docs

1. Review `git diff` for the real delta.
2. Update only the docs the change actually affects — file paths, commands, settings, behavior.
3. Verify docs match the final code: check paths, command names, settings, and guidance against the implementation. Do not assume correctness.

## Step 4: Close out

Close the ticket — requires nonblank verification evidence and all tasks complete. Stores the evidence in the ticket archive.

## Step 5: Commit

If only ticket files changed: commit with `chore(tndm): close <ticket_id>`. Otherwise ask whether to commit now (use the `commit` skill if available) or let the user handle it manually.

## Red flag

Stop if you're treating this as repository cleanup instead of a verification closeout.
