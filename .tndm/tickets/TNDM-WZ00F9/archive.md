# Archive

## Verification Results

### Fresh verification commands (all pass)

| Command | Result |
|---|---|
| `cargo build -p tandem-core -p tandem-storage -p tandem-repo` | clean |
| `cargo test -p tandem-core --locked` | 45 passed, 1 ignored |
| `cargo test -p tandem-core --locked -- canonical` | 7 passed (canonical format tests) |
| `cargo test -p tandem-storage -p tandem-repo --locked` | 33 passed |
| `cargo clippy -p tandem-core -p tandem-storage -p tandem-repo` | no issues |
| `cargo xtask check-arch` | passes |

### Changes in `crates/tandem-core/src/ticket.rs`

**Net: +172 lines, -359 lines (-187 lines reduction)**

1. **Added `string_enum!` macro** (~30 lines): generates `parse()`, `as_str()`, `FromStr`, `Display`, and `Serialize` from variant→string mappings. No external dependencies.

2. **Replaced 4× enum boilerplate** with macro invocations:
   - `TicketType`, `TicketPriority`, `TicketStatus`, `TicketEffort` (each ~35 lines → each ~9 lines)
   - Multi-word variant handled naturally: `InProgress => "in_progress"`

3. **Consolidated tests**: 13 redundant enum-behavior tests → 1 `macro_generated_impls` test; kept 4 per-enum roundtrip tests.

### Why no strum

Custom declarative macro avoids adding two new transitive dependencies (`strum` + `strum_macros`) while maintaining full control over error messages and zero risk of version churn.

### No documentation changes needed

Internal refactoring with no behavioral change. No docs reference these enum internals.
