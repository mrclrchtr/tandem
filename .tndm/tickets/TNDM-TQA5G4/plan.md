## Plan

- [x] **Task 1**: Delete all Claude Code / Codex directories and files
  - Files: `plugins/tndm/`, `.claude-plugin/`, `.claude/`, `.codex/`, `.agents/`, `CLAUDE.md`, `skills/`, `skills-lock.json`, `.github/workflows/upload-plugin.yml`
  - Verification: `ls` confirms all paths are gone; `rg -r "plugins/tndm" --glob='!target/**'` returns no results

- [x] **Task 2**: Strip sync-version from `crates/xtask/src/main.rs`
  - File: `crates/xtask/src/main.rs`
  - Verification: `cargo xtask check-arch` passes; `cargo xtask sync-version` prints usage error

- [x] **Task 3**: Remove plugin tasks from `mise.toml`
  - File: `mise.toml`
  - Verification: `mise run sync-version` fails (task not found)

- [x] **Task 4**: Remove plugin extra-files from `release-please-config.json`
  - File: `release-please-config.json`
  - Verification: `jq` parse succeeds; no plugin paths remain

- [x] **Task 5**: Remove upload-plugin from `dist-workspace.toml`
  - File: `dist-workspace.toml`
  - Verification: grep confirms no `upload-plugin` reference

- [x] **Task 6**: Remove sync-version linter and stale excludes from `hk.pkl`
  - File: `hk.pkl`
  - Verification: `pkl eval hk.pkl` succeeds; `mise run check` passes

- [x] **Task 7**: Remove Agent plugin section from `README.md`
  - File: `README.md`
  - Verification: grep confirms no Claude Code / Codex / skills.sh references remain

- [x] **Task 8**: Remove `plugins/tndm` reference from `docs/architecture.md`
  - File: `docs/architecture.md`
  - Verification: grep confirms no `plugins/tndm` reference

- [x] **Task 9**: Remove plugin version-sync mention from `docs/releasing.md`
  - File: `docs/releasing.md`
  - Verification: grep confirms no `plugin manifest` reference

- [x] **Task 10**: Full verification — build, arch, and reference scan
  - Verification: `cargo build` succeeds; `mise run arch` passes; `rg -r "claude-plugin|codex-plugin|plugins/tndm" --glob='!.git/**' --glob='!target/**' --glob='!node_modules/**'` returns no results (except possibly in supi-flow which is allowed)
