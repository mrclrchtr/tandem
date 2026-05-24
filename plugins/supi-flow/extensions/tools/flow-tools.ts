import { writeArchiveDoc, writeTaskDetailDoc } from "./doc-writes.js";
import { dirname } from "node:path";
import { type Static, Type } from "typebox";
import { StringEnum } from "@earendil-works/pi-ai";
import { tndm, tndmJson } from "../cli.js";
import {
  ensureTaskDetailDoc,
  extractContentPath,
  extractLatestTaskNumber,
  extractTaskTitle,
  extractTicketStatus,
  extractTicketTags,
  filterFlowTasks,
  findRepoRoot,
  type FlowTaskListEntry,
  loadTicket,
  readRequiredTicketContent,
  resolveTicketPath,
  unwrapTicket,
} from "./ticket-helpers.js";

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
      description: "Brief design summary to store in ticket content (brainstorm intent)",
    }),
  ),
});

export type FlowStartParams = Static<typeof supiFlowStartParams>;

export async function executeFlowStart(params: FlowStartParams, signal?: AbortSignal) {
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
  const createResult = await tndmJson<{ id: string; content_path?: string }>(args, signal);
  const ticketId = createResult.id;
  const contentPath = createResult.content_path ?? "";
  const ticketDir = contentPath ? dirname(contentPath) : "";
  const pathInfo = ticketDir ? ` at ${ticketDir}` : "";

  if (params.context) {
    await tndm(["ticket", "update", ticketId, "--content", params.context], signal);
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
      "Approved overview markdown to store in content.md. Task authoring happens separately via supi_flow_task.",
  }),
});

export type FlowPlanParams = Static<typeof supiFlowPlanParams>;

export async function executeFlowPlan(params: FlowPlanParams, signal?: AbortSignal) {
  if (!params.plan_content.trim()) {
    throw new Error("supi_flow_plan: plan_content must not be blank");
  }

  await tndm([
    "ticket",
    "update",
    params.ticket_id,
    "--content",
    params.plan_content,
  ], signal);

  await tndm([
    "ticket",
    "update",
    params.ticket_id,
    "--remove-tags",
    "flow:brainstorm,flow:planned,flow:applying,flow:done",
    "--add-tags",
    "flow:planned",
  ], signal);

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

// ─── supi_flow_apply ───────────────────────────────────────────

export const supiFlowApplyParams = Type.Object({
  ticket_id: Type.String({ description: "Ticket ID (e.g. TNDM-A1B2C3)" }),
});

export type FlowApplyParams = Static<typeof supiFlowApplyParams>;

export async function executeFlowApply(params: FlowApplyParams, signal?: AbortSignal) {
  const ticket = await loadTicket(params.ticket_id, signal);
  const status = extractTicketStatus(ticket);
  const tags = extractTicketTags(ticket);

  if (status === "done" || tags.includes("flow:done")) {
    throw new Error(`supi_flow_apply: ticket ${params.ticket_id} is already closed`);
  }

  if (!tags.includes("flow:planned") && !tags.includes("flow:applying")) {
    throw new Error(
      `supi_flow_apply: ticket ${params.ticket_id} must be in flow:planned or flow:applying`,
    );
  }
  if (
    tags.includes("flow:applying") &&
    status !== "in_progress" &&
    status !== "blocked"
  ) {
    throw new Error(
      `supi_flow_apply: ticket ${params.ticket_id} must have status in_progress or blocked when tagged flow:applying`,
    );
  }

  const overview = await readRequiredTicketContent(
    params.ticket_id,
    extractContentPath(ticket),
    "supi_flow_apply",
  );
  const tasks = await loadTaskList(params.ticket_id, signal);

  if (tasks.length === 0) {
    throw new Error(`supi_flow_apply: ticket ${params.ticket_id} has no structured tasks`);
  }

  let transitioned = false;
  let applyStatus = status ?? "in_progress";

  if (tags.includes("flow:planned")) {
    await tndm([
      "ticket",
      "update",
      params.ticket_id,
      "--status",
      "in_progress",
      "--remove-tags",
      "flow:planned",
      "--add-tags",
      "flow:applying",
    ], signal);
    transitioned = true;
    applyStatus = "in_progress";
  }

  const contentPath = extractContentPath(ticket) ?? "";
  const taskCount = tasks.length;
  const transitionText = transitioned
    ? `Ticket ${params.ticket_id} moved to flow:applying.`
    : applyStatus === "blocked"
      ? `Ticket ${params.ticket_id} is already in flow:applying and currently blocked.`
      : `Ticket ${params.ticket_id} is already in flow:applying.`;

  return {
    content: [
      {
        type: "text" as const,
        text: `${transitionText} Loaded approved overview and ${taskCount} task${taskCount === 1 ? "" : "s"}.`,
      },
    ],
    details: {
      action: "flow_apply",
      ticketId: params.ticket_id,
      transitioned,
      status: applyStatus,
      tags: "flow:applying",
      contentPath,
      overview,
      tasks,
    },
  };
}

// ─── supi_flow_task ────────────────────────────────────────────

export const supiFlowTaskParams = Type.Object({
  ticket_id: Type.String({ description: "Ticket ID (e.g. TNDM-A1B2C3)" }),
  operation: StringEnum(["add", "edit", "remove"] as const, {
    description: "Single-task mutation to apply",
  }),
  task_number: Type.Optional(
    Type.Number({ description: "Task number for edit/remove operations" }),
  ),
  title: Type.Optional(
    Type.String({ description: "Task title (required for add)" }),
  ),

  detail: Type.Optional(
    Type.String({ description: "Optional markdown body for the canonical task detail doc" }),
  ),
});

export type FlowTaskParams = Static<typeof supiFlowTaskParams>;

export async function executeFlowTask(params: FlowTaskParams, signal?: AbortSignal) {


  switch (params.operation) {
    case "add": {
      if (params.task_number !== undefined) {
        throw new Error("supi_flow_task: task_number is not used for add");
      }
      if (!params.title || !params.title.trim()) {
        throw new Error("supi_flow_task: title is required for add");
      }

      const args: string[] = ["ticket", "task", "add", params.ticket_id, "--title", params.title];

      const result = await tndmJson<Record<string, unknown>>(args, signal);
      const taskNumber = extractLatestTaskNumber(result);
      let finalResult = result;

      if (params.detail !== undefined) {
        const detailResult = await ensureTaskDetailDoc(params.ticket_id, taskNumber, signal);
        await writeTaskDetailDoc(detailResult.path, taskNumber, params.title, params.detail);
        await tndm(["ticket", "sync", params.ticket_id], signal);
        finalResult = await loadTicket(params.ticket_id, signal);
      }

      return {
        content: [
          {
            type: "text" as const,
            text: `Task ${taskNumber} added to ${params.ticket_id}.`,
          },
        ],
        details: {
          action: "flow_task",
          operation: "add",
          ticketId: params.ticket_id,
          taskNumber,
          result: finalResult,
        },
      };
    }

    case "edit": {
      if (params.task_number === undefined) {
        throw new Error("supi_flow_task: task_number is required for edit");
      }
      if (params.title !== undefined && !params.title.trim()) {
        throw new Error("supi_flow_task: title must not be blank when provided");
      }
      const hasRequestedChange =
        params.title !== undefined ||
        params.detail !== undefined;
      if (!hasRequestedChange) {
        throw new Error("supi_flow_task: edit requires at least one field change");
      }

      const args: string[] = ["ticket", "task", "edit", params.ticket_id, String(params.task_number)];
      if (params.title !== undefined) args.push("--title", params.title);

      const hasManifestFieldChanges = args.length > 5;
      let finalResult: Record<string, unknown> | undefined;

      if (params.detail !== undefined) {
        const detailResult = await ensureTaskDetailDoc(params.ticket_id, params.task_number, signal);
        const taskSnapshot = hasManifestFieldChanges
          ? await tndmJson<Record<string, unknown>>(args, signal)
          : await loadTicket(params.ticket_id, signal);
        const taskTitle =
          params.title ??
          extractTaskTitle(taskSnapshot, params.task_number) ??
          `Task ${params.task_number}`;
        await writeTaskDetailDoc(detailResult.path, params.task_number, taskTitle, params.detail);
        await tndm(["ticket", "sync", params.ticket_id], signal);
        finalResult = await loadTicket(params.ticket_id, signal);
      } else if (hasManifestFieldChanges) {
        finalResult = await tndmJson<Record<string, unknown>>(args);
      } else {
        finalResult = await tndmJson<Record<string, unknown>>(args);
      }

      return {
        content: [
          {
            type: "text" as const,
            text: `Task ${params.task_number} updated in ${params.ticket_id}.`,
          },
        ],
        details: {
          action: "flow_task",
          operation: "edit",
          ticketId: params.ticket_id,
          taskNumber: params.task_number,
          result: finalResult,
        },
      };
    }

    case "remove": {
      if (params.task_number === undefined) {
        throw new Error("supi_flow_task: task_number is required for remove");
      }

      const result = await tndmJson<Record<string, unknown>>(
        ["ticket", "task", "remove", params.ticket_id, String(params.task_number)],
        signal,
      );

      return {
        content: [
          {
            type: "text" as const,
            text: `Task ${params.task_number} removed from ${params.ticket_id}.`,
          },
        ],
        details: {
          action: "flow_task",
          operation: "remove",
          ticketId: params.ticket_id,
          taskNumber: params.task_number,
          removed: true,
          result,
        },
      };
    }
  }
}

// ─── supi_flow_complete_task ───────────────────────────────────

export const supiFlowCompleteTaskParams = Type.Object({
  ticket_id: Type.String({ description: "Ticket ID (e.g. TNDM-A1B2C3)" }),
  task_number: Type.Number({
    description: "1-based task number to mark as complete (e.g. 1, 2, 3)",
  }),
});

export type FlowCompleteTaskParams = Static<typeof supiFlowCompleteTaskParams>;

export async function executeFlowCompleteTask(params: FlowCompleteTaskParams, signal?: AbortSignal) {
  try {
    const result = await tndmJson<Record<string, unknown>>(
      ["ticket", "task", "complete", params.ticket_id, String(params.task_number)],
      signal,
    );

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
    if (/\btask\s+\d+\s+not\s+found/i.test(message)) {
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
  verification_results: Type.String({
    description:
      "Verification evidence to write into archive.md before closing the ticket.",
  }),
});

export type FlowCloseParams = Static<typeof supiFlowCloseParams>;

export async function executeFlowClose(params: FlowCloseParams, signal?: AbortSignal) {
  const verificationResults = params.verification_results?.trim() ?? "";
  if (!verificationResults) {
    throw new Error("supi_flow_close: verification_results is required");
  }

  const ticket = await loadTicket(params.ticket_id, signal);
  const status = extractTicketStatus(ticket);
  const tags = extractTicketTags(ticket);

  if (status === "done" || tags.includes("flow:done")) {
    throw new Error(`supi_flow_close: ticket ${params.ticket_id} is already closed`);
  }
  if (!tags.includes("flow:applying")) {
    throw new Error(
      `supi_flow_close: ticket ${params.ticket_id} must be in flow:applying before close`,
    );
  }
  if (status !== "in_progress" && status !== "blocked") {
    throw new Error(
      `supi_flow_close: ticket ${params.ticket_id} must have status in_progress or blocked before close`,
    );
  }

  const tasks = await loadTaskList(params.ticket_id, signal);
  if (tasks.length === 0) {
    throw new Error(`supi_flow_close: ticket ${params.ticket_id} has no structured tasks`);
  }

  const incompleteTasks = tasks.filter((task) => task.status !== "done");
  if (incompleteTasks.length > 0) {
    const taskList = incompleteTasks
      .map((task) => `#${task.number ?? "?"}${task.title ? ` ${task.title}` : ""}`)
      .join(", ");
    throw new Error(
      `supi_flow_close: ticket ${params.ticket_id} has incomplete tasks: ${taskList}`,
    );
  }

  // Create/register archive.md via document registry, then write results
  const docResult = await tndmJson<{ path: string }>(
    ["ticket", "doc", "create", params.ticket_id, "archive"],
    signal,
  );
  await writeArchiveDoc(docResult.path, verificationResults);
  await tndm(["ticket", "sync", params.ticket_id]);

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
  ], signal);

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

async function loadTaskList(id: string, signal?: AbortSignal): Promise<FlowTaskListEntry[]> {
  const tasks = await tndmJson<unknown>(["ticket", "task", "list", id], signal);
  if (!Array.isArray(tasks)) {
    throw new Error(`supi_flow: task list for ticket ${id} did not return an array`);
  }
  return filterFlowTasks(tasks);
}
