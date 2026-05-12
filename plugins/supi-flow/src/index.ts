import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";
import { Type } from "typebox";

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

export default function (pi: ExtensionAPI) {
  // ── Resource discovery ──────────────────────────────────────
  pi.on("resources_discover", () => ({
    skillPaths: [join(baseDir, "skills")],
    promptPaths: [join(baseDir, "prompts")],
  }));

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
      "This is the first step in every flow. Returns the ticket ID.",
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
      "Store an implementation plan in a ticket's plan.md. " +
      "Updates tags from flow:brainstorm to flow:planned. " +
      "Tasks must be numbered as '**Task {N}**' in the plan.",
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
      "Writes verification results to archive.md, sets status=done, tags=flow:done, " +
      "and auto-commits .tndm/ changes.",
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

  // ── Command: /supi-flow-status ──────────────────────────────
  pi.registerCommand("supi-flow-status", {
    description: "Show current flow workflow state",
    handler: async (_args, ctx) => {
      const ids = collectTicketIds(ctx.sessionManager.getBranch());
      if (ids.length === 0) {
        ctx.ui.notify("No active flow. Start with /skill:supi-flow-brainstorm.", "info");
        return;
      }
      ctx.ui.notify(
        `Active tickets: ${ids.join(", ")}. Use /skill:supi-flow-plan <ID> to continue.`,
        "info",
      );
    },
  });

  // ── Command: /supi-flow ─────────────────────────────────────
  pi.registerCommand("supi-flow", {
    description: "List available flow workflow commands",
    handler: async (_args, ctx) => {
      ctx.ui.notify(
        "Flow: /skill:supi-flow-brainstorm -> /skill:supi-flow-plan -> /skill:supi-flow-apply -> /skill:supi-flow-archive\n" +
          "  /supi-flow-status -- show current state\n" +
          "  /supi-flow        -- this help\n" +
          "Available tools: supi_tndm_cli, supi_flow_start, supi_flow_plan, supi_flow_complete_task, supi_flow_close",
        "info",
      );
    },
  });
}

function collectTicketIds(
  entries: Array<{ type: string; message?: { role?: string; content?: unknown } }>,
): string[] {
  const ids = new Set<string>();
  for (const entry of entries) {
    if (entry.type !== "message") continue;
    if (entry.message?.role !== "user") continue;
    const content = entry.message?.content;
    if (typeof content !== "string") continue;
    for (const m of content.matchAll(/TNDM-\w{6}/g)) ids.add(m[0]);
  }
  return Array.from(ids);
}
