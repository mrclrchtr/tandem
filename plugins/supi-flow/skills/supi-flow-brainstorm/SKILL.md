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

## Anti-Pattern: Stopping questions too early

The most common failure is jumping to approaches after 1-2 questions. The user nods along, you think you understand — but you haven't walked every branch. If you haven't covered purpose, non-goals, constraints, edge cases, and success criteria, you're not done. Ask another question.

## Checklist

Complete these in order:

1. **Explore context** — relevant files, docs, recent commits, existing tickets.
2. **Ask clarifying questions** — one at a time, with a recommendation. Walk every branch of the decision tree before moving on.
3. **Propose 2-3 approaches** — include trade-offs and a recommendation. One of the 2-3 should be intentionally "crazy" to surface overlooked ideas.
4. **Present the design** — scale detail to complexity; get approval per section.
5. **Classify and persist** — trivial (chat-only) or non-trivial (ticket). See below.
6. **Self-review** — run the four checks.
7. **User review gate** — pause for user review before proceeding.
8. **Handoff** — present outcome, recommend next step.

## Understanding the idea

- **Check project state first.** Follow existing patterns. Explore code, docs, and history before asking the user. Only ask when the answer requires judgment or intent.
- **Flag multi-scope requests early.** If the request spans independent changes, help decompose into sub-projects and brainstorm the first one.
- **Ask one question per message, with a recommended answer.** Guide — don't interrogate. The user can override, but shouldn't have to invent answers from scratch.
- **Walk every branch relentlessly.** Resolve dependencies between decisions one-by-one. For each answer, ask what it implies and what it rules out. Keep refining until goals, non-goals, constraints, edge cases, and success criteria are clear.
- **Go back when needed.** When a new answer contradicts an earlier assumption, loop back and clarify.
- **Include targeted cleanup** when it directly helps the work. Do not propose unrelated refactors.

## Exploring approaches

- Propose 2-3 total approaches with trade-offs.
- Make one of those approaches intentionally "crazy" — unconventional, high-variance, or overpowered for the problem — so you can surface ideas that a conservative design might miss.
- Lead with your recommendation and say why.
- Prefer simple, well-bounded designs for the final recommendation.

## Presenting the design

Cover the parts that matter for the change: approach, main components/files, data/control flow, edge cases and error handling, testing and verification, docs to update. Scale each section to complexity.

## Classify and persist

**Trivial** (single file, no tests/docs, one step, or "just do it"): keep the design in chat. Implement directly. No ticket.

**Non-trivial** (multi-file, needs tests/docs, multi-step, or likely multi-session): call `supi_flow_start`, then store the approved design via `supi_tndm_cli update content`. Task authoring happens later, during plan phase.

**Retroactive escalation:** if a trivial change grows mid-implementation, stop, call `supi_flow_start`, and store a summary of completed work plus new scope.

## Self-review

**Design Self-Review:** After writing the design document, look at it with fresh eyes:

- **Placeholder scan:** Any "TBD", "TODO", incomplete sections, or vague requirements? Fix them.
- **Internal consistency:** Do any sections contradict each other? Does the architecture match the feature descriptions?
- **Scope check:** Is this focused enough for a single implementation plan, or does it need decomposition?
- **Ambiguity check:** Could any requirement be interpreted two different ways? If so, pick one and make it explicit. Use specific, unambiguous language — every claim in the design should survive a different agent reading it without context.

Fix any issues inline. No need to re-review — just fix and move on.

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
