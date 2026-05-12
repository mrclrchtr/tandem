# CLAUDE.md

Guidance for Claude Code when working on the supi-flow plugin.

## Purpose

`supi-flow` is a **PI-only extension** (not a Claude Code plugin) that implements a spec-driven workflow (brainstorm → plan → apply → archive) coupled to TNDM ticket coordination for non-trivial changes. Trivial changes can be implemented directly without a ticket. It ships inside the tandem repository under `plugins/supi-flow/`.

It registers 5 custom PI tools (`supi_tndm_cli`, `supi_flow_start`, `supi_flow_plan`, `supi_flow_complete_task`, `supi_flow_close`) and auto-discovers 5 flow skills from `skills/`. All `tndm` CLI interactions go through these tools (agents should not shell out to `tndm` directly).

## PI-specific guardrails

- Never guess PI extension APIs or conventions from memory; read the installed PI docs first (`README.md`, `docs/index.md`, relevant files in `docs/`, and matching `examples/`) and follow linked `.md` cross-references.
- PI loads this extension directly from the working tree; after editing `extensions/`, `skills/`, or `prompts/`, use `/reload` or restart PI before validating behavior.
- This package uses PI's conventional directory structure (no `pi` manifest in `package.json`): `extensions/`, `skills/`, and `prompts/` are all auto-discovered by pi.
- Do not register a `resources_discover` handler for `skills/` or `prompts/` paths — pi already auto-discovers them from convention directories. Returning already-discovered paths causes duplicate-discovery warnings at startup.

## Relationship to the tandem repo

- **tandem** (Rust) provides the `tndm` CLI that this plugin shells out to via `child_process.execFile`.
- This plugin is consumed by pi (not Claude Code), so its `package.json` uses PI's conventional directory structure instead of a Claude Code `plugin.json` manifest.
- The sibling `plugins/tndm/` is a separate Claude Code plugin that teaches agents to use `tndm` directly. This plugin wraps those same operations in structured PI tools.

## File structure

```
plugins/supi-flow/
├── extensions/
│   ├── index.ts          # Extension entry point: registers tools and commands
│   ├── cli.ts            # Node.js wrappers around tndm / git via child_process.execFile
│   └── tools/
│       ├── tndm-cli.ts   # supi_tndm_cli tool (create, update, show, list, awareness)
│       └── flow-tools.ts # supi_flow_start, supi_flow_plan, supi_flow_complete_task, supi_flow_close
├── skills/               # 5 flow skills (auto-discovered by pi)
├── prompts/              # supi-coding-retro prompt template
├── __tests__/
│   ├── resources.test.ts # Extension registration tests
│   ├── index.test.ts     # Command-level tests for /supi-flow and /supi-flow-status
│   ├── cli.test.ts       # Unit tests for cli.ts (vitest mocks)
│   └── flow-tools.test.ts# Unit tests for flow tools
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

## Verification shortcuts

- After changing `extensions/index.ts`, command behavior, or tool schemas, run `pnpm exec tsc --noEmit && pnpm exec vitest run __tests__/index.test.ts __tests__/resources.test.ts`.
- After changing `extensions/cli.ts` or tool execution paths, run `pnpm exec vitest run __tests__/cli.test.ts`.
- After changing `skills/` or `prompts/`, `/reload` or restart PI before validating behavior.

## Coding conventions

- **TypeScript** with `strict: true`, `target: ES2022`, `module: ES2022`.
- **TypeBox** (`typebox`) for tool parameter schemas: `Type.Object`, `Type.Optional`, `Type.String`, `Type.Number`, `Type.Boolean`, and `StringEnum` (from `@earendil-works/pi-ai`).
- Tool execute functions return `{ content: ToolContent[], details: Record<string, unknown> }`.
- `cli.ts` wraps `child_process.execFile` (never use `exec`; shell injection risk). Use `tndm()` for raw output, `tndmJson()` for `--json` output, `gitAddCommit()` for committing `.tndm/` changes.
- Tests use **vitest** with `vi.mock` to stub `child_process.execFile`.

## Tool registration pattern

1. Define parameters with TypeBox in the tool's source file.
2. Export the schema and execute function.
3. Register in `extensions/index.ts` via `pi.registerTool({ name, label, description, promptSnippet, promptGuidelines, parameters, execute })`.
4. Add a test in `__tests__/resources.test.ts` verifying the tool name appears in the registered tools list.
5. Prefer stable guidance in `promptGuidelines`; PI flattens these bullets into the system prompt, so each bullet should name the tool it governs.

## Skill conventions

- Skills live in `skills/<name>/SKILL.md` and are auto-discovered by pi.
- Skills reference tools (e.g. `supi_tndm_cli`, `supi_flow_start`) with structured parameter examples, never raw `tndm` CLI commands.
- `content.md` is the canonical approved-design body, `plan.md` is the executable checklist, and `archive.md` stores final verification evidence.
- Older tickets may still contain a legacy brainstorm sidecar document, but new flow behavior should not create it or depend on it.
- Task numbering in plans uses `**Task N**` format; `supi_flow_complete_task` relies on this convention.

## When changing this plugin

- Update `__tests__/resources.test.ts` if adding or removing tools.
- If a new npm dependency is added, run `pnpm install` to update `pnpm-lock.yaml`.
- Bump `version` in `package.json` following semantic versioning.
- The tandem repo's root `CLAUDE.md` references this plugin; keep the description there current.
- Keep `@earendil-works/pi-*` peer dependency ranges at `"*"`; put non-PI runtime deps in `dependencies`, not `peerDependencies`.
