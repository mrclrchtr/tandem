# CLAUDE.md

Guidance for Claude Code when working on the supi-flow plugin.

## Purpose

`supi-flow` is a **PI-only extension** (not a Claude Code plugin) that implements a spec-driven workflow (brainstorm → plan → apply → archive) coupled to mandatory TNDM ticket coordination. It ships inside the tandem repository under `plugins/supi-flow/`.

It registers 5 custom PI tools (`supi_tndm_cli`, `supi_flow_start`, `supi_flow_plan`, `supi_flow_complete_task`, `supi_flow_close`) and auto-discovers 6 flow skills from `skills/`. All `tndm` CLI interactions go through these tools — agents should not shell out to `tndm` directly.

## Relationship to the tandem repo

- **tandem** (Rust) provides the `tndm` CLI that this plugin shells out to via `child_process.execFile`.
- This plugin is consumed by pi (not Claude Code), so its `package.json` uses `pi.extensions` instead of a Claude Code `plugin.json` manifest.
- The sibling `plugins/tndm/` is a separate Claude Code plugin that teaches agents to use `tndm` directly. This plugin wraps those same operations in structured PI tools.

## File structure

```
plugins/supi-flow/
├── src/
│   ├── index.ts          # Extension entry point — registers tools, commands, resource discovery
│   ├── cli.ts            # Node.js wrappers around tndm / git via child_process.execFile
│   └── tools/
│       ├── tndm-cli.ts   # supi_tndm_cli tool (create, update, show, list, awareness)
│       └── flow-tools.ts # supi_flow_start, supi_flow_plan, supi_flow_complete_task, supi_flow_close
├── skills/               # 6 flow skills (auto-discovered by pi)
├── prompts/              # supi-coding-retro prompt template
├── __tests__/
│   ├── resources.test.ts # Extension registration + resource discovery tests
│   └── cli.test.ts       # Unit tests for cli.ts (vitest mocks)
├── package.json
├── tsconfig.json
├── vitest.config.ts
└── pnpm-lock.yaml
```

## Development commands

```sh
cd plugins/supi-flow
pnpm install

# Type-check
pnpm exec tsc --noEmit

# Run tests
pnpm exec vitest run

# Run a single test file
pnpm exec vitest run __tests__/cli.test.ts
```

## Coding conventions

- **TypeScript** with `strict: true`, `target: ES2022`, `module: ES2022`.
- **TypeBox** (`typebox`) for tool parameter schemas — use `Type.Object`, `Type.Optional`, `Type.String`, `Type.Number`, `Type.Boolean`, and `StringEnum` (from `@earendil-works/pi-ai`).
- Tool execute functions return `{ content: ToolContent[], details: Record<string, unknown> }`.
- `cli.ts` wraps `child_process.execFile` — never use `exec` (shell injection risk). Use `tndm()` for raw output, `tndmJson()` for `--json` output, `gitAddCommit()` for committing `.tndm/` changes.
- Tests use **vitest** with `vi.mock` to stub `child_process.execFile`.

## Tool registration pattern

1. Define parameters with TypeBox in the tool's source file.
2. Export the schema and execute function.
3. Register in `src/index.ts` via `pi.registerTool({ name, label, description, promptSnippet, promptGuidelines, parameters, execute })`.
4. Add a test in `__tests__/resources.test.ts` verifying the tool name appears in the registered tools list.

## Skill conventions

- Skills live in `skills/<name>/SKILL.md` and are auto-discovered by pi.
- Skills reference tools (e.g. `supi_tndm_cli`, `supi_flow_start`) with structured parameter examples — never raw `tndm` CLI commands.
- Task numbering in plans uses `**Task N**` format — `supi_flow_complete_task` relies on this convention.

## When changing this plugin

- Update `__tests__/resources.test.ts` if adding or removing tools.
- If a new npm dependency is added, run `pnpm install` to update `pnpm-lock.yaml`.
- Bump `version` in `package.json` following semantic versioning.
- The tandem repo's root `CLAUDE.md` references this plugin — keep the description there current.
