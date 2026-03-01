---
name: git-commit
description: "Creates a commit: detects conventions, stages intentionally, writes a clear subject, add a concise body when useful, and commits."
---

# Git Commit

## Goal

Make a logical, reviewable concise commit using the commit style of the repository.

## Guardrails

- If potential secrets are found: **STOP and ask** what to do.
- No `--no-verify`, no `--amend`/rebase/force-push, no pushing unless asked.
- If changes look like multiple commits: **STOP and propose a split plan** (don’t commit yet).

## Fast workflow

1) Gather information

    ```bash
    echo "## DATE" \
    && date \
    && echo "## BRANCH" \
    && git branch --show-current \
    && echo "## STATUS" \
    && git status --porcelain=v1 \
    && echo "## DIFF (unstaged)" \
    && git --no-pager diff \
    && echo "## DIFF (staged)" \
    && git --no-pager diff --staged \
    && echo "## LOG (last 20)" \
    && git --no-pager log --oneline -20 --graph
    ```

   > **NOTE**:
   > Run as one command.

2) Stage changes intentionally

   ```bash
   git add path/to/file1 path/to/file2
   # or:
   git add -A # when all changes belong to the commit to create
   ```

   If the staged diff contains unrelated changes, **STOP and ask** what to do.

3) Write a concise commit message

   Infer commit style from recent subjects:
    - If they look like `type(scope): msg` → use Conventional Commits.
    - Otherwise, match the common pattern (caps, prefixes, ticket IDs, etc.).

   Subject rules:
    - Imperative mood, no trailing period
    - Prefer ≤ 72 chars (or match repo norm)
    - Include scope only if the repo typically does

   Body rules:
    - Add a body **only** if it answers “why” or prevents confusion:
        - Why this change is needed
        - Key tradeoffs or constraints
        - Notable side effects/follow-ups

4) Commit and verify
   Use multiple `-m` flags for multi-line messages (no \n).

   ```bash
   git commit -m "type(scope): concise summary"
   # or with body:
   git commit -m "type(scope): concise summary" -m "Why this change was needed (brief)."
   ```
