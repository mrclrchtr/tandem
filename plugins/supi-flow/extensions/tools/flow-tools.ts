import { writeArchiveDoc } from "./doc-writes.js";
import { dirname } from "node:path";
import { type Static, Type } from "typebox";
import { StringEnum } from "@earendil-works/pi-ai";
import { tndm, tndmJson } from "../cli.js";
import {
  extractContentPath,
  extractLatestTaskNumber,
  extractTaskTitle,
  extractTicketStatus,
  extractTicketTags,
  FLOW_TAGS_ALL,
  FLOW_TAG_APPLYING,
  FLOW_TAG_BRAINSTORM,
  FLOW_TAG_DONE,
  FLOW_TAG_PLANNED,
  type FlowTaskListEntry,
  formatContent,
  loadTaskList,
  loadTicket,
  readRequiredTicketContent,
  applyTaskMutation,
} from "./ticket-helpers.js";

export const supiFlowStartParams = Type.Object({
  title: Type.String(),
  priority: Type.Optional(
    StringEnum(["p0", "p1", "p2", "p3", "p4"] as const, {
      default: "p2",
    }),
  ),
  type: Type.Optional(
    StringEnum(["task", "bug", "feature", "chore", "epic"] as const, {
      default: "task",
    }),
  ),
  context: Type.Optional(
    Type.String({
      description: "Initial context for content.md",
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
    FLOW_TAG_BRAINSTORM,
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
      tags: FLOW_TAG_BRAINSTORM,
    },
  };
}

// ─── supi_flow_plan ────────────────────────────────────────────

export const supiFlowPlanParams = Type.Object({
  ticket_id: Type.String(),
  plan_content: Type.String({
    description: "Approved overview markdown for content.md",
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
    FLOW_TAGS_ALL,
    "--add-tags",
    FLOW_TAG_PLANNED,
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
      tags: FLOW_TAG_PLANNED,
      contentStored: true,
    },
  };
}

// ─── supi_flow_apply ───────────────────────────────────────────

export const supiFlowApplyParams = Type.Object({
  ticket_id: Type.String(),
});

export type FlowApplyParams = Static<typeof supiFlowApplyParams>;

export async function executeFlowApply(params: FlowApplyParams, signal?: AbortSignal) {
  const ticket = await loadTicket(params.ticket_id, signal);
  const status = extractTicketStatus(ticket);
  const tags = extractTicketTags(ticket);

  if (status === "done" || tags.includes(FLOW_TAG_DONE)) {
    throw new Error(`supi_flow_apply: ticket ${params.ticket_id} is already closed`);
  }

  if (!tags.includes(FLOW_TAG_PLANNED) && !tags.includes(FLOW_TAG_APPLYING)) {
    throw new Error(
      `supi_flow_apply: ticket ${params.ticket_id} must be in flow:planned or flow:applying`,
    );
  }
  if (
    tags.includes(FLOW_TAG_APPLYING) &&
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

  if (tags.includes(FLOW_TAG_PLANNED)) {
    await tndm([
      "ticket",
      "update",
      params.ticket_id,
      "--status",
      "in_progress",
      "--remove-tags",
      FLOW_TAG_PLANNED,
      "--add-tags",
      FLOW_TAG_APPLYING,
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

  const taskSummary = tasks
    .map((t) => `  ${t.number ?? "?"}. ${t.detail_path ?? "?"} — ${t.title ?? "(untitled)"}`)
    .join("\n");

  return {
    content: [{ type: "text" as const, text: formatContent(`${transitionText} Loaded approved overview and ${taskCount} task${taskCount === 1 ? "" : "s"}.\n\nTask detail docs (read each before its task):\n${taskSummary}`) }],
    details: {
      action: "flow_apply",
      ticketId: params.ticket_id,
      transitioned,
      status: applyStatus,
      tags: FLOW_TAG_APPLYING,
      contentPath,
      overview,
      tasks,
    },
  };
}

// ─── supi_flow_task ────────────────────────────────────────────

export const supiFlowTaskParams = Type.Object({
  ticket_id: Type.String(),
  operation: StringEnum(["add", "edit", "remove"] as const),
  task_number: Type.Optional(
    Type.Number({ description: "1-based task number for edit/remove" }),
  ),
  title: Type.Optional(Type.String()),

  detail: Type.Optional(
    Type.String({ description: "Task detail markdown" }),
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
        finalResult = await applyTaskMutation(
          params.ticket_id, taskNumber, params.title, params.detail, signal,
        );
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
        const applyTitleEdit = hasManifestFieldChanges;
        let taskTitle: string;
        if (params.title !== undefined) {
          taskTitle = params.title;
        } else {
          const ticket = await loadTicket(params.ticket_id, signal);
          taskTitle = extractTaskTitle(ticket, params.task_number) ?? `Task ${params.task_number}`;
        }
        finalResult = await applyTaskMutation(
          params.ticket_id, params.task_number, taskTitle, params.detail, signal, applyTitleEdit,
        );
      } else {
        finalResult = await tndmJson<Record<string, unknown>>(args, signal);
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
  ticket_id: Type.String(),
  task_number: Type.Number(),
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
  ticket_id: Type.String(),
  verification_results: Type.String({
    description: "Verification evidence for archive.md",
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

  if (status === "done" || tags.includes(FLOW_TAG_DONE)) {
    throw new Error(`supi_flow_close: ticket ${params.ticket_id} is already closed`);
  }
  if (!tags.includes(FLOW_TAG_APPLYING)) {
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
  await tndm(["ticket", "sync", params.ticket_id], signal);

  // Replace any flow-state tag with flow:done — remove all possible flow-state tags,
  // set status, and add flow:done in one atomic call, to work correctly regardless of
  // the ticket's current flow state.
  await tndm([
    "ticket",
    "update",
    params.ticket_id,
    "--remove-tags",
    FLOW_TAGS_ALL,
    "--status",
    "done",
    "--add-tags",
    FLOW_TAG_DONE,
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
      tags: FLOW_TAG_DONE,
    },
  };
}
