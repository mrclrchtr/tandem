# Archive

## Verification Results

### Fresh verification (2026-05-13)
- `cargo build` — 0 crates recompiled, clean
- `cargo clippy -p tandem-storage -p tandem-cli` — no issues found
- `cargo test` — 134 passed (16 suites), fingerprint-specific: 4 passed
- `cargo doc -p tandem-storage --no-deps` — built without warnings

### What was done
1. **Extracted `fingerprint_bytes`** — public free function in `tandem-storage` that computes SHA-256 fingerprint of a byte slice, returning `sha256:<hex>` string
2. **Extracted `fingerprint_file`** — public free function in `tandem-storage` that reads a file and fingerprints it, with `StorageError` for I/O errors
3. **Replaced inline hashing in `create_ticket`** — uses `fingerprint_bytes` instead of inline `Sha256::new()` / `.update()` / `.finalize()` / hex-format chain
4. **Replaced inline hashing in `handle_doc_create`** (tandem-cli) — uses `fingerprint_file` instead of the same chain; removed `sha2` dependency from tandem-cli

### Before/After
| Location | Before | After |
|----------|--------|-------|
| `tandem-storage` `fingerprint_file` | private associated fn, 16 lines | public free fn, 10 lines (delegates to `fingerprint_bytes`) |
| `tandem-storage` `create_ticket` | 10 lines inline hash | 1 line: `fingerprint_bytes(ticket.content.as_bytes())` |
| `tandem-cli` `handle_doc_create` | 11 lines inline hash | 1 line: `fingerprint_file(&doc_path)` |
| **Total** | **~37 lines duplicated 3x** | **~20 lines shared** |

### Files changed
- `crates/tandem-storage/src/lib.rs` — +33/-35 (net -2 lines)
- `crates/tandem-cli/src/main.rs` — +4/-16 (net -12 lines)
- `crates/tandem-cli/Cargo.toml` — -1 line (`sha2` dep removed)
- `Cargo.lock` — auto-updated
