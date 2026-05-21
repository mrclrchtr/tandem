import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, isAbsolute, join, resolve } from "node:path";
import { type Static, Type } from "typebox";
import { StringEnum } from "@earendil-works/pi-ai";
import { tndm, tndmJson } from "../cli.js";

type FlowTaskListEntry = {
  number?: number;
  title?: string;
  status?: string;
  files?: string[];
  verification?: string;
  notes?: string;
  detail_path?: string;
};

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

// ─── supi_flow_apply ───────────────────────────────────────────

export const supiFlowApplyParams = Type.Object({
  ticket_id: Type.String({ description: "Ticket ID (e.g. TNDM-A1B2C3)" }),
});

export type FlowApplyParams = Static<typeof supiFlowApplyParams>;

export async function executeFlowApply(params: FlowApplyParams) {
  const ticket = await loadTicket(params.ticket_id);
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

  const overview = readRequiredTicketContent(
    params.ticket_id,
    extractContentPath(ticket),
    "supi_flow_apply",
  );
  const tasks = await loadTaskList(params.ticket_id);

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
    ]);
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
  files: Type.Optional(
    Type.Array(Type.String(), { description: "File paths for the task" }),
  ),
  clear_files: Type.Optional(
    Type.Boolean({ description: "Clear all file paths during edit" }),
  ),
  verification: Type.Optional(
    Type.String({ description: "Verification command for the task" }),
  ),
  clear_verification: Type.Optional(
    Type.Boolean({ description: "Clear the verification command during edit" }),
  ),
  notes: Type.Optional(
    Type.String({ description: "Extra notes for the task" }),
  ),
  clear_notes: Type.Optional(
    Type.Boolean({ description: "Clear task notes during edit" }),
  ),
  detail: Type.Optional(
    Type.String({ description: "Optional markdown body for the canonical task detail doc" }),
  ),
  clear_detail: Type.Optional(
    Type.Boolean({ description: "Clear the linked canonical task detail doc reference during edit" }),
  ),
});

export type FlowTaskParams = Static<typeof supiFlowTaskParams>;

export async function executeFlowTask(params: FlowTaskParams) {
  if (params.files !== undefined && params.clear_files) {
    throw new Error("supi_flow_task: files and clear_files cannot be used together");
  }
  if (params.verification !== undefined && params.clear_verification) {
    throw new Error("supi_flow_task: verification and clear_verification cannot be used together");
  }
  if (params.notes !== undefined && params.clear_notes) {
    throw new Error("supi_flow_task: notes and clear_notes cannot be used together");
  }
  if (params.detail !== undefined && params.clear_detail) {
    throw new Error("supi_flow_task: detail and clear_detail cannot be used together");
  }

  switch (params.operation) {
    case "add": {
      if (params.task_number !== undefined) {
        throw new Error("supi_flow_task: task_number is not used for add");
      }
      if (!params.title || !params.title.trim()) {
        throw new Error("supi_flow_task: title is required for add");
      }

      const args: string[] = [
        "ticket",
        "task",
        "add",
        params.ticket_id,
        "--title",
        params.title,
      ];
      for (const file of params.files ?? []) {
        args.push("--file", file);
      }
      if (params.verification && params.verification.trim()) {
        args.push("--verification", params.verification);
      }
      if (params.notes && params.notes.trim()) {
        args.push("--notes", params.notes);
      }

      const result = await tndmJson<Record<string, unknown>>(args);
      const taskNumber = extractLatestTaskNumber(result);
      let finalResult = result;

      if (params.detail !== undefined) {
        const detailResult = await ensureTaskDetailDoc(params.ticket_id, taskNumber);
        writeTaskDetailDoc(detailResult.path, taskNumber, params.title, params.detail);
        await tndm(["ticket", "sync", params.ticket_id]);
        finalResult = await loadTicket(params.ticket_id);
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
        params.files !== undefined ||
        Boolean(params.clear_files) ||
        params.verification !== undefined ||
        Boolean(params.clear_verification) ||
        params.notes !== undefined ||
        Boolean(params.clear_notes) ||
        params.detail !== undefined ||
        Boolean(params.clear_detail);
      if (!hasRequestedChange) {
        throw new Error("supi_flow_task: edit requires at least one field change");
      }

      const args: string[] = [
        "ticket",
        "task",
        "edit",
        params.ticket_id,
        String(params.task_number),
      ];
      if (params.title !== undefined) args.push("--title", params.title);
      if (params.files !== undefined) {
        if (params.files.length === 0) {
          args.push("--clear-files");
        } else {
          for (const file of params.files) args.push("--file", file);
        }
      } else if (params.clear_files) {
        args.push("--clear-files");
      }
      if (params.verification !== undefined) {
        args.push("--verification", params.verification);
      } else if (params.clear_verification) {
        args.push("--verification", "");
      }
      if (params.notes !== undefined) {
        args.push("--notes", params.notes);
      } else if (params.clear_notes) {
        args.push("--notes", "");
      }

      const hasManifestFieldChanges = args.length > 5;
      const result = hasManifestFieldChanges
        ? await tndmJson<Record<string, unknown>>(args)
        : undefined;
      let finalResult = result;

      if (params.detail !== undefined) {
        const detailResult = await ensureTaskDetailDoc(params.ticket_id, params.task_number);
        const taskSnapshot = result ?? await loadTicket(params.ticket_id);
        const taskTitle =
          params.title ??
          extractTaskTitle(taskSnapshot, params.task_number) ??
          `Task ${params.task_number}`;
        writeTaskDetailDoc(
          detailResult.path,
          params.task_number,
          taskTitle,
          params.detail,
        );
        await tndm(["ticket", "sync", params.ticket_id]);
        finalResult = await loadTicket(params.ticket_id);
      } else if (params.clear_detail) {
        await tndmJson([
          "ticket",
          "task",
          "detail",
          "clear",
          params.ticket_id,
          String(params.task_number),
        ]);
        finalResult = await loadTicket(params.ticket_id);
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

      const result = await tndmJson<Record<string, unknown>>([
        "ticket",
        "task",
        "remove",
        params.ticket_id,
        String(params.task_number),
      ]);

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
  verification_results: Type.String({
    description:
      "Verification results / evidence from the agent to write into archive.md before closing the ticket.",
  }),
});

export type FlowCloseParams = Static<typeof supiFlowCloseParams>;

export async function executeFlowClose(params: FlowCloseParams) {
  const verificationResults = params.verification_results?.trim() ?? "";
  if (!verificationResults) {
    throw new Error("supi_flow_close: verification_results is required");
  }

  const ticket = await loadTicket(params.ticket_id);
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

  const tasks = await loadTaskList(params.ticket_id);
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
  const docResult = await tndmJson<{ path: string }>([
    "ticket",
    "doc",
    "create",
    params.ticket_id,
    "archive",
  ]);
  writeFileSync(docResult.path, `# Archive\n\n${verificationResults}\n`, "utf-8");
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

function extractLatestTaskNumber(result: Record<string, unknown>): number {
  const tasks = extractTasks(result);
  const numbers = tasks
    .map((task) => task.number)
    .filter((value): value is number => typeof value === "number");

  if (numbers.length === 0) {
    throw new Error("supi_flow_task: task_add did not return a task list");
  }

  return Math.max(...numbers);
}

function extractTaskTitle(result: Record<string, unknown>, taskNumber: number): string | undefined {
  return extractTasks(result).find((task) => task.number === taskNumber)?.title;
}

function extractTasks(result: Record<string, unknown>): FlowTaskListEntry[] {
  const ticket = unwrapTicket(result);

  if (Array.isArray(ticket.tasks)) {
    return filterFlowTasks(ticket.tasks);
  }

  const state = ticket.state;
  if (
    typeof state === "object" &&
    state !== null &&
    Array.isArray((state as { tasks?: unknown }).tasks)
  ) {
    return filterFlowTasks((state as { tasks: unknown[] }).tasks);
  }

  return [];
}

function filterFlowTasks(tasks: unknown[]): FlowTaskListEntry[] {
  return tasks.filter(
    (task): task is FlowTaskListEntry => typeof task === "object" && task !== null,
  );
}

function unwrapTicket(result: Record<string, unknown>): Record<string, unknown> {
  const ticket = result.ticket;
  if (typeof ticket === "object" && ticket !== null) {
    return ticket as Record<string, unknown>;
  }
  return result;
}

function extractTicketTags(result: Record<string, unknown>): string[] {
  const ticket = unwrapTicket(result);
  if (!Array.isArray(ticket.tags)) {
    return [];
  }

  return ticket.tags.filter((tag): tag is string => typeof tag === "string");
}

function extractTicketStatus(result: Record<string, unknown>): string | undefined {
  const ticket = unwrapTicket(result);
  return typeof ticket.status === "string" ? ticket.status : undefined;
}

function extractContentPath(result: Record<string, unknown>): string | undefined {
  const ticket = unwrapTicket(result);
  return typeof ticket.content_path === "string" ? ticket.content_path : undefined;
}

function readRequiredTicketContent(
  ticketId: string,
  contentPath: string | undefined,
  toolName: string,
): string {
  if (!contentPath) {
    throw new Error(`${toolName}: ticket ${ticketId} is missing content_path`);
  }

  let overview: string;
  try {
    overview = readFileSync(resolveTicketPath(contentPath), "utf-8");
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    throw new Error(`${toolName}: failed to read content.md for ticket ${ticketId}: ${message}`);
  }

  if (!overview.trim()) {
    throw new Error(`${toolName}: approved overview in content.md must not be blank`);
  }

  return overview;
}

function resolveTicketPath(ticketPath: string): string {
  if (isAbsolute(ticketPath)) {
    return ticketPath;
  }

  return resolve(findRepoRoot(), ticketPath);
}

function findRepoRoot(startDir = process.cwd()): string {
  let current = resolve(startDir);

  while (true) {
    if (existsSync(join(current, ".git")) || existsSync(join(current, ".tndm"))) {
      return current;
    }

    const parent = dirname(current);
    if (parent === current) {
      throw new Error(`failed to locate repository root from ${startDir}`);
    }
    current = parent;
  }
}

async function loadTicket(id: string): Promise<Record<string, unknown>> {
  return tndmJson<Record<string, unknown>>(["ticket", "show", id]);
}

async function loadTaskList(id: string): Promise<FlowTaskListEntry[]> {
  const tasks = await tndmJson<unknown>(["ticket", "task", "list", id]);
  if (!Array.isArray(tasks)) {
    throw new Error(`supi_flow: task list for ticket ${id} did not return an array`);
  }
  return filterFlowTasks(tasks);
}

async function ensureTaskDetailDoc(id: string, taskNumber: number): Promise<{ path: string }> {
  return tndmJson<{ path: string }>([
    "ticket",
    "task",
    "detail",
    "ensure",
    id,
    String(taskNumber),
  ]);
}

function writeTaskDetailDoc(path: string, taskNumber: number, title: string, detail: string): void {
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, `# Task ${taskNumber}: ${title}\n\n${detail}\n`, "utf-8");
}
