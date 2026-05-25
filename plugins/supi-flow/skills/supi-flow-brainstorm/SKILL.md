---
name: supi-flow-brainstorm
description: You MUST use this before any implementation. Clarify intent, shape the design, and get approval before touching code.
disable-model-invocation: true
---

# Flow Brainstorm

Turn an idea into an approved design through focused collaboration. Default to lightweight conversation; add structure only when the change is larger, riskier, or likely to span sessions. No change is too small — unexamined assumptions cause the most rework.

<HARD-GATE>
Do NOT write code, scaffold anything, or take implementation action until you have presented a design and the user has approved it. This applies even to changes that seem simple.
</HARD-GATE>

## Checklist

Complete these in order:

1. **Explore context** — relevant files, docs, recent commits, existing tickets.
2. **Ask clarifying questions** — one at a time, with a recommendation. Focus on purpose, constraints, and success criteria.
3. **Propose 2-3 approaches** — include trade-offs and a recommendation.
4. **Present the design** — scale detail to complexity; get approval per section.
5. **Classify and persist** — trivial (chat-only) or non-trivial (ticket). See below.
6. **Self-review** — run the four checks.
7. **User review gate** — pause for user review before proceeding.
8. **Handoff** — present outcome, recommend next step.

## Understanding the idea

- **Check project state first.** Follow existing patterns. Explore code, docs, and history before asking the user. Only ask when the answer requires judgment or intent.
- **Flag multi-scope requests early.** If the request spans independent changes, help decompose into sub-projects and brainstorm the first one.
- **Ask one question per message, with a recommended answer.** Guide — don't interrogate. The user can override, but shouldn't have to invent answers from scratch.
- **Walk every branch.** Resolve dependencies between decisions one-by-one. Keep refining until goals, non-goals, constraints, and success criteria are clear.
- **Include targeted cleanup** when it directly helps the work. Do not propose unrelated refactors.

## Exploring approaches

- Propose 2-3 approaches with trade-offs.
- Lead with your recommendation and say why.
- Prefer simple, well-bounded designs.

## Presenting the design

Cover the parts that matter for the change: approach, main components/files, data/control flow, edge cases and error handling, testing and verification, docs to update. Scale each section to complexity.

## Classify and persist

**Trivial** (single file, no tests/docs, one step, or "just do it"): keep the design in chat. Implement directly. No ticket.

**Non-trivial** (multi-file, needs tests/docs, multi-step, or likely multi-session): call `supi_flow_start`, then store the approved design via `supi_tndm_cli update content`. Task authoring happens later, during plan phase.

**Retroactive escalation:** if a trivial change grows mid-implementation, stop, call `supi_flow_start`, and store a summary of completed work plus new scope.

## Self-review

1. **Placeholder scan** — no TODOs, vague requirements, or incomplete sections.
2. **Consistency** — no contradictions between sections.
3. **Scope** — focused enough for one plan, or flagged for decomposition.
4. **Ambiguity** — pick one interpretation for any ambiguous requirement and state it explicitly.

Fix issues inline before handing off.

## User review gate

After the design is persisted, pause:

> "Design is ready in `<chat or ticket ID>`. Please review it and let me know if you want to make any changes before we proceed."

Wait for approval. Rerun self-review after any changes.

## Handoff

```markdown
## Brainstorming Outcome
**Problem**: ...
**Recommended approach**: ...
**Why**: ...
**Constraints / non-goals**: ...
**Open questions**: ...
**Ticket**: TNDM-XXXXXX / none
```

Then recommend:
- Non-trivial: `/skill:supi-flow-plan TNDM-XXXXXX`
- Trivial: proceed with direct implementation

<HARD-GATE>
For non-trivial work, do NOT invoke any implementation skill or write code. The ONLY next step after brainstorming is `/skill:supi-flow-plan`. Trivial work may proceed directly.
</HARD-GATE>
