# Shell Completion via `CompleteEnv`

## Summary

Add dynamic shell completion to the `tndm` CLI using `clap_complete`'s `CompleteEnv` (unstable-dynamic engine). This gives users tab-completion for subcommands, flags, enum values, file paths, and — critically — **ticket IDs** queried from the repo at tab-press time.

## Motivation

Shell completion is table-stakes UX for CLI tools. Static (AOT) completion scripts can only complete fixed tokens (subcommands, flags, enum variants). Dynamic completion lets the binary itself answer "what are the valid values here?" at runtime, enabling context-aware completions like ticket IDs from the local `.tndm/` store.

## Approach: `CompleteEnv` (dynamic)

`clap_complete::env::CompleteEnv` intercepts shell completion requests via an environment variable (`COMPLETE`). When the shell asks for completions, the binary builds its `clap::Command`, resolves completions (including custom value completers), prints them, and exits. Normal invocations are unaffected.

### Why not AOT?

- Cannot complete ticket IDs (the primary user request).
- Requires a separate `tndm completions <shell>` subcommand and regeneration when the CLI changes.
- `CompleteEnv` is where clap is converging (clap-rs/clap#3166).

### Trade-off: `unstable-dynamic` feature

The `CompleteEnv` API requires the `unstable-dynamic` cargo feature on `clap_complete`. This is acceptable because:

- `tndm` is a developer tool with a controlled toolchain.
- The API surface we use is small (`CompleteEnv::with_factory`, `ArgValueCompleter`).
- Clap is actively stabilizing this (tracking issue clap-rs/clap#3166).

## Design

### 1. Dependency changes

**Root `Cargo.toml`** — add workspace dependency:

```toml
clap_complete = { version = "4.5", features = ["unstable-dynamic"] }
```

**`crates/tandem-cli/Cargo.toml`** — wire it in:

```toml
clap_complete.workspace = true
```

No other crates are affected. Architecture constraints are preserved — `clap_complete` is a CLI concern, and the `check-arch` xtask checks for exact dependency name `"clap"`, so `"clap_complete"` will not trigger a false positive.

### 2. `main()` integration

`CompleteEnv::with_factory` must run **before** `Cli::parse()` so it can intercept completion requests and exit early without executing any CLI logic:

```rust
use clap::CommandFactory;
use clap_complete::env::CompleteEnv;

fn main() -> anyhow::Result<()> {
    CompleteEnv::with_factory(Cli::command)
        .complete();

    let cli = Cli::parse();
    // ... rest unchanged
}
```

**Important:** No stdout output may occur before `.complete()` returns, or it will corrupt completion output. The current `main()` has no such output, so this is safe.

### 3. Custom ticket ID completer

A completion function that discovers the repo root and lists ticket IDs, filtering by the current prefix:

```rust
use std::ffi::OsStr;
use clap_complete::engine::{ArgValueCompleter, CompletionCandidate};

fn complete_ticket_ids(current: &OsStr) -> Vec<CompletionCandidate> {
    let Ok(cwd) = env::current_dir() else { return vec![] };
    let Ok(root) = discover_repo_root(&cwd) else { return vec![] };
    let store = FileTicketStore::new(root);
    let Ok(ids) = store.list_ticket_ids() else { return vec![] };

    let prefix = current.to_string_lossy();
    ids.into_iter()
        .filter(|id| id.to_string().starts_with(prefix.as_ref()))
        .map(|id| CompletionCandidate::new(id.to_string()))
        .collect()
}
```

This completer is attached to the `id` positional arguments on:

- `ticket show <id>`
- `ticket update <id>`
- `ticket create --id <id>` (less useful but consistent)

Using the derive API, this is wired via `#[arg(add = ArgValueCompleter::new(complete_ticket_ids))]`. For fields that previously had no `#[arg(...)]` attribute (e.g. `Show::id`), one must be added.

### 4. `ValueHint` annotations

| Argument | Hint |
|----------|------|
| `--content-file` (create) | `ValueHint::FilePath` |
| `--content-file` (update) | `ValueHint::FilePath` |

Other arguments either have enum possible values (auto-completed) or free-form strings where no hint applies.

### 5. What comes for free

The clap derive macros + `CompleteEnv` automatically provide completions for:

- Subcommand names: `fmt`, `ticket`, `awareness`, `create`, `show`, `list`, `update`
- All flag names: `--status`, `--priority`, `--type`, `--json`, `--check`, `--all`, etc.
- Short flags: `-s`, `-p`, `-T`, `-g`, `-d`
- Enum possible values: `todo`, `in_progress`, `blocked`, `done`, `p0`–`p4`, `task`, `bug`, `feature`, `chore`, `epic`

### 6. User setup

One-time addition to shell config:

```bash
# bash (~/.bashrc)
source <(COMPLETE=bash tndm)

# zsh (~/.zshrc)
source <(COMPLETE=zsh tndm)

# fish (~/.config/fish/config.fish)
source (COMPLETE=fish tndm | psub)
```

## Testing

### Unit test

Create a temp repo with tickets, call `complete_ticket_ids` with various prefixes, assert correct filtering.

### Integration test

Run `COMPLETE=bash tndm` via `assert_cmd` and verify the output contains the binary name `tndm` and a shell function pattern (e.g. `complete` for bash). This ensures `CompleteEnv` is properly wired and produces valid shell setup output.

## Scope exclusions

- No AOT/static script generation fallback.
- No `tndm completions <shell>` subcommand.
- No git ref completion for `--against` (could be added in a follow-up).
- No custom completers for `--tags` (free-form, no canonical source).
- No custom completer for `--depends-on` — it accepts a comma-separated string, so completing individual IDs within the value would require splitting on commas and reconstructing. Can be added as a follow-up.

## Files changed

| File | Change |
|------|--------|
| `Cargo.toml` (root) | Add `clap_complete` workspace dependency |
| `crates/tandem-cli/Cargo.toml` | Add `clap_complete` dependency |
| `crates/tandem-cli/src/main.rs` | Add `CompleteEnv` call, ticket ID completer, `ValueHint`/`ArgValueCompleter` annotations |
| `crates/tandem-cli/tests/` | Add completion integration test |
