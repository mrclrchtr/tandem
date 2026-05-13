## Context
The `documents` diff in `AwarenessFieldDiffs::between` builds two `Vec<String>` of all document names, then compares fingerprints. `AwarenessVecDiff` for documents contains *all* document names, not just changed ones — unlike `depends_on` and `tags` diffs which list full sorted lists (correct for set-like fields). For documents, users likely care which specific fingerprints changed.

## Suggestion
Consider whether `documents` should report only changed document names, or add a dedicated `AwarenessDocDiff` that lists only changed entries. If current behavior is intentional, document it clearly.

## First Task Before Planning
- [ ] **Validate this refactoring for correctness, best practices, and actual value to the project.** Review how the awareness JSON output is consumed (e.g., by pi agents). Determine whether changing the `documents` diff shape is a breaking change. Verify that the current behavior isn't actually preferred for downstream consumers.
