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
      "Direct wrapper for tndm ticket/task operations; use instead of bash. " +
      "Prefer supi_flow_task for normal task authoring; use task_* only for repair. " +
      "show/list/awareness/task output truncates at 2000 lines/50KB; full payload in details.",
    promptSnippet: "Run direct tndm ticket/task operations",
    promptGuidelines: [],
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
      "Create a ticket for non-trivial work (status=todo, tag flow:brainstorm). Writes context to content.md when given. Returns ticket id.",
    promptSnippet: "Create a brainstorm ticket for non-trivial work",
    promptGuidelines: [
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
      "Store the approved overview in content.md and set tag flow:planned. plan_content must be non-blank. Author tasks afterwards with supi_flow_task.",
    promptSnippet: "Store the approved overview for a flow ticket",
    promptGuidelines: [],
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
      "Use when entering the apply phase. Load the approved overview and structured task manifest for a planned ticket. Moves planned tickets into applying. Returns overview text and task list.",
    promptSnippet: "Enter apply for an approved flow ticket",
    promptGuidelines: [],
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
    description:
      "Use when authoring or reconciling tasks in a plan. Add, edit, or remove one structured task in a flow ticket. Writes tasks/task-XX.md detail doc when detail is given. Use edit/remove/add to reconcile existing manifests.",
    promptSnippet: "Manage one structured task in a flow ticket",
    promptGuidelines: [],
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
    description:
      "Mark a verified task done by its 1-based task number. Throws if the task does not exist in the ticket.",
    promptSnippet: "Mark one verified task done in a flow ticket",
    promptGuidelines: [],
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
      "Use at archive closeout. Close a completed flow ticket, write verification evidence to archive.md, and set status=done + flow:done. Requires all tasks complete.",
    promptSnippet: "Close a completed flow ticket with evidence",
    promptGuidelines: [],
    executionMode: "sequential" as const,
    parameters: supiFlowCloseParams,
    execute: typedExecute<Static<typeof supiFlowCloseParams>>(
      async (_toolCallId, params, signal) => executeFlowClose(params, signal),
    ),
  },
];
