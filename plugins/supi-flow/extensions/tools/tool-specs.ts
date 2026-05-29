import type { TObject, Static } from "typebox";
import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";
import { type ToolResult } from "./ticket-helpers.js";
import { supi_tndm_cli_params, executeTndmCli } from "./tndm-cli.js";
import {
  supiFlowStartParams,
  supiFlowPlanParams,
  supiFlowApplyParams,
  supiFlowTaskParams,
  supiFlowCompleteTaskParams,
  supiFlowCloseParams,
  executeFlowStart,
  executeFlowPlan,
  executeFlowApply,
  executeFlowTask,
  executeFlowCompleteTask,
  executeFlowClose,
} from "./flow-tools.js";

/**
 * Typed registration adapter: confines the `as never` cast to a single boundary,
 * so tool-spec execute functions can use typed params without `as never` everywhere.
 */
export function registerTypedTool<T extends TObject>(
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

export type ToolSpec = {
  name: string;
  label: string;
  description: string;
  promptSnippet: string;
  promptGuidelines: string[];
  executionMode: "sequential";
  parameters: TObject;
  execute: (toolCallId: string, params: Record<string, unknown>, signal?: AbortSignal) => Promise<ToolResult>;
};

// Helper to wrap a typed execute function into the Record<string, unknown> signature.
// The cast is confined to one place rather than 7 separate execute wrappers.
function typedExecute<T>(
  fn: (toolCallId: string, params: T, signal?: AbortSignal) => Promise<ToolResult>,
): (toolCallId: string, params: Record<string, unknown>, signal?: AbortSignal) => Promise<ToolResult> {
  return (toolCallId, params, signal) => fn(toolCallId, params as T, signal);
}

/**
 * Ordered definitions of the seven public supi-flow tools.
 * Each entry is shaped to match pi.registerTool() expectations.
 */
export const toolSpecs: ToolSpec[] = [
  // ── supi_tndm_cli ──────────────────────────────────────────
  {
    name: "supi_tndm_cli",
    label: "TNDM CLI",
    description:
      "Direct wrapper for tndm ticket and task operations. Use instead of running tndm via bash.",
    promptSnippet: "Run direct tndm ticket/task operations",
    promptGuidelines: ["Use supi_tndm_cli for direct tndm operations instead of bash."],
    executionMode: "sequential" as const,
    parameters: supi_tndm_cli_params,
    execute: typedExecute<Static<typeof supi_tndm_cli_params>>(
      async (_toolCallId, params, signal) => executeTndmCli(params, signal),
    ),
  },

  // ── supi_flow_start ────────────────────────────────────────
  {
    name: "supi_flow_start",
    label: "Flow Start",
    description:
      "Create a flow:brainstorm ticket for non-trivial work and store known context.",
    promptSnippet: "Create a brainstorm ticket for non-trivial work",
    promptGuidelines: [
      "Use supi_flow_start for non-trivial work that needs a durable ticket.",
      "Do not use supi_flow_start when the user explicitly wants direct implementation.",
    ],
    executionMode: "sequential" as const,
    parameters: supiFlowStartParams,
    execute: typedExecute<Static<typeof supiFlowStartParams>>(
      async (_toolCallId, params, signal) => executeFlowStart(params, signal),
    ),
  },

  // ── supi_flow_plan ─────────────────────────────────────────
  {
    name: "supi_flow_plan",
    label: "Flow Plan",
    description:
      "Store the approved overview in content.md and move the ticket to flow:planned.",
    promptSnippet: "Store the approved overview for a flow ticket",
    promptGuidelines: [
      "Use supi_flow_plan to persist the approved overview; use supi_flow_task to author tasks separately.",
    ],
    executionMode: "sequential" as const,
    parameters: supiFlowPlanParams,
    execute: typedExecute<Static<typeof supiFlowPlanParams>>(
      async (_toolCallId, params, signal) => executeFlowPlan(params, signal),
    ),
  },

  // ── supi_flow_apply ────────────────────────────────────────
  {
    name: "supi_flow_apply",
    label: "Flow Apply",
    description:
      "Enter or resume apply for a planned ticket, loading its overview and task manifest.",
    promptSnippet: "Enter apply for an approved flow ticket",
    promptGuidelines: [
      "Use supi_flow_apply before implementation on a planned flow ticket.",
    ],
    executionMode: "sequential" as const,
    parameters: supiFlowApplyParams,
    execute: typedExecute<Static<typeof supiFlowApplyParams>>(
      async (_toolCallId, params, signal) => executeFlowApply(params, signal),
    ),
  },

  // ── supi_flow_task ─────────────────────────────────────────
  {
    name: "supi_flow_task",
    label: "Flow Task",
    description: "Add, edit, or remove one structured task in a flow ticket.",
    promptSnippet: "Manage one structured task in a flow ticket",
    promptGuidelines: ["Use supi_flow_task as the normal path to author flow tasks."],
    executionMode: "sequential" as const,
    parameters: supiFlowTaskParams,
    execute: typedExecute<Static<typeof supiFlowTaskParams>>(
      async (_toolCallId, params, signal) => executeFlowTask(params, signal),
    ),
  },

  // ── supi_flow_complete_task ────────────────────────────────
  {
    name: "supi_flow_complete_task",
    label: "Flow Complete Task",
    description: "Mark a verified task done by its 1-based task number.",
    promptSnippet: "Mark one verified task done in a flow ticket",
    promptGuidelines: [
      "Use supi_flow_complete_task after verification passes and pass the task number.",
    ],
    executionMode: "sequential" as const,
    parameters: supiFlowCompleteTaskParams,
    execute: typedExecute<Static<typeof supiFlowCompleteTaskParams>>(
      async (_toolCallId, params, signal) => executeFlowCompleteTask(params, signal),
    ),
  },

  // ── supi_flow_close ────────────────────────────────────────
  {
    name: "supi_flow_close",
    label: "Flow Close",
    description:
      "Close a completed flow ticket and write verification evidence to archive.md.",
    promptSnippet: "Close a completed flow ticket with evidence",
    promptGuidelines: [
      "Use supi_flow_close for final closeout and pass full verification evidence.",
    ],
    executionMode: "sequential" as const,
    parameters: supiFlowCloseParams,
    execute: typedExecute<Static<typeof supiFlowCloseParams>>(
      async (_toolCallId, params, signal) => executeFlowClose(params, signal),
    ),
  },
];
