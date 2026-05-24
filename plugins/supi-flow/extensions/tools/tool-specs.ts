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
 * Ordered definitions of the seven public supi-flow tools.
 * Each entry is shaped to match pi.registerTool() expectations.
 */
export const toolSpecs = [
  // ── supi_tndm_cli ──────────────────────────────────────────
  {
    name: "supi_tndm_cli",
    label: "TNDM CLI",
    description:
      "Execute tndm ticket operations. Action determines which params apply; see parameter descriptions for required fields.",
    promptSnippet: "Execute tndm ticket operations",
    promptGuidelines: [
      "Use supi_tndm_cli for direct tndm operations instead of running tndm via bash",
    ],
    executionMode: "sequential" as const,
    parameters: supi_tndm_cli_params,
    async execute(
      _toolCallId: string,
      params: Record<string, unknown>,
      _signal?: AbortSignal,
    ) {
      return executeTndmCli(params as never, _signal);
    },
  },

  // ── supi_flow_start ────────────────────────────────────────
  {
    name: "supi_flow_start",
    label: "Flow Start",
    description:
      "Start a new flow: creates a TNDM ticket with status=todo and tag=flow:brainstorm. " +
      "Stores known design context in content.md and returns the ticket ID.",
    promptSnippet: "Begin a new flow by creating a TNDM ticket",
    promptGuidelines: [
      "Use supi_flow_start when a brainstorm becomes non-trivial and needs a durable ticket",
      "Always include context (design intent/summary) when known",
    ],
    executionMode: "sequential" as const,
    parameters: supiFlowStartParams,
    async execute(
      _toolCallId: string,
      params: Record<string, unknown>,
      _signal?: AbortSignal,
    ) {
      return executeFlowStart(params as never, _signal);
    },
  },

  // ── supi_flow_plan ─────────────────────────────────────────
  {
    name: "supi_flow_plan",
    label: "Flow Plan",
    description:
      "Store the approved overview in the ticket's content.md. " +
      "Updates tags from flow:brainstorm to flow:planned. Task authoring happens separately in state.toml.",
    promptSnippet: "Store a plan in a TNDM ticket",
    promptGuidelines: [
      "Use supi_flow_plan after creating a plan to persist the approved overview in content.md",
      "Use supi_flow_task to author tasks after supi_flow_plan; supi_flow_plan only stores the overview, not tasks",
    ],
    executionMode: "sequential" as const,
    parameters: supiFlowPlanParams,
    async execute(
      _toolCallId: string,
      params: Record<string, unknown>,
      _signal?: AbortSignal,
    ) {
      return executeFlowPlan(params as never, _signal);
    },
  },

  // ── supi_flow_apply ────────────────────────────────────────
  {
    name: "supi_flow_apply",
    label: "Flow Apply",
    description:
      "Start the apply phase for a planned ticket. " +
      "Loads the approved overview and task manifest, transitions flow:planned tickets to flow:applying, and preserves the current status for already-applying tickets.",
    promptSnippet: "Start the apply phase for a TNDM flow ticket",
    promptGuidelines: [
      "Use supi_flow_apply at the beginning of implementation to load the approved overview and task manifest",
      "Review the full overview and task list up front; read task detail docs only when that task becomes active",
    ],
    executionMode: "sequential" as const,
    parameters: supiFlowApplyParams,
    async execute(
      _toolCallId: string,
      params: Record<string, unknown>,
      _signal?: AbortSignal,
    ) {
      return executeFlowApply(params as never, _signal);
    },
  },

  // ── supi_flow_task ─────────────────────────────────────────
  {
    name: "supi_flow_task",
    label: "Flow Task",
    description:
      "Manage one structured task in a flow ticket. " +
      "Operation determines which params apply: add requires title; edit/remove require task_number; optional detail writes or clears the canonical task detail doc.",
    promptSnippet: "Manage one task in a TNDM flow ticket",
    promptGuidelines: [
      "Use supi_flow_task for the common plan-time path to add, edit, or remove one structured task at a time",
      "Prefer supi_flow_task over raw task_json or detail_path handling when authoring normal flow tasks",
    ],
    executionMode: "sequential" as const,
    parameters: supiFlowTaskParams,
    async execute(
      _toolCallId: string,
      params: Record<string, unknown>,
      _signal?: AbortSignal,
    ) {
      return executeFlowTask(params as never, _signal);
    },
  },

  // ── supi_flow_complete_task ────────────────────────────────
  {
    name: "supi_flow_complete_task",
    label: "Flow Complete Task",
    description:
      "Mark a task as done in a ticket by task number (1-based). " +
      "Calls 'tndm ticket task complete' to update the structured task in state.toml.",
    promptSnippet: "Check off a completed plan task in a TNDM ticket",
    promptGuidelines: [
      "Use supi_flow_complete_task after each task's verification passes during apply",
      "Pass the task number, not the description text",
    ],
    executionMode: "sequential" as const,
    parameters: supiFlowCompleteTaskParams,
    async execute(
      _toolCallId: string,
      params: Record<string, unknown>,
      _signal?: AbortSignal,
    ) {
      return executeFlowCompleteTask(params as never, _signal);
    },
  },

  // ── supi_flow_close ────────────────────────────────────────
  {
    name: "supi_flow_close",
    label: "Flow Close",
    description:
      "Close a ticket and finalize the flow. " +
      "Requires flow:applying with a non-empty all-done task list, writes verification results to archive.md, sets status=done, and tags=flow:done.",
    promptSnippet: "Close a TNDM ticket after implementation and verification",
    promptGuidelines: [
      "Use supi_flow_close at the end of the archive phase after all verification is complete",
      "Pass the full verification evidence as verification_results",
    ],
    executionMode: "sequential" as const,
    parameters: supiFlowCloseParams,
    async execute(
      _toolCallId: string,
      params: Record<string, unknown>,
      _signal?: AbortSignal,
    ) {
      return executeFlowClose(params as never, _signal);
    },
  },
];
