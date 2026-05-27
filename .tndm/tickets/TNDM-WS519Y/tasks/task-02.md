# Task 2: Wire ticket_ctx module into cli/mod.rs

Add `mod ticket_ctx;` to `crates/tandem-cli/src/cli/mod.rs` after the existing module declarations (`mod awareness; mod doc; ...`).

**Verification**: `cargo build -p tandem-cli` compiles without errors.
