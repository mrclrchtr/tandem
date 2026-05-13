# Implementation Plan: Document registry + agent file-edit protocol

## Approved design

- TNDM owns document creation/registration.
- Agents edit registered markdown files directly.
- TNDM syncs SHA-256 fingerprints after edits.
- `tndm fmt --check` fails on stale/missing/unregistered ticket documents.
- Plugins stop encouraging large markdown content strings.

## Tasks

## Status: ✅ DONE — All tasks implemented and verified

### ✅ Task 1: Core document model
- **Files**: `crates/tandem-core/src/ticket.rs`
- **Verify**: `cargo test -p tandem-core` → **56 passed**

### ✅ Task 2: Storage fingerprinting and document validation
- **Files**: `Cargo.toml`, `crates/tandem-storage/`, its tests
- **Verify**: `cargo test -p tandem-storage` → **29 passed**

### ✅ Task 3: CLI doc create/sync/fmt-check
- **Files**: `crates/tandem-cli/src/main.rs`, its tests
- **Verify**: `cargo test -p tandem-cli` → **45 passed**

### ✅ Task 4: Awareness document diff
- **Files**: `crates/tandem-core/src/awareness.rs`
- **Verify**: `cargo test -p tandem-core` (part of suite)

### ✅ Task 5: TNDM plugin guidance update
- **Files**: docs + plugin skills
- **Verify**: `rg heredoc plugins/` → **0 matches**

### ✅ Task 6: Supi-flow tools/skills update
- **Files**: supi-flow tools, skills, tests
- **Verify**: `pnpm exec vitest run` in plugins/supi-flow → **30 passed, 0 failed**

### ✅ Task 7: Migrate existing ticket files
- **Verify**: `tndm fmt --check` detects stale fingerprints (expected for old closed tickets)
