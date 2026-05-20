import { dirname } from "node:path";
import { writeFileSync } from "node:fs";
import { type Static, Type } from "typebox";
import { StringEnum } from "@earendil-works/pi-ai";
import { tndm, tndmJson } from "../cli.js";

// ─── supi_flow_start ───────────────────────────────────────────

export const supiFlowStartParams = Type.Object({
  title: Type.String({ description: "Ticket title describing the change" }),
  priority: Type.Optional(
    StringEnum(["p0", "p1", "p2", "p3", "p4"] as const, {
      description: "Priority",
      default: "p2",
    }),
  ),
  type: Type.Optional(
    StringEnum(["task", "bug", "feature", "chore", "epic"] as const, {
      description: "Ticket type",
      default: "task",
    }),
  ),
  context: Type.Optional(
    Type.String({
      description: "Brief context to store in ticket content (brainstorm intent / design summary)",
    }),
  ),
});

export type FlowStartParams = Static<typeof supiFlowStartParams>;

export async function executeFlowStart(params: FlowStartParams) {
  const args: string[] = [
    "ticket",
    "create",
    params.title,
    "--status",
    "todo",
    "--tags",
    "flow:brainstorm",
  ];

  if (params.priority) args.push("--priority", params.priority);
  if (params.type) args.push("--type", params.type);

  // Use --json on create to get id + content_path in one call
  const createResult = await tndmJson<{ id: string; content_path?: string }>(args);
  const ticketId = createResult.id;
  const contentPath = createResult.content_path ?? "";
  const ticketDir = contentPath ? dirname(contentPath) : "";
  const pathInfo = ticketDir ? ` at ${ticketDir}` : "";

  if (params.context) {
    await tndm(["ticket", "update", ticketId, "--content", params.context]);
  }

  return {
    content: [
      {
        type: "text" as const,
        text: `Created ticket ${ticketId}${pathInfo} with status=todo and flow:brainstorm tag.`,
      },
    ],
    details: {
      action: "flow_start",
      ticketId,
      ticketPath: ticketDir,
      status: "todo",
      tags: "flow:brainstorm",
    },
  };
}

// ─── supi_flow_plan ────────────────────────────────────────────

export const supiFlowPlanParams = Type.Object({
  ticket_id: Type.String({ description: "Ticket ID (e.g. TNDM-A1B2C3)" }),
  plan_content: Type.String({
    description:
      "Approved overview / plan markdown to store in the ticket's canonical content.md. This may contain zero tasks; task authoring happens separately in state.toml.",
  }),
});

export type FlowPlanParams = Static<typeof supiFlowPlanParams>;

export async function executeFlowPlan(params: FlowPlanParams) {
  if (!params.plan_content.trim()) {
    throw new Error("supi_flow_plan: plan_content must not be blank");
  }

  await tndm([
    "ticket",
    "update",
    params.ticket_id,
    "--content",
    params.plan_content,
  ]);

  await tndm([
    "ticket",
    "update",
    params.ticket_id,
    "--remove-tags",
    "flow:brainstorm,flow:planned,flow:applying,flow:done",
    "--add-tags",
    "flow:planned",
  ]);

  return {
    content: [
      {
        type: "text" as const,
        text: `Overview stored in content.md for ticket ${params.ticket_id}. Tags updated to flow:planned.`,
      },
    ],
    details: {
      action: "flow_plan",
      ticketId: params.ticket_id,
      tags: "flow:planned",
      contentStored: true,
    },
  };
}

// ─── supi_flow_complete_task ───────────────────────────────────

export const supiFlowCompleteTaskParams = Type.Object({
  ticket_id: Type.String({ description: "Ticket ID (e.g. TNDM-A1B2C3)" }),
  task_number: Type.Number({
    description: "1-based task number to mark as complete (e.g. 1, 2, 3)",
  }),
});

export type FlowCompleteTaskParams = Static<typeof supiFlowCompleteTaskParams>;

export async function executeFlowCompleteTask(params: FlowCompleteTaskParams) {
  try {
    const result = await tndmJson<Record<string, unknown>>([
      "ticket",
      "task",
      "complete",
      params.ticket_id,
      String(params.task_number),
    ]);

    return {
      content: [
        {
          type: "text" as const,
          text: `Task ${params.task_number} completed in ${params.ticket_id}.`,
        },
      ],
      details: {
        action: "flow_complete_task",
        ticketId: params.ticket_id,
        taskNumber: params.task_number,
        completed: true,
        result,
      },
    };
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    // Task already done returns a regular success from the CLI; only hard failure
    // happens when the task doesn't exist
    if (message.includes("not found")) {
      throw new Error(
        `Task ${params.task_number} not found in ticket ${params.ticket_id}.`,
      );
    }
    throw error;
  }
}

// ─── supi_flow_close ───────────────────────────────────────────

export const supiFlowCloseParams = Type.Object({
  ticket_id: Type.String({ description: "Ticket ID (e.g. TNDM-A1B2C3)" }),
  verification_results: Type.Optional(
    Type.String({
      description:
        "Verification results / evidence from the agent. Appended to the ticket content under ## Verification Results.",
    }),
  ),
});

export type FlowCloseParams = Static<typeof supiFlowCloseParams>;

export async function executeFlowClose(params: FlowCloseParams) {
  let archivePath = "";

  if (params.verification_results) {
    // Create/register archive.md via document registry, then write results
    const docResult = await tndmJson<{ path: string }>([
      "ticket",
      "doc",
      "create",
      params.ticket_id,
      "archive",
    ]);
    archivePath = docResult.path;
    writeFileSync(archivePath, `# Archive\n\n${params.verification_results}\n`, "utf-8");
    await tndm(["ticket", "sync", params.ticket_id]);
  }

  // Replace any flow-state tag with flow:done — remove all possible flow-state tags,
  // set status, and add flow:done in one atomic call, to work correctly regardless of
  // the ticket's current flow state.
  await tndm([
    "ticket",
    "update",
    params.ticket_id,
    "--remove-tags",
    "flow:brainstorm,flow:planned,flow:applying,flow:done",
    "--status",
    "done",
    "--add-tags",
    "flow:done",
  ]);

  return {
    content: [
      {
        type: "text" as const,
        text: `Ticket ${params.ticket_id} closed (status=done, flow:done).`,
      },
    ],
    details: {
      action: "flow_close",
      ticketId: params.ticket_id,
      status: "done",
      tags: "flow:done",
    },
  };
}
