# Archive

- `cd plugins/supi-flow && pnpm exec tsc --noEmit` → exits 0
- `cd plugins/supi-flow && pnpm exec vitest run -v __tests__/flow-tools.test.ts __tests__/resources.test.ts` → 17 passed, 0 failed
- `cd plugins/supi-flow && pnpm exec vitest run -v __tests__/tndm-cli-tool.test.ts __tests__/flow-tools.test.ts` → 21 passed, 0 failed
- `rg -n "6 custom|supi_flow_task|state\.toml|task_json|detail_path|archive\.md" plugins/supi-flow/skills/supi-flow-plan/SKILL.md plugins/supi-flow/skills/supi-flow-brainstorm/SKILL.md plugins/supi-flow/README.md plugins/supi-flow/CLAUDE.md README.md CLAUDE.md` → matched updated docs and workflow guidance for the 6-tool model
- `cd plugins/supi-flow && pnpm exec vitest run -v` → 40 passed, 0 failed
