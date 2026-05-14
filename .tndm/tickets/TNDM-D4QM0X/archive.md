# Archive

Implemented bare ticket ID normalization in the CLI using the repo-configured `id.prefix`, while leaving already-formed IDs unchanged.

Files changed:
- `crates/tandem-cli/src/cli/util.rs`
- `crates/tandem-cli/src/cli/ticket.rs`
- `crates/tandem-cli/src/cli/doc.rs`
- `crates/tandem-cli/tests/ticket_cli_tests.rs`

Fresh verification evidence:
- `cargo test -p tandem-cli bare_ticket -- --nocapture` → passed (`6 passed, 45 filtered out`)
- `cargo test -p tandem-cli ticket_ -- --nocapture` → passed after formatting (`43 passed, 8 filtered out`)
- `cargo fmt --all` → passed with no output

Behavior verified:
- `ticket show <bare-id>` resolves via configured prefix
- `ticket update <bare-id>` resolves via configured prefix
- `ticket sync <bare-id>` resolves via configured prefix
- `ticket doc create <bare-id> <name>` resolves via configured prefix
- `ticket create/update --depends-on <bare-id,...>` stores prefixed dependency IDs when the repo prefix is configured
- explicit already-formed IDs continue to work via existing coverage

Documentation review:
- No living docs update was needed; this is a CLI behavior fix covered by regression tests.
