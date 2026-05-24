# Task 5: Rename _signal → signal in tool-specs.ts execute wrappers

In `extensions/tools/tool-specs.ts`, rename the `_signal` parameter to `signal` in all 7 execute function wrappers:

```typescript
// Before:
async execute(_toolCallId: string, params: Record<string, unknown>, _signal?: AbortSignal) {

// After:
async execute(_toolCallId: string, params: Record<string, unknown>, signal?: AbortSignal) {
```

And update the passthrough calls from `_signal` to `signal`:

```typescript
// Before:
return executeTndmCli(params as never, _signal);

// After:
return executeTndmCli(params as never, signal);
```

Applies to all 7 tools: supi_tndm_cli, supi_flow_start, supi_flow_plan, supi_flow_apply, supi_flow_task, supi_flow_complete_task, supi_flow_close.

**Verification**: `pnpm exec tsc --noEmit` passes.
