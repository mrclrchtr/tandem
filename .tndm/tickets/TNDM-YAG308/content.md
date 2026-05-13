## Context
A `ticket()` test helper exists in `tandem-core/src/awareness.rs` tests. Similar fixture builders may emerge in `tandem-repo` and `tandem-storage` tests.

## Validation Survey
Surveyed all 10 test/lint files across 4 crates. Findings:

- The `ticket()` helper in `awareness.rs` exists in **exactly 1 location**, used in 17+ tests within the same module
- **No cross-crate duplication** — `tandem-storage/tests/*` use public API directly
  (`TicketMeta::new()`, `NewTicket`)
- `TestRepo` in `tandem-repo/tests/awareness_repo_tests.rs` is a git integration
  harness, not a ticket-builder duplicate
- `write_ticket()` in `tandem-cli/tests/awareness_cli_tests.rs` writes raw TOML
  at a different abstraction level (CLI integration tests)
- The remaining CLI tests (`ticket_cli_tests.rs`, `fmt_cli_tests.rs`) use
  `Command::new(env!("CARGO_BIN_EXE_tndm"))` and have no programmatic builder

## Alternatives Evaluated

| Option | Cost | Benefit | When to use |
|---|---|---|---|
| **A: Do nothing** | Zero | Right-sized for today | Now (current state) |
| **B: Feature-gated export** from `tandem-core`
  (`#[cfg(feature = "test-support")]` pub module) | Negligible compile cost;
  standard Rust pattern | Cross-crate sharing
  without new workspace crate | When a second crate
  duplicates the builder |
| **C: New `tandem-test-support`** dev-dep crate | New workspace crate,
  Cargo.toml, CI cost | Full separation | Only if B is
  insufficient |

## Recommendation
**Option A (do nothing) for now.** The ticket's own framing said "If helper
duplication grows… lower priority" — the threshold has not been met.
Duplication is exactly 1 location, no cross-crate reach.

If/when a second crate needs the builder, **Option B (feature-gated export)** is
the right response: thin, standard Rust idiom (`proptest` et al.), no new
workspace crate.
