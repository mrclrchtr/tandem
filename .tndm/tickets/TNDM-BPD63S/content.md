# Implementation Plan: Document registry + agent file-edit protocol

## Approved design

- TNDM owns document creation/registration.
- Agents edit registered markdown files directly.
- TNDM syncs SHA-256 fingerprints after edits.
- `tndm fmt --check` fails on stale/missing/unregistered ticket documents.
- Plugins stop encouraging large markdown content strings.

## Tasks

### Task 1: Core document model
- **Files**: `crates/tandem-core/src/ticket.rs`
- **TDD**: Add `TicketDocument`, `documents` field on `TicketMeta`, `document_fingerprints` on `TicketState`
- **Verify**: `cargo test -p tandem-core`

### Task 2: Storage fingerprinting and document validation
- **Files**: `Cargo.toml`, `crates/tandem-storage/`, its tests
- **TDD**: SHA-256 fingerprint helper, document registry I/O, sync/validate
- **Verify**: `cargo test -p tandem-storage`

### Task 3: CLI doc create/sync/fmt-check
- **Files**: `crates/tandem-cli/src/main.rs`, its tests
- **TDD**: `ticket doc create`, `ticket sync`, fmt stale-fingerprint detection
- **Verify**: `cargo test -p tandem-cli`

### Task 4: Awareness document diff
- **Files**: `crates/tandem-core/src/awareness.rs`
- **TDD**: fingerprint-only diff, no content embedding
- **Verify**: `cargo test -p tandem-core`

### Task 5: TNDM plugin guidance update
- **Files**: docs + plugin skills
- **Test-exempt**: docs only
- **Verify**: `rg heredoc plugins/tndm/` (should have 0)

### Task 6: Supi-flow tools/skills update
- **Files**: supi-flow tools, skills, tests
- **TDD**: update flow tools to use doc create/sync
- **Verify**: `pnpm exec vitest run` in plugins/supi-flow

### Task 7: Migrate existing ticket files
- **Test-exempt**: migration of local data
- **Verify**: `./tndm-dev fmt --check`
