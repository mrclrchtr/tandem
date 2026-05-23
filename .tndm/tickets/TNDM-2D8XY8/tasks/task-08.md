# Task 8: Update CLAUDE.md task guidance for mandatory detail docs

## Goal

Replace the "Use headline-only tasks when possible" guidance with documentation that every task always gets a canonical detail doc automatically.

## Change

In `plugins/supi-flow/CLAUDE.md`:
- Find: `Use headline-only tasks when possible. If a task needs real implementation detail or notices, attach an optional \`tasks/task-XX.md\` task doc after the task already exists.`
- Replace with guidance like: `Every task gets a canonical \`tasks/task-XX.md\` detail doc automatically at creation time. ...`

Also check `plugins/supi-flow/skills/supi-flow-plan/SKILL.md` — it mentions headline-only tasks too and should be updated.
