## Goal
Refresh `plugins/supi-flow/README.md` so it accurately describes the current `supi-flow` package as a PI-only extension/package, its workflow, shipped resources, installation paths, and development/verification expectations.

## Scope
Limit implementation to `plugins/supi-flow/README.md`. Use the current plugin source, tests, and PI docs as the source of truth.

## Findings driving the rewrite
- The current README is partly stale relative to the current source and tests.
- It under-explains the task-capable `supi_tndm_cli` surface.
- It does not mention the startup `tndm`↔`supi-flow` version mismatch warning.
- Its local install guidance is more extension-entrypoint-centric than package-centric, which under-documents how bundled skills and prompts are loaded from a local package path.
- It does not clearly position the package as using PI's conventional directories (`extensions/`, `skills/`, `prompts/`).

## Planned README shape
Use a user-first package README with these sections:

1. **What `supi-flow` is**
   - PI-only extension/package for spec-driven workflow on top of TNDM tickets.
   - Trivial changes can skip tickets; non-trivial changes use the full flow.

2. **What ships in the package**
   - 5 custom tools
   - 5 skills
   - 1 prompt template
   - startup/reload version check for `tndm` vs package version
   - conventional-directory package layout

3. **Install / load**
   - `pi install npm:@mrclrchtr/supi-flow`
   - local package-path install via `pi install ./plugins/supi-flow` or settings `packages` entry
   - clarify when to use package install vs loading only an extension entrypoint

4. **Workflow and ticket model**
   - brainstorm → plan → apply → archive
   - overview-first `content.md`
   - structured tasks in `state.toml`
   - optional `tasks/task-XX.md`
   - `archive.md` written at closeout

5. **Tool / skill summaries**
   - concise tables grounded in the current extension code
   - mention `supi_tndm_cli` task operations without turning README into a full API reference

6. **Development / verification**
   - install, type-check, tests, targeted verification, and `/reload` guidance

## Files
- `plugins/supi-flow/README.md` — rewrite to match the current package behavior and PI package conventions.

## Verification
This is a docs-only, test-exempt change. Verify by checking the final README against:

- `plugins/supi-flow/package.json`
- `plugins/supi-flow/extensions/index.ts`
- `plugins/supi-flow/extensions/tools/tndm-cli.ts`
- `plugins/supi-flow/extensions/tools/flow-tools.ts`
- `plugins/supi-flow/extensions/cli.ts`
- `plugins/supi-flow/__tests__/resources.test.ts`
- `plugins/supi-flow/__tests__/index.test.ts`
- PI docs:
  - `README.md`
  - `docs/extensions.md`
  - `docs/packages.md`
  - `docs/skills.md`

The README should accurately describe package installation, auto-discovered resources, workflow phases, tools/skills/prompts, version-check behavior, and development commands without claiming unsupported commands or behavior.
