# Plan: Deduplicate stdin-reading logic in tandem-cli

## Files modified

| File | Change |
|------|--------|
| `crates/tandem-cli/src/cli/util.rs` | Add `pub(crate) fn read_stdin_if_no_flags(no_explicit: bool) -> anyhow::Result<Option<String>>` helper |
| `crates/tandem-cli/src/cli/ticket.rs` | Replace both duplicated stdin blocks with calls to the helper; remove unused `io::{self, IsTerminal, Read}` import |

## Tasks

- [x] **Task 1**: Extract `read_stdin_if_no_flags` helper into `util.rs` and use it in both `handle_ticket_create` and `handle_ticket_update`
  - File: `crates/tandem-cli/src/cli/util.rs` — add the helper function:
    ```rust
    pub(crate) fn read_stdin_if_no_flags(no_explicit: bool) -> anyhow::Result<Option<String>> {
        if no_explicit && !std::io::stdin().is_terminal() {
            let mut buf = String::new();
            std::io::stdin()
                .read_to_string(&mut buf)
                .map_err(|error| anyhow::anyhow!("{error}"))?;
            if buf.is_empty() { None } else { Some(buf) }
        } else {
            None
        }
    }
    ```
    Add `use std::io::{self, IsTerminal, Read};` to `util.rs` imports.
  - File: `crates/tandem-cli/src/cli/ticket.rs` — in `handle_ticket_create`, replace:
    ```rust
    let stdin_content = if no_explicit_create && !io::stdin().is_terminal() {
        let mut buf = String::new();
        io::stdin()
            .read_to_string(&mut buf)
            .map_err(|error| anyhow::anyhow!("{error}"))?;
        if buf.is_empty() { None } else { Some(buf) }
    } else {
        None
    };
    ```
    with:
    ```rust
    let stdin_content = read_stdin_if_no_flags(no_explicit_create)?;
    ```
    — same replacement in `handle_ticket_update` using `no_explicit_update`.
  - Clean up: remove `io::{self, IsTerminal, Read}` from `ticket.rs` imports (only used in the now-extracted blocks).
  - Verification: `cargo build --workspace && cargo test --workspace`
  - Test exemption: The helper is a direct extraction with a `bool` parameter — it is identical logic moved to a shared location. Compilation and full test suite coverage confirms no regression.
