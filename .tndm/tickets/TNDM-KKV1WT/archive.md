# Archive

## Validation Analysis (per content.md first-task-before-planning)

### Finding: Premature abstraction — do not proceed

The three error types (`ValidationError`, `StorageError`, `RepoError`) serve **distinct semantic domains** despite being structurally identical:

| Type | Domain | Unique |
|---|---|---|
| `ValidationError` | Invalid input data | `pub` constructor, core domain |
| `StorageError` | I/O failures | `TicketStore::Error` associated type |
| `RepoError` | Git operation failures | `git_command_failed(args, stderr)` helper |

### Reasons against unification

1. **Semantic loss**: Unifying into a single `TandemError` erases the ability to distinguish validation failures from I/O failures from git failures at the type level. The design suggestion to "wrap in an enum later" adds back complexity.

2. **`RepoError::git_command_failed`** is a genuine behavioral difference — it formats git command+stderr into error messages. Sharing this via `TandemError` would pollute core with git-specific logic, or require keeping `RepoError` as a wrapper anyway.

3. **Minimal savings**: ~45 lines total eliminated across the workspace. Not worth the cross-crate refactoring risk.

4. **No maintenance burden**: With only 3 crate boundaries and stable error patterns, the duplication is not causing friction. No code matches on error type variants — all errors flow through `Display`/`Error` into `anyhow`.

### Decision

**Close without changes.** The design goal is sound in spirit (reduce boilerplate duplication) but the full unification is the wrong tradeoff for this codebase at this stage.
