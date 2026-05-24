---
name: supi-flow-brainstorm
description: You MUST use this before any implementation. Clarify intent, shape the design, and get approval before touching code.
---

# Flow Brainstorm

Turn an idea into an approved design through focused collaboration. Default to a lightweight conversation. Add more structure only when the change is larger, riskier, or likely to span sessions.

<HARD-GATE>
Do NOT write code, scaffold anything, or take implementation action until you have presented a design and the user has approved it. This applies even to changes that seem simple.
</HARD-GATE>

## Core flow

1. **Explore context:** check relevant files, docs, recent commits, and existing tickets.
2. **Check scope:** if the request really contains multiple independent changes, decompose it before going deeper.
3. **Ask clarifying questions:** one at a time. Focus on purpose, constraints, and success criteria.
4. **Propose 2-3 approaches:** include trade-offs and a recommendation.
5. **Present the design:** scale detail to complexity and get approval.
6. **Classify and decide:** propose whether this change is trivial or non-trivial.
   - **Trivial:** single file, no tests/docs needed, one verification step, or user says "just do it".
     → "This looks trivial — skip ticket and implement directly?"
     If user agrees: implement directly, verify, done. No ticket.
   - **Non-trivial:** multi-file, needs tests or docs, multi-step, or likely multi-session.
     → Call `supi_flow_start` to create a TNDM ticket, then store the approved design in `content.md` via `supi_tndm_cli { action: "update", id: "<ID>", content: "<outcome>" }`.
7. **Recommend next step:**
   - If trivial: implement directly by following the approved design.
   - If non-trivial: `/supi-flow-plan <ID>`

## Understanding the idea

- Check the current project state first. Follow existing patterns before proposing new ones.
- Assess scope early. If the user is really asking for several subsystems, say so and help break the work into smaller changes.
- Ask one question per message. Multiple choice is great when it makes the decision easier.
- Keep refining until you understand the goal, non-goals, constraints, and what success looks like.

## Visual Companion

If upcoming questions would be easier with mockups, diagrams, or visual comparisons, offer a visual companion once:

> "Some of what we're working on could be easier to explain if I can show it to you visually. I can put together mockups, diagrams, comparisons, and other visuals as we go. Want to try it?"

That offer MUST be its own message. Do not combine it with any other content. If the user declines, continue in text.

Even if they accept, use visuals only when seeing the idea would help more than reading it.

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

## Persistence and tracking

After approval:

- **Default to conversation-first for trivial work.** Small single-session changes can keep the design in chat and implement directly without a ticket.
- **Create a ticket for non-trivial work.** When the change is multi-file, needs tests/docs, spans multiple steps, or is likely multi-session, call `supi_flow_start` and persist the approved design in `content.md`.
- If a ticket exists, save the design to the ticket's canonical `content.md` via `supi_tndm_cli { action: "update", id: "<ID>", content: "<outcome>" }`. During plan phase, keep task authoring separate: the overview stays in `content.md`, and executable tasks are later authored one at a time via `supi_flow_task`.
- **Retroactive escalation:** if a trivial change grows in scope mid-implementation, stop, create a retroactive ticket via `supi_flow_start`, and store a summary of completed work + new scope.

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

## Key principles

- One question at a time
- Explore alternatives before settling
- Scale rigor to risk
- Default to lightweight collaboration
- Keep the design clear enough to implement without guessing
