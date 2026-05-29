# Task 1: Add registerTypedTool adapter and drop as-never casts

## Goal

Add a `registerTypedTool<T>()` function to `tool-specs.ts` that confines the `as never` cast to a single registration boundary. Drop all `as never` casts from the execute wrappers in `tool-specs.ts`.

## Files

- `extensions/tools/tool-specs.ts`

## Change

Add a generic adapter function before the `toolSpecs` array:

```typescript
import type { TObject } from "typebox";
import type { Static } from "typebox";
import type { ExtensionAPI, ToolContent } from "@earendil-works/pi-coding-agent";

type ToolResult = { content: ToolContent[]; details: Record<string, unknown> };

function registerTypedTool<T extends TObject>(
  pi: ExtensionAPI,
  spec: {
    name: string;
    label: string;
    description: string;
    promptSnippet: string;
    promptGuidelines: string[];
    executionMode: "sequential";
    parameters: T;
    execute: (toolCallId: string, params: Static<T>, signal?: AbortSignal) => Promise<ToolResult>;
  },
): void {
  pi.registerTool(spec as never);
}
```

Remove `as never` from all 7 execute wrappers in the `toolSpecs` array. Replace:

```typescript
executeTndmCli(params as never, signal)
```
with:
```typescript
executeTndmCli(params, signal)
```

Do the same for all 7 tools.

## Verification

- `pnpm exec tsc --noEmit` — zero type errors
- `pnpm exec vitest run __tests__/resources.test.ts` — all 7 tools still registered, `registerTypedTool` not imported in test (it's called from `index.ts`)
