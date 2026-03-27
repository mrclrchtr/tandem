# Shell Completion Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add dynamic shell completion to the `tndm` CLI, including custom ticket ID completion at tab-press time.

**Architecture:** Wire `clap_complete`'s `CompleteEnv` into `main()` before argument parsing. Annotate positional ticket ID args with a custom `ArgValueCompleter` that queries `FileTicketStore`. Add `ValueHint::FilePath` to `--content-file` args.

**Tech Stack:** Rust, clap 4.x derive API, clap_complete (unstable-dynamic feature), clap (unstable-ext feature for `#[arg(add = ...)]`)

**Spec:** `docs/superpowers/specs/2026-03-27-shell-completion-design.md`

**Note on clippy:** This project disallows `std::process::Command` (type) and `std::fs::create_dir_all` (method) in `clippy.toml`. Test files must use file-level `#![allow(clippy::disallowed_types)]` and per-test `#[allow(clippy::disallowed_methods)]` as needed. See existing test files for the pattern.

---

## File Structure

| File | Responsibility |
|------|---------------|
| `Cargo.toml` (root) | Add `clap_complete` workspace dependency, add `unstable-ext` feature to `clap` |
| `crates/tandem-cli/Cargo.toml` | Wire `clap_complete` into CLI crate |
| `crates/tandem-cli/src/main.rs` | `CompleteEnv` call, ticket ID completer fn, `#[arg]` annotations |
| `crates/tandem-cli/tests/completion_cli_tests.rs` | Integration tests for completion setup and ticket ID completion |

---

## Chunk 1: Dependencies and CompleteEnv Wiring

### Task 1: Add `clap_complete` dependency

**Files:**
- Modify: `Cargo.toml:20` (workspace dependencies)
- Modify: `crates/tandem-cli/Cargo.toml:13-21` (dependencies)

- [ ] **Step 1: Add workspace dependency and enable `unstable-ext` on clap**

In root `Cargo.toml`, change the `clap` line (line 20) from:

```toml
clap = { version = "4.5.23", features = ["derive"] }
```

to:

```toml
clap = { version = "4.5.23", features = ["derive", "unstable-ext"] }
```

Then add after that line:

```toml
clap_complete = { version = "4.5", features = ["unstable-dynamic"] }
```

The `unstable-ext` feature is required for `#[arg(add = ...)]` syntax used to attach `ArgValueCompleter` to arguments.

- [ ] **Step 2: Wire into CLI crate**

In `crates/tandem-cli/Cargo.toml`, add after the `clap.workspace = true` line (line 14):

```toml
clap_complete.workspace = true
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo build -p tandem-cli`
Expected: successful build with no errors

- [ ] **Step 4: Verify architecture checks pass**

Run: `cargo xtask check-arch`
Expected: "architecture checks passed" — `clap_complete` is a different dependency name than `clap`, so the clap-only-in-CLI check won't trigger.

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml Cargo.lock crates/tandem-cli/Cargo.toml
git commit -m "feat(cli): add clap_complete dependency for shell completion"
```

---

### Task 2: Wire `CompleteEnv` into `main()`

**Files:**
- Modify: `crates/tandem-cli/src/main.rs:1-10` (imports), `crates/tandem-cli/src/main.rs:181-182` (main fn)

- [ ] **Step 1: Write the integration test for completion setup**

Create `crates/tandem-cli/tests/completion_cli_tests.rs`:

```rust
#![allow(clippy::disallowed_types)]

use std::process::Command;

#[test]
fn complete_env_produces_bash_setup_script() {
    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .env("COMPLETE", "bash")
        .output()
        .expect("run tndm with COMPLETE=bash");

    assert!(
        output.status.success(),
        "expected success, stderr was: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert!(
        stdout.contains("tndm"),
        "bash setup script should reference the binary name, got: {stdout:?}"
    );
    assert!(
        stdout.contains("complete"),
        "bash setup script should contain a 'complete' builtin call, got: {stdout:?}"
    );
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p tandem-cli complete_env_produces_bash_setup_script`
Expected: FAIL — currently `COMPLETE=bash` is not handled, so `tndm` will try to parse args and fail or produce unexpected output.

- [ ] **Step 3: Add imports and `CompleteEnv` call**

In `crates/tandem-cli/src/main.rs`, add to the imports at the top (after line 9):

```rust
use clap::CommandFactory;
use clap_complete::env::CompleteEnv;
```

Then change the beginning of `fn main()` (line 182) from:

```rust
    let cli = Cli::parse();
```

to:

```rust
    CompleteEnv::with_factory(Cli::command)
        .complete();

    let cli = Cli::parse();
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p tandem-cli complete_env_produces_bash_setup_script`
Expected: PASS

- [ ] **Step 5: Run full test suite to verify no regressions**

Run: `cargo test -p tandem-cli`
Expected: all existing tests pass

- [ ] **Step 6: Commit**

```bash
git add crates/tandem-cli/src/main.rs crates/tandem-cli/tests/completion_cli_tests.rs
git commit -m "feat(cli): wire CompleteEnv for dynamic shell completion"
```

---

## Chunk 2: Custom Ticket ID Completer

### Task 3: Write unit test for ticket ID completer

**Files:**
- Modify: `crates/tandem-cli/src/main.rs` (add test module)

- [ ] **Step 1: Write the failing unit test**

At the bottom of `crates/tandem-cli/src/main.rs`, add a test module:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::disallowed_methods)]
    fn complete_ticket_ids_filters_by_prefix() {
        let repo_root = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

        let store = FileTicketStore::new(repo_root.path().to_path_buf());
        let meta_a = TicketMeta::new(TicketId::parse("TEST-AAA111").unwrap(), "First".to_string()).unwrap();
        let meta_b = TicketMeta::new(TicketId::parse("TEST-BBB222").unwrap(), "Second".to_string()).unwrap();
        let meta_c = TicketMeta::new(TicketId::parse("OTHER-CCC333").unwrap(), "Third".to_string()).unwrap();

        store.create_ticket(NewTicket { meta: meta_a, content: String::new() }).unwrap();
        store.create_ticket(NewTicket { meta: meta_b, content: String::new() }).unwrap();
        store.create_ticket(NewTicket { meta: meta_c, content: String::new() }).unwrap();

        // The completer uses env::current_dir(), so we test the inner logic directly
        let ids = store.list_ticket_ids().unwrap();
        let prefix = "TEST";
        let results: Vec<String> = ids
            .iter()
            .filter(|id| id.as_str().starts_with(prefix))
            .map(|id| id.to_string())
            .collect();

        assert_eq!(results.len(), 2);
        assert!(results.contains(&"TEST-AAA111".to_string()));
        assert!(results.contains(&"TEST-BBB222".to_string()));
        assert!(!results.contains(&"OTHER-CCC333".to_string()));
    }

    #[test]
    #[allow(clippy::disallowed_methods)]
    fn complete_ticket_ids_empty_prefix_returns_all() {
        let repo_root = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

        let store = FileTicketStore::new(repo_root.path().to_path_buf());
        let meta = TicketMeta::new(TicketId::parse("TEST-AAA111").unwrap(), "Only".to_string()).unwrap();
        store.create_ticket(NewTicket { meta, content: String::new() }).unwrap();

        let ids = store.list_ticket_ids().unwrap();
        let prefix = "";
        let results: Vec<String> = ids
            .iter()
            .filter(|id| id.as_str().starts_with(prefix))
            .map(|id| id.to_string())
            .collect();

        assert_eq!(results.len(), 1);
        assert!(results.contains(&"TEST-AAA111".to_string()));
    }
}
```

- [ ] **Step 2: Run test to verify it passes**

Run: `cargo test -p tandem-cli complete_ticket_ids`
Expected: PASS — the test exercises the filtering logic directly using the storage layer.

- [ ] **Step 3: Commit**

```bash
git add crates/tandem-cli/src/main.rs
git commit -m "test(cli): add unit tests for ticket ID completion filtering"
```

---

### Task 4: Implement ticket ID completer and wire into args

**Files:**
- Modify: `crates/tandem-cli/src/main.rs` (imports, new function, `#[arg]` annotations)

- [ ] **Step 1: Add imports**

In `crates/tandem-cli/src/main.rs`, add `std::ffi::OsStr` to the existing `std` import block (lines 3-7):

```rust
use std::{
    env, ffi::OsStr, fs,
    io::{self, IsTerminal, Read},
    path::PathBuf,
};
```

Add the completer import after the `clap_complete::env::CompleteEnv` import:

```rust
use clap_complete::engine::{ArgValueCompleter, CompletionCandidate};
```

- [ ] **Step 2: Add the completer function**

Add this function before `fn main()`:

```rust
fn complete_ticket_ids(current: &OsStr) -> Vec<CompletionCandidate> {
    let Ok(cwd) = env::current_dir() else {
        return vec![];
    };
    let Ok(root) = discover_repo_root(&cwd) else {
        return vec![];
    };
    let store = FileTicketStore::new(root);
    let Ok(ids) = store.list_ticket_ids() else {
        return vec![];
    };

    let prefix = current.to_string_lossy();
    ids.into_iter()
        .filter(|id| id.as_str().starts_with(prefix.as_ref()))
        .map(|id| CompletionCandidate::new(id.to_string()))
        .collect()
}
```

- [ ] **Step 3: Annotate ticket ID args with the completer**

Update the following `id` fields in the `TicketCommand` enum:

For `Create` — change the `id` field (around line 103-104):

```rust
        /// Optional explicit ticket ID.
        #[arg(long, add = ArgValueCompleter::new(complete_ticket_ids))]
        id: Option<String>,
```

For `Show` — add `#[arg]` to the bare `id` field (around line 118):

```rust
    Show {
        #[arg(add = ArgValueCompleter::new(complete_ticket_ids))]
        id: String,

        #[command(flatten)]
        output: OutputArgs,
    },
```

For `Update` — update the `id` field (around line 135):

```rust
        /// Ticket ID to update.
        #[arg(add = ArgValueCompleter::new(complete_ticket_ids))]
        id: String,
```

- [ ] **Step 4: Verify it compiles and tests pass**

Run: `cargo build -p tandem-cli && cargo test -p tandem-cli`
Expected: successful build, all tests pass

- [ ] **Step 5: Commit**

```bash
git add crates/tandem-cli/src/main.rs
git commit -m "feat(cli): add custom ticket ID completer for shell completion"
```

---

### Task 5: Add `ValueHint::FilePath` to `--content-file` args

**Files:**
- Modify: `crates/tandem-cli/src/main.rs` (two `content_file` args)

- [ ] **Step 1: Add `ValueHint` import**

In `crates/tandem-cli/src/main.rs`, update the `clap` import (which after Task 2's commit and rustfmt will be a merged line) from:

```rust
use clap::{Args, CommandFactory, Parser, Subcommand};
```

to:

```rust
use clap::{Args, CommandFactory, Parser, Subcommand, ValueHint};
```

- [ ] **Step 2: Annotate Create's `content_file`**

Change the `content_file` field in `TicketCommand::Create` (around line 107-108) from:

```rust
        #[arg(long, conflicts_with = "content")]
        content_file: Option<PathBuf>,
```

to:

```rust
        #[arg(long, conflicts_with = "content", value_hint = ValueHint::FilePath)]
        content_file: Option<PathBuf>,
```

- [ ] **Step 3: Annotate Update's `content_file`**

Change the `content_file` field in `TicketCommand::Update` (around line 162) from:

```rust
        #[arg(long, conflicts_with = "update_content")]
        content_file: Option<PathBuf>,
```

to:

```rust
        #[arg(long, conflicts_with = "update_content", value_hint = ValueHint::FilePath)]
        content_file: Option<PathBuf>,
```

- [ ] **Step 4: Verify it compiles and tests pass**

Run: `cargo build -p tandem-cli && cargo test -p tandem-cli`
Expected: successful build, all tests pass

- [ ] **Step 5: Commit**

```bash
git add crates/tandem-cli/src/main.rs
git commit -m "feat(cli): add FilePath value hints for --content-file args"
```

---

## Chunk 3: Verification

### Task 6: Final verification

- [ ] **Step 1: Run the full check suite**

Run: `mise run check`
Expected: all checks pass (fmt, compile, arch, clippy, test)

- [ ] **Step 2: Manual smoke test (informational)**

If you have the shell setup configured, test interactively:

```bash
# Build the binary
cargo build

# Set up completion for current shell session (bash example)
source <(COMPLETE=bash ./target/debug/tndm)

# Try completing:
# ./target/debug/tndm <TAB>        → should show: fmt, ticket, awareness
# ./target/debug/tndm ticket <TAB> → should show: create, show, list, update
# ./target/debug/tndm ticket show <TAB> → should show ticket IDs from .tndm/
```

- [ ] **Step 3: Commit any final fixups if needed**

Only if previous steps required adjustments.

---

## Summary of commits

1. `feat(cli): add clap_complete dependency for shell completion`
2. `feat(cli): wire CompleteEnv for dynamic shell completion`
3. `test(cli): add unit tests for ticket ID completion filtering`
4. `feat(cli): add custom ticket ID completer for shell completion`
5. `feat(cli): add FilePath value hints for --content-file args`
