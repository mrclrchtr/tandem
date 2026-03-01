---
name: agent-orchestrator
description: Orchestrate complex work via a phase-gated multi-agent loop (audit → design → implement → review → validate → deliver). Use when you need to split work into subsystems, run independent audits, reconcile findings into a confirmed issue list, delegate fixes in clusters, enforce PASS/FAIL review gates, and drive an end-to-end validated delivery. Do not use for small, single-file tasks.
---

# Agent Orchestrator

## Overview

Orchestrate multi-agent work end-to-end: delegate audits and fixes, reconcile results, enforce quality gates, and deliver a validated outcome.

Follow this core pattern: delegate a fresh implementer per cluster, then run a two-stage review (spec compliance first, then code quality).

Non-negotiable rule: **never** implement changes directly (no coding, no file edits).

## Agent Roles (incl. tiered variants — choose per task complexity)

Assume these roles are available in your environment. Choose tiered variants based on task complexity and risk.
Do not edit agent definitions or configs. If a required role is missing, stop and ask the operator to configure it.

- `architect` (design/decisions/contracts)
- `auditor` / `auditor_high` (read-only issue finding; no fixes)
- `explorer` / `scout` (read-only repo lookup)
- `implementer_medium` / `implementer` / `implementer_xhigh` (code + tests)
- `spec_reviewer` (read-only, PASS/FAIL: nothing missing, nothing extra)
- `quality_reviewer` (read-only, PASS/FAIL: maintainability + test quality)

## Workflow

1. Use skills when they directly match a subtask
   - If a skill matches the task, invoke it explicitly and follow it (e.g., `$web-fetch-to-markdown <url>`).
   - When delegating, tell sub-agents which skill to use in their prompt (e.g., “Use `$git-commit` for the commit step.”).

2. Freeze scope + success criteria
   - Restate the mission, constraints, and “done” criteria in concrete terms.
   - Identify any authoritative sources (docs/specs) and record what claims must be backed by evidence.

3. Create a phase plan and keep it current
   - Use your environment’s planning mechanism (e.g., `update_plan` if available) to track phases and prevent drifting.
   - Prefer 4–7 steps; keep exactly one step in progress.

4. Decompose into subsystems
   - Choose subsystems that can be audited independently (API surface, core logic, error handling, perf, integrations, tests, docs).
   - For each subsystem, define 2–5 invariants (what must always be true).

5. Run dual independent audits per subsystem
   - Choose the audit tiers:
     - Default: spawn `auditor` + `auditor` (fast/cheap).
     - High-risk or subtle work (security, auth, money, data loss, concurrency, cross-module interactions, or when prior audits disagree): spawn `auditor_high` + `auditor` (one deep, one fast).
     - Maximum assurance: spawn `auditor_high` + `auditor_high`.
   - Spawn two independent auditors per subsystem (auditA and auditB) using the chosen tiers.
   - Tell them to work independently until reconciliation (no cross-talk).
   - Require evidence for every issue (repo location, deterministic repro, expected vs actual, severity).

6. Reconcile audits into a single confirmed issue list
   - Compare auditA vs auditB outputs and keep only mutually confirmed issues (or independently verify disputed ones with `explorer`).
   - Track rejected candidates with a brief reason (weak evidence, out of scope, non-deterministic).
   - Use this reconciled list as the only input to implementation.
   - Reconciliation output:
     - Confirmed issues (only mutual)
     - Rejected candidates (reason)
     - Consensus achieved: YES/NO

7. Implement in clusters with clear ownership
   - Group confirmed issues into clusters that can be fixed with minimal coupling.
   - Spawn exactly one `implementer` tier per cluster:
     - Use `implementer_medium` for trivial, low-risk edits.
     - Use `implementer` for most work.
     - Use `implementer_xhigh` for tricky bugs, risky refactors, or high-stakes changes.
   - Assign each implementer a file set to “own” and require them to avoid broad refactors.
   - Do not implement any cluster work directly; always delegate to the implementer (even for “quick” changes).
   - Every fix must come with a regression test (unit/integration/e2e as appropriate).
   - For each cluster, run a two-stage review loop:
      - Have the implementer complete the cluster (tests, self-review) and report what changed.
      - `spec_reviewer` validates “nothing more, nothing less” by reading code (do not trust the report).
      - `quality_reviewer` validates maintainability and test quality (only after spec compliance passes).
      - If any review FAILs, send concrete feedback to the implementer and repeat the failed review stage.

8. Enforce review gates
   - Do not merge/land a cluster unless spec compliance PASS and code quality PASS are both recorded with concrete references.

9. Integrate + validate
   - Run the repo’s standard validations (tests, lint, build, typecheck).
   - If the repo has no clear commands, discover them from `README`, `package.json`, `pyproject.toml`, CI config, etc.

10. Deliver a concise completion report
    - State what is usable now.
    - State what remains intentionally unsupported (with next steps/issues).
    - List commands executed (at least key validation commands) and results.

## What to send to sub-agents

Keep your messages task-specific and concise. Do not restate generic role behavior; focus on the task at hand.

For any audit/review/implementation message, include:
- Goal + success criteria (what “done” means)
- Scope boundaries / owned files (what to touch, what not to touch)
- Invariants (2–5) that must hold
- Commands to run (if known), and what evidence to collect
