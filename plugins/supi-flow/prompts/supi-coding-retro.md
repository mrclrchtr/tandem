---
description: Agent retrospective on project setup, architecture, tooling, and workflows
---

# Agent Retrospective — Project Setup & Tooling

Reflect on what made development harder than necessary during this coding session.

Focus only on friction points actually encountered while implementing, debugging, validating, or navigating the task. Do not infer or invent issues from the repository structure, architecture, codebase shape, or available tooling alone.

Do not do additional research or inspect the repository again. Use only your memory of this session.

If no concrete friction points were encountered, write exactly:

> No concrete friction points encountered during this session.

Consider concrete examples such as:
- sources of friction or unnecessary complexity
- confusing or inconsistent patterns
- tooling or processes that slowed implementation
- over-engineered abstractions encountered during the work
- repetitive manual work that could be automated
- missing documentation, scripts, tests, or conventions
- setup, validation, CI, or workflow issues
- anything that should be simplified, standardized, removed, or made more obvious

Be honest and specific. The goal is to identify practical improvements that would reduce unnecessary work, reduce token usage, reduce debugging, or improve outcomes in future sessions. This is not a complaint log.

## Output Format

Group items under the sections below. Skip sections that have no concrete items.

Sections are grouped by where the fix should happen. Each friction point should appear once, under the best-fitting section.

Order items by expected future work avoided, highest first.

Each item should include:

- **Problem**: What concrete friction occurred?
- **Impact**: What extra work, risk, delay, token usage, or confusion it caused.
- **Fix**: The smallest concrete change that would prevent or reduce it next time.

## Sections

### Repository Changes

Changes to files, code organization, architecture, abstractions, conventions, tests, or docs.

### Tooling & Process Changes

Changes to scripts, commands, setup flows, CI checks, local validation, automation, or manual workflows.

### Harness / Agent Tool Changes

Changes to the agent harness, available tools, PI extensions, context providers, retrieval tools, or generated prompt context.

### Prompt / Context Changes

Changes to the coding-session prompt, repo instructions, task framing, generated context, or other information shown to the agent.

### Other

Anything that should be simplified, standardized, removed, or made more obvious that does not fit above.
