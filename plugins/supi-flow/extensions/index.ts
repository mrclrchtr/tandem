import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import type { ExtensionAPI, ToolExecutionMode } from "@earendil-works/pi-coding-agent";

import { tndmVersion } from "./cli.js";
import { supi_tndm_cli_params, executeTndmCli } from "./tools/tndm-cli.js";
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
} from "./tools/flow-tools.js";

const baseDir = dirname(dirname(fileURLToPath(import.meta.url)));
const pkg = JSON.parse(readFileSync(join(baseDir, "package.json"), "utf-8"));
export const FLOW_VERSION: string = pkg.version;

/**
 * Check tndm version against supi-flow version. Notifies on mismatch.
 * Exported for testing.
 */
export async function checkTndmVersion(
  event: { reason: string },
  ctx: { ui: { notify: (message: string, type?: "info" | "warning" | "error") => void } },
): Promise<void> {
  if (event.reason !== "startup" && event.reason !== "reload") return;
  const tndmVer = await tndmVersion();
  if (!tndmVer) return;
  if (tndmVer !== FLOW_VERSION) {
    ctx.ui.notify(
      `tndm v${tndmVer} found, but supi-flow expects v${FLOW_VERSION}. ` +
        `Install matching version: brew install mrclrchtr/tap/tndm`,
      "warning",
    );
  }
}

export default function (pi: ExtensionAPI) {
  // ── Version check on startup ────────────────────────────────
  pi.on("session_start", async (event, ctx) => {
    await checkTndmVersion(event, ctx);
  });

  // ── Tool: supi_tndm_cli ─────────────────────────────────────
  pi.registerTool({
    name: "supi_tndm_cli",
    label: "TNDM CLI",
    description:
      "Execute tndm ticket operations. Action determines which params apply:\n" +
      "- create: title (required), status, priority, type, tags, depends_on, effort, content\n" +
      "- update: id (required), title, status, priority, type, tags, add_tags, remove_tags, depends_on, effort, content\n" +
      "- show: id (required)\n" +
      "- list: all (boolean), definition (ready|questions|unknown)\n" +
      "- awareness: against (git ref, required)\n" +
      "- task_add: id (required), task_title (required), task_files, task_verification, task_notes, task_detail\n" +
      "- task_list: id (required)\n" +
      "- task_complete: id (required), task_number (required)\n" +
      "- task_remove: id (required), task_number (required)\n" +
      "- task_edit: id (required), task_number (required), task_title, task_files, task_clear_files, task_verification, task_notes, task_detail\n" +
      "- task_set: id (required), task_json (required)",
    promptSnippet: "Execute tndm ticket operations via supi_tndm_cli",
    promptGuidelines: [
      "Use supi_tndm_cli for direct tndm operations instead of running tndm via bash",
    ],
    parameters: supi_tndm_cli_params,
    async execute(_toolCallId, params, _signal, _onUpdate, _ctx) {
      return executeTndmCli(params);
    },
  });

  // ── Tool: supi_flow_start ───────────────────────────────────
  pi.registerTool({
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
    executionMode: "sequential" as ToolExecutionMode,
    parameters: supiFlowStartParams,
    async execute(_toolCallId, params, _signal, _onUpdate, _ctx) {
      return executeFlowStart(params);
    },
  });

  // ── Tool: supi_flow_plan ────────────────────────────────────
  pi.registerTool({
    name: "supi_flow_plan",
    label: "Flow Plan",
    description:
      "Store the approved overview / plan in the ticket's canonical content.md. " +
      "Updates tags from flow:brainstorm to flow:planned. Task authoring happens separately in state.toml.",
    promptSnippet: "Store a plan in a TNDM ticket",
    promptGuidelines: [
      "Use supi_flow_plan after creating a plan to persist the approved overview in content.md",
      "Create execution tasks separately after the overview exists; do not rely on supi_flow_plan to parse task blocks into state.toml",
    ],
    executionMode: "sequential" as ToolExecutionMode,
    parameters: supiFlowPlanParams,
    async execute(_toolCallId, params, _signal, _onUpdate, _ctx) {
      return executeFlowPlan(params);
    },
  });

  // ── Tool: supi_flow_apply ───────────────────────────────────
  pi.registerTool({
    name: "supi_flow_apply",
    label: "Flow Apply",
    description:
      "Start the apply phase for a planned ticket. " +
      "Loads the approved content.md overview, returns the structured task manifest, transitions flow:planned tickets to status=in_progress with flow:applying, and preserves the current in_progress/blocked status for already-applying tickets.",
    promptSnippet: "Start the apply phase for a TNDM flow ticket",
    promptGuidelines: [
      "Use supi_flow_apply at the beginning of implementation to load the approved overview and task manifest, and to move a planned ticket into flow:applying when needed.",
    ],
    executionMode: "sequential" as ToolExecutionMode,
    parameters: supiFlowApplyParams,
    async execute(_toolCallId, params, _signal, _onUpdate, _ctx) {
      return executeFlowApply(params);
    },
  });

  // ── Tool: supi_flow_task ────────────────────────────────────
  pi.registerTool({
    name: "supi_flow_task",
    label: "Flow Task",
    description:
      "Manage one structured task in a flow ticket. " +
      "Operation determines which params apply: add requires title; edit/remove require task_number; optional detail writes or clears the canonical task detail doc.",
    promptSnippet: "Manage one task at a time in a TNDM flow ticket",
    promptGuidelines: [
      "Use supi_flow_task for the common plan-time path to add, edit, or remove one structured task at a time",
      "Prefer supi_flow_task over raw task_json or detail_path handling when authoring normal flow tasks",
    ],
    executionMode: "sequential" as ToolExecutionMode,
    parameters: supiFlowTaskParams,
    async execute(_toolCallId, params, _signal, _onUpdate, _ctx) {
      return executeFlowTask(params);
    },
  });

  // ── Tool: supi_flow_complete_task ───────────────────────────
  pi.registerTool({
    name: "supi_flow_complete_task",
    label: "Flow Complete Task",
    description:
      "Mark a task as done in a ticket by task number (1-based). " +
      "Calls 'tndm ticket task complete' to update the structured task in state.toml.",
    promptSnippet: "Check off a completed plan task in a TNDM ticket",
    promptGuidelines: [
      "Use supi_flow_complete_task after each task's verification passes during apply",
      "Call this with the task number, not the description text",
    ],
    executionMode: "sequential" as ToolExecutionMode,
    parameters: supiFlowCompleteTaskParams,
    async execute(_toolCallId, params, _signal, _onUpdate, _ctx) {
      return executeFlowCompleteTask(params);
    },
  });

  // ── Tool: supi_flow_close ───────────────────────────────────
  pi.registerTool({
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
    executionMode: "sequential" as ToolExecutionMode,
    parameters: supiFlowCloseParams,
    async execute(_toolCallId, params, _signal, _onUpdate, _ctx) {
      return executeFlowClose(params);
    },
  });

}
