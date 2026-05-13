# Archive

## Verification Results

### Changes
- **crates/tandem-cli/src/cli/util.rs**: Added `read_stdin_if_no_flags(no_explicit: bool) -> anyhow::Result<Option<String>>`
- **crates/tandem-cli/src/cli/ticket.rs**: Replaced both duplicated stdin-reading blocks (in `handle_ticket_create` and `handle_ticket_update`) with calls to the helper; removed unused `io::{self, IsTerminal, Read}` import

### Fresh verification
| Command | Result |
|---------|--------|
| `cargo build --workspace` | ✅ Clean build |
| `cargo test --workspace` | ✅ 123 passed, 0 failed, 1 ignored |
| `cargo clippy --workspace` | ✅ No issues |

### Semantics preserved
- Same TTY-detection (`is_terminal()`)
- Same `read_to_string` with identical error message
- Same empty-buffer → `None` handling
- `bool` parameter cleanly captures the only difference between call sites (different field sets for "no explicit flags")

### Design alignment
Extracted helper matches the suggested signature from content.md:
```rust
fn read_stdin_if_no_flags(no_explicit: bool) -> anyhow::Result<Option<String>>;
```
