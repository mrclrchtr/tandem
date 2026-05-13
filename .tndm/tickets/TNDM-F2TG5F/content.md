## Brainstorming Outcome
**Problem**: `supi-flow` currently splits real workflow state across `brainstorm.md`, `plan.md`, and `archive.md` while TNDM still treats `content.md` as the canonical ticket body. Skills, docs, and tool behavior are inconsistent about file layout, document reading, and phase handling.

**Recommended approach**: Adopt a single primary ticket body plus phase attachments. Use `content.md` as the canonical live ticket body / approved design summary, keep `plan.md` for executable implementation tasks, keep `archive.md` for final verification evidence, and remove `brainstorm.md`. Update skills and tools to use registered document paths instead of inferred filenames, and align docs/prompts/commands around that model.

**Why**: This aligns `supi-flow` with the TNDM CLI, reduces duplication, improves resumability for agents, and removes hidden assumptions like deriving `plan.md` from `content_path` or assuming `ticket show` includes document contents.

**Constraints / non-goals**: Keep the existing brainstorm → plan → apply → archive phase model. Avoid turning this into a full workflow engine redesign. Minimize ceremony while preserving explicit approval, planning, verification, and archive steps.

**Open questions**:
- Should `content.md` hold only the approved design summary, or also a compact live status summary during apply?
- Should `/supi-flow-status` be redesigned to query TNDM truth instead of scanning session messages?

**Classification**: Non-trivial
**Ticket**: TNDM-F2TG5F
