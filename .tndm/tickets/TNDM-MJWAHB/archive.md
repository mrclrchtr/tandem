# Archive

## Verification Summary

Archived `TNDM-MJWAHB` after a docs-only closeout. The implementation stayed within the approved scope: refresh `README.md` with a source-grounded overview of tandem's current capabilities and CLI/API design.

### Plan vs implementation
- Planned scope: `README.md` only.
- Actual scope: `README.md` only for product docs, plus the ticket's own `.tndm/` flow records.
- Task status: both planned tasks are complete and no deviation from the approved overview was needed.

### Fresh verification evidence

#### 1. README structure / content check
Ran a fresh section-presence validation script against `README.md`.

Observed results:
- `core capabilities section: OK`
- `cli design section: OK`
- `on-disk ticket model section: OK`
- `quick tour section: OK`
- `json api section: OK`
- `pi extension section: OK`
- `task detail ensure example: OK`
- `content_path mention: OK`
- `document_fingerprints mention: OK`
- `awareness tasks mention: OK`

This confirms the README now explicitly covers the capabilities and API/CLI topics required by the plan.

#### 2. Fresh CLI help validation
Ran these commands fresh and checked their output and exit status:
- `./tndm-dev --help`
- `./tndm-dev ticket --help`
- `./tndm-dev ticket create --help`
- `./tndm-dev ticket update --help`
- `./tndm-dev ticket doc create --help`
- `./tndm-dev ticket task --help`
- `./tndm-dev ticket task add --help`
- `./tndm-dev ticket task edit --help`
- `./tndm-dev ticket task set --help`
- `./tndm-dev ticket task detail --help`
- `./tndm-dev awareness --help`

Observed behavior:
- top-level CLI exposes `fmt`, `ticket`, and `awareness`
- `ticket` exposes `create`, `show`, `list`, `update`, `doc`, `task`, and `sync`
- `task` exposes `add`, `list`, `complete`, `remove`, `edit`, `detail`, and `set`
- `task detail` exposes `ensure` and `clear`
- create/update/doc/task help text matches the README's described flags and workflow
- all commands exited successfully

#### 3. Fresh source-anchor validation for README claims
Ran a fresh `rg` cross-check across the relevant source files and README.

Observed anchors:
- `crates/tandem-cli/src/cli/render.rs` exposes `content_path`
- `crates/tandem-cli/src/cli/util.rs` contains `ticket_content_path()` and `parse_ticket_id_input()`
- `crates/tandem-cli/src/cli/doc.rs` contains `normalize_ticket_relative_doc_path()` and document fingerprint recomputation
- `crates/tandem-core/src/ticket/state.rs` defines `document_fingerprints`
- `crates/tandem-core/src/awareness.rs` defines `AwarenessTasksDiff` and task/document diff reporting
- `crates/tandem-cli/src/cli/ticket.rs` contains the task detail ensure guidance and JSON envelope wiring using `content_path`

This confirms the README's descriptions of document fingerprints, bare-ID normalization, task detail docs, JSON envelopes, and awareness task/doc diffs still match the code.

#### 4. Fresh doc-delta review
Ran:
- `git diff --stat`
- `git diff -- README.md`

Observed result:
- the user-facing documentation delta is the intended `README.md` rewrite
- no additional living docs required updates after reviewing the actual change

#### 5. Fresh formatting / whitespace check
Ran:
- `git diff --check -- README.md`

Observed result:
- passed with `git diff --check -- README.md: OK`

### Conclusion
The change matches the approved intent, every planned task is complete, the README was verified against live CLI help and source anchors, and the docs-only closeout has fresh evidence.
