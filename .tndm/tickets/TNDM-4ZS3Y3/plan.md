## Plan: Unify SHA-256 fingerprint computation

### Pre-check validation
- `tandem-storage` is the right home: already has `sha2` dep, already has `fingerprint_file`, `tandem-cli` already depends on it
- `fingerprint_bytes` is infallible → returns `String`
- `fingerprint_file` → returns `Result<String, StorageError>`
- Tests check `sha256:` prefix — output format identical, no test changes needed

### Tasks

- [x] **Task 1**: Add public `fingerprint_bytes` and `fingerprint_file` functions to `tandem-storage/src/lib.rs`
  - Extract hex formatting from `fingerprint_file` into `pub fn fingerprint_bytes(data: &[u8]) -> String`
  - Refactor `fingerprint_file` to call `fingerprint_bytes` internally
  - Make `fingerprint_file` a public free function: `pub fn fingerprint_file(path: &Path) -> Result<String, StorageError>`
  - Verification: `cargo build -p tandem-storage`

- [x] **Task 2**: Replace inline hashing in `FileTicketStore::create_ticket` with `fingerprint_bytes`
  - In `tandem-storage/src/lib.rs` `create_ticket`, replace the inline `Sha256::new()` → `hasher.update()` → `format!("sha256:{}", ...)` block with `fingerprint_bytes(ticket.content.as_bytes())`
  - Verification: `cargo build -p tandem-storage`

- [x] **Task 3**: Replace inline hashing in `tandem-cli::handle_doc_create` with `fingerprint_file`
  - Add `fingerprint_file` to the `use tandem_storage::{...}` import in `tandem-cli/src/main.rs`
  - Replace the inline `sha2::Sha256::new()` → `hasher.update()` → `format!("sha256:{}", ...)` block with `fingerprint_file(&doc_path).map_err(|e| anyhow::anyhow!("{e}"))?`
  - Remove `use sha2::Digest;` if no longer needed
  - Verification: `cargo build -p tandem-cli`

- [x] **Task 4**: Full workspace build and test
  - `cargo build` — ensure no warnings
  - `cargo test` — ensure all existing tests pass, especially fingerprint tests in `ticket_store_tests.rs`
  - Verification: `cargo build && cargo test`
