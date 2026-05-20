## Goal
Refresh the root `README.md` so it accurately reflects tandem's current source-backed capabilities and API/CLI design.

## Scope
Limit the implementation to `README.md`. Keep the existing high-level positioning, install guidance, and quick-start value, but expand the README so a new reader can understand:

- what tandem can do today
- how the `tndm` CLI is organized
- how ticket data is stored on disk
- how document registration and fingerprint verification work
- how agent-friendly JSON output and awareness diffs are shaped
- how the PI `supi-flow` extension relates to the core project

## Files
- `README.md` — rewrite and reorganize the top-level project documentation around the current source-backed capabilities and CLI surface.

## Planned README changes
1. Keep the current project introduction, installation guidance, and quick tour, but tighten wording where needed to match the current product language from `docs/vision.md` and the Rust workspace architecture.
2. Add a capability-oriented section that explicitly covers ticket lifecycle management, document registry + fingerprint freshness checks, task tracking and task detail docs, git-aware awareness across branches/worktrees, deterministic formatting, and agent-first JSON output.
3. Add a CLI design section that documents the actual command hierarchy from the source and `--help` output: top-level `fmt`, `ticket`, and `awareness`; nested `ticket create/show/list/update/doc/sync/task`; and nested task operations including `detail ensure|clear` and bulk `set`.
4. Add an on-disk model section explaining the `.tndm/tickets/<ID>/` layout (`meta.toml`, `state.toml`, `content.md`, optional registered docs and `tasks/task-XX.md`) and the split between stable metadata and volatile state.
5. Add an API / JSON section describing the schema-versioned JSON responses, flattened ticket envelopes with `content_path`, and awareness field diffs for metadata, documents, and tasks.
6. Keep the README concise enough to avoid becoming a full command reference; link readers to `docs/vision.md`, `docs/architecture.md`, and `docs/decisions.md` for deeper detail.

## Verification
This is a docs-only, test-exempt change. Verify accuracy by comparing the revised README against the current source and live CLI help output, including:

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

Also cross-check the README's storage and JSON descriptions against:

- `crates/tandem-core/src/ticket/meta.rs`
- `crates/tandem-core/src/ticket/state.rs`
- `crates/tandem-core/src/awareness.rs`
- `crates/tandem-cli/src/cli/ticket.rs`
- `crates/tandem-cli/src/cli/doc.rs`
- `crates/tandem-cli/src/cli/render.rs`
- `crates/tandem-cli/src/cli/util.rs`
