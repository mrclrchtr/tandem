# Task 6: Replace includes("not found") with regex in executeFlowCompleteTask

In `extensions/tools/flow-tools.ts`, in `executeFlowCompleteTask`, replace the fragile string match:

```typescript
// Before (line ~410):
if (message.includes("not found")) {

// After:
if (/task\s+\d+\s+not\s+found/i.test(message)) {
```

This targets the specific "task N not found" pattern rather than matching any "not found" string, and survives minor CLI rewording (e.g., extra whitespace, capitalization changes).

**Verification**: `pnpm exec vitest run __tests__/flow-tools.test.ts` — the "throws when task number does not exist" test still passes (it throws `new Error("task 99 not found")` which matches the new regex).
