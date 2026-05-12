## Design: Remove Claude Code & Codex plugin support

**Approach**: Delete all Claude Code / Codex plugin directories and config files. Strip plugin-related logic from tooling, CI, and docs. Keep `plugins/supi-flow` (PI extension) and all Rust crates untouched.

### Deletions
1. `plugins/tndm/` — entire directory (skills, hooks, plugin.json manifests)
2. `.claude-plugin/` — marketplace listing
3. `.claude/` — local Claude Code config + hooks + skills
4. `.codex/` — local Codex config
5. `.agents/` — agent marketplace config
6. `CLAUDE.md` — root Claude Code instruction file
7. `skills/` — duplicate ticket/awareness skills
8. `skills-lock.json` — skills.sh lockfile
9. `.github/workflows/upload-plugin.yml` — plugin release CI

### Edits
- `crates/xtask/src/main.rs` — remove sync-version subcommand, keep check-arch
- `mise.toml` — remove sync-version, sync-version-check, bump tasks
- `release-please-config.json` — remove plugin/marketplace extra-files
- `dist-workspace.toml` — remove upload-plugin from publish-jobs
- `hk.pkl` — remove sync-version linter; remove .agents/ and .claude/ from exclude
- `README.md` — remove Agent plugin section, keep PI section
- `docs/architecture.md` — remove plugins/tndm reference
- `docs/releasing.md` — remove plugin version-sync mention

### Not touched
- `plugins/supi-flow/` — PI extension
- All `crates/` — Rust workspace
- `.commitlintrc.json`, commitlint hook
- `context7.json`
