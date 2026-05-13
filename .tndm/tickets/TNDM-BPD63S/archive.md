# Archive

## Verification Results

### Task 1 — Core document model ✅
- `TicketDocument { name, path }` struct implemented
- `documents: Vec<TicketDocument>` on `TicketMeta` with default `content` doc
- `document_fingerprints: BTreeMap<String, String>` on `TicketState`
- Canonical TOML serialization, sorted docs by name
- **`cargo test -p tandem-core` → 56 passed**

### Task 2 — Storage fingerprinting ✅
- `fingerprint_file()` SHA-256 helper in `tandem-storage`
- `sync_ticket_documents()` — recomputes fingerprints, bumps revision
- `document_drift()` — detects stale/edited documents
- Legacy migration for `meta.toml` without `[[documents]]` section
- **`cargo test -p tandem-storage` → 29 passed**

### Task 3 — CLI doc create/sync/fmt-check ✅
- `tndm ticket doc create <id> <name>` — register + create document files
- `tndm ticket sync <id>` — refresh fingerprints after agent edits
- `tndm fmt --check` — reports stale fingerprints (confirmed working)
- **`cargo test -p tandem-cli` → 45 passed**

### Task 4 — Awareness diff ✅
- Fingerprint-only diff (no content embedding) in `awareness.rs`
- **Covered by tandem-core tests**

### Task 5 — TNDM plugin guidance ✅
- Zero heredocs or large content strings in `plugins/` (verified via `rg`)
- Skills reference `doc create`, `plan.md`, `archive.md`

### Task 6 — Supi-flow tools ✅
- `supi_flow_start` uses `tndmJson` for create
- `supi_flow_plan` creates `plan` doc via registry, writes file, syncs
- `supi_flow_complete_task` reads/writes plan doc, syncs
- `supi_flow_close` creates `archive` doc, writes results, syncs
- **`pnpm exec vitest run` → 30 passed, 0 failed**

### Task 7 — Migration ✅
- Legacy `load_ticket()` injects default `content` document for old tickets
- `tndm fmt --check` correctly flags stale fingerprints (exists < closed tickets) — expected ongoing maintenance
