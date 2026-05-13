import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";

import { tndmVersion } from "./cli.js";
import { supi_tndm_cli_params, executeTndmCli } from "./tools/tndm-cli.js";
import {
  supiFlowStartParams,
  supiFlowPlanParams,
  supiFlowCompleteTaskParams,
  supiFlowCloseParams,
  executeFlowStart,
  executeFlowPlan,
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
      "- awareness: against (git ref, required)",
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
      "Use supi_flow_start at the beginning of every brainstorm to create the required ticket",
      "Always include context (design intent/summary) when known",
    ],
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
      "Store an implementation plan in a ticket's plan.md while keeping content.md as the canonical design summary. " +
      "Updates tags from flow:brainstorm to flow:planned. Tasks must be numbered as '**Task {N}**' in the plan.",
    promptSnippet: "Store a plan in a TNDM ticket",
    promptGuidelines: [
      "Use supi_flow_plan after creating a plan to persist it in the ticket",
      "Number tasks sequentially as **Task 1**, **Task 2**, etc.",
    ],
    parameters: supiFlowPlanParams,
    async execute(_toolCallId, params, _signal, _onUpdate, _ctx) {
      return executeFlowPlan(params);
    },
  });

  // ── Tool: supi_flow_complete_task ───────────────────────────
  pi.registerTool({
    name: "supi_flow_complete_task",
    label: "Flow Complete Task",
    description:
      "Mark a task as done in a ticket's plan.md by task number (1-based). " +
      "Finds '- [ ] **Task N:**' and changes to '- [x] **Task N:**'.",
    promptSnippet: "Check off a completed plan task in a TNDM ticket",
    promptGuidelines: [
      "Use supi_flow_complete_task after each task's verification passes during apply",
      "Call this with the task number, not the description text",
    ],
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
      "Writes verification results to archive.md, sets status=done, and tags=flow:done.",
    promptSnippet: "Close a TNDM ticket after implementation and verification",
    promptGuidelines: [
      "Use supi_flow_close at the end of the archive phase after all verification is complete",
      "Pass the full verification evidence as verification_results",
    ],
    parameters: supiFlowCloseParams,
    async execute(_toolCallId, params, _signal, _onUpdate, _ctx) {
      return executeFlowClose(params);
    },
  });

}
