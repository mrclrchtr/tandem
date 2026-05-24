---
name: supi-flow-brainstorm
description: You MUST use this before any implementation. Clarify intent, shape the design, and get approval before touching code.
---

# Flow Brainstorm

Turn an idea into an approved design through focused collaboration. Default to a lightweight conversation. Add more structure only when the change is larger, riskier, or likely to span sessions.

<HARD-GATE>
Do NOT write code, scaffold anything, or take implementation action until you have presented a design and the user has approved it. This applies even to changes that seem simple.
</HARD-GATE>

## Anti-Pattern: "This Change Is Too Small To Need Brainstorming"

Every change goes through this process — a one-line fix, a config tweak, a refactor, all of them. "Simple" changes are where unexamined assumptions cause the most rework. The design can be short (a few sentences for truly simple changes), but you MUST present it and get approval.

## Checklist

You MUST complete these items in order. Each step is expanded in the sections below.

1. **Explore context** — check relevant files, docs, recent commits, and existing tickets.
2. **Ask clarifying questions** — one at a time. Focus on purpose, constraints, and success criteria.
3. **Propose 2-3 approaches** — include trade-offs and a recommendation.
4. **Present the design** — scale detail to complexity, get approval per section.
5. **Classify and persist** — decide trivial vs non-trivial, then store the design (see [Classify and persist](#classify-and-persist)).
6. **Self-review** — run the four checks in [Self-review](#self-review), fix issues inline.
7. **User review gate** — pause and ask the user to review the written design before proceeding.
8. **Handoff** — present the outcome and recommend the next step.

## Understanding the idea

- Check the current project state first. Follow existing patterns before proposing new ones.
- **Assess scope before asking detailed questions.** If the request describes multiple independent changes, flag this immediately — don't spend questions refining details of work that needs decomposition first. Help the user break it into sub-projects: what are the independent pieces, how do they relate, what order should they be built? Then brainstorm the first sub-project through the normal flow. Each sub-project gets its own spec → plan → implementation cycle.
- Ask one question per message. Multiple choice is great when it makes the decision easier.
- Keep refining until you understand the goal, non-goals, constraints, and what success looks like.

## Exploring approaches

- Propose 2-3 approaches with trade-offs.
- Lead with your recommendation and say why.
- Prefer simple, well-bounded designs over sprawling ones.

## Presenting the design

Cover the parts that matter for the change, such as:

- approach
- main components or files
- data flow or control flow
- edge cases and error handling
- testing and verification
- docs to update, if any

Keep each section proportional to the complexity. A small change may only need a few sentences.

## Working in existing codebases

- Follow established patterns unless there is a strong reason not to.
- Include targeted cleanup when it directly helps the work.
- Do not propose unrelated refactors.

## Classify and persist

After the design is approved, decide how to store it.

**Trivial** — single file, no tests/docs needed, one verification step, or the user says "just do it":
- Keep the design in chat.
- Implement directly, verify, done. No ticket.

**Non-trivial** — multi-file, needs tests or docs, multi-step, or likely multi-session:
- Call `supi_flow_start` to create a TNDM ticket.
- Store the approved design in the ticket via `supi_tndm_cli { action: "update", id: "<ID>", content: "<outcome>" }`.
- During plan phase, keep task authoring separate: the overview stays in the ticket, and executable tasks are later authored one at a time via `supi_flow_task`.

**Retroactive escalation:** if a trivial change grows in scope mid-implementation, stop, create a retroactive ticket via `supi_flow_start`, and store a summary of completed work + new scope.

## Self-review

Before handing off:

1. **Placeholder scan** — any "TBD", "TODO", incomplete sections, or vague requirements? Fix them.
2. **Internal consistency** — do any sections contradict each other?
3. **Scope check** — is this focused enough for a single implementation plan, or does it need decomposition?
4. **Ambiguity check** — could any requirement be interpreted two ways? If so, pick one and make it explicit.

Fix issues inline, then continue.

## User review gate

After the design is persisted (in the ticket or in chat for trivial work), pause and ask the user to review it:

> "Design is ready in `<chat or ticket ID>`. Please review it and let me know if you want to make any changes before we proceed."

Wait for the user's response. If they request changes, make them and re-run the self-review. Only proceed once the user approves.

## Handoff

Present the outcome in a compact form:

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
- If non-trivial: `/supi-flow-plan TNDM-XXXXXX`
- If trivial: proceed with direct implementation

<HARD-GATE>
For non-trivial work, do NOT invoke any implementation skill or write code. The ONLY next step after brainstorming is `/supi-flow-plan`. Trivial work may proceed directly.
</HARD-GATE>

## Key principles

- One question at a time
- Explore alternatives before settling
- Scale rigor to risk
- Default to lightweight collaboration
- Keep the design clear enough to implement without guessing
