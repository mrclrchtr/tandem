---
name: supi-flow-archive
description: Verify implementation against the plan, update living documentation, and close out the change.
---

# Archive and document

Use this after `/supi-flow-apply` when implementation is complete. This is a docs-first closeout step, not a repository-cleanup workflow.

## The Iron Law

```text
NO COMPLETION CLAIMS WITHOUT FRESH VERIFICATION EVIDENCE
```

Before claiming the change is done, the docs are accurate, or the ticket can be closed: run the proof fresh, read the result, and check the exit status.

## Step 1: Find the change

- A TNDM-ID was set during plan phase. Read the ticket metadata first:
  `supi_tndm_cli { action: "show", id: "<ID>" }` — inspect `content_path` and the registered documents, then read `content.md` for the approved design and `plan.md` for the executed checklist.
- Archive runs only when a ticket exists. Trivial flows that skipped the ticket close out directly in conversation — do not run archive.
- If nothing is clear: ask which change to archive.

## Step 2: Verify completion

Compare the plan against what was actually done. Fresh checks only.

- [ ] Every planned task is complete, or any deviation is explained.
- [ ] Tests and verification commands were run fresh.
- [ ] The implemented result still matches the approved intent.
- [ ] Any claimed manual verification was actually performed.

If any check fails, stop and fix that first.

### Verification gate

```text
1. Identify the command or evidence that proves the claim.
2. Run it fresh.
3. Read the full result and exit code.
4. Confirm the claim matches the evidence.
5. Only then report success.
```

## Step 3: Update living documentation

Update docs only where the change actually affects them.

1. Review `git diff` to understand the real delta.
2. Identify the docs that should change.
3. Update them with grounded, specific language.
4. Reference actual file paths, commands, settings, or behavior when helpful.

## Step 4: Verify doc accuracy

Do the docs match the actual code and workflow?

- check file paths
- check command names
- check settings or behavior descriptions
- check that new guidance matches the final implementation

Do not assume documentation is correct just because it sounds right.

## Step 5: Close out

- Call `supi_flow_close { ticket_id: "<ID>", verification_results: "..." }` with the full verification evidence.
  This will set status=done, tags=flow:done, and store verification results in archive.md.
- There is no ticket-less closeout.

## Step 6: Commit or finish

```instructions
run("git status")
if only_changed(".tndm/"):
  commit(".tndm/", "chore(tndm): close <ticket_id>")
  say("The ticket is closed. All changes are committed.")
else:
  ask_user("Commit all changes now, including .tndm/, or finish and commit manually?")
  if user_chose_commit_now:
    if skill_exists_matching("commit"):
      use_skill_matching("commit")
    else:
      git_add_all()
      git_commit()
  else:
    say("The ticket is closed. Remember to commit your changes when ready.")
```

## Red flags

Stop if you catch yourself:

- claiming success from an old test run
- saying "should" or "probably" instead of citing evidence
- updating docs before confirming the implementation
- treating this as a repository cleanup workflow instead of a verification-and-docs closeout
