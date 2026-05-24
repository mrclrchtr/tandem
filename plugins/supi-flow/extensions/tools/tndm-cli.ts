import { mkdirSync, writeFileSync } from "node:fs";
import { dirname } from "node:path";
import { type Static, Type } from "typebox";
import { StringEnum } from "@earendil-works/pi-ai";
import { tndm, tndmJson } from "../cli.js";

export const actionEnum = StringEnum([
  "create",
  "update",
  "show",
  "list",
  "awareness",
  "task_add",
  "task_list",
  "task_complete",
  "task_remove",
  "task_edit",
  "task_set",
] as const);

export const supi_tndm_cli_params = Type.Object({
  action: actionEnum,
  // Common identifiers
  id: Type.Optional(Type.String({ description: "Ticket ID (required for update/show)" })),

  // Create / Update params
  title: Type.Optional(Type.String({ description: "Ticket title (required for create)" })),
  status: Type.Optional(
    StringEnum(["todo", "in_progress", "blocked", "done"] as const, {
      description: "Ticket status",
    }),
  ),
  priority: Type.Optional(
    StringEnum(["p0", "p1", "p2", "p3", "p4"] as const, {
      description: "Priority (p0=critical)",
    }),
  ),
  type: Type.Optional(
    StringEnum(["task", "bug", "feature", "chore", "epic"] as const, {
      description: "Ticket type",
    }),
  ),
  tags: Type.Optional(
    Type.String({
      description: "Comma-separated tags (replaces existing list; e.g. 'auth,security,flow:brainstorm')",
    }),
  ),
  add_tags: Type.Optional(
    Type.String({
      description: "Comma-separated tags to add (preserves existing tags)",
    }),
  ),
  remove_tags: Type.Optional(
    Type.String({
      description: "Comma-separated tags to remove from existing list",
    }),
  ),
  depends_on: Type.Optional(
    Type.String({ description: "Comma-separated ticket IDs this ticket depends on" }),
  ),
  effort: Type.Optional(
    StringEnum(["xs", "s", "m", "l", "xl"] as const, {
      description: "Effort estimate",
    }),
  ),
  content: Type.Optional(
    Type.String({ description: "Markdown content body for the ticket" }),
  ),

  // List params
  all: Type.Optional(Type.Boolean({ description: "Include done tickets in list" })),
  definition: Type.Optional(
    StringEnum(["ready", "questions", "unknown"] as const, {
      description: "Filter by definition state",
    }),
  ),

  // Awareness params
  against: Type.Optional(
    Type.String({ description: "Git ref to run awareness against (required for awareness)" }),
  ),

  // Task params
  task_title: Type.Optional(
    Type.String({ description: "Task title (required for task_add)" }),
  ),
  task_number: Type.Optional(
    Type.Number({ description: "Task number (required for task_complete, task_remove, task_edit)" }),
  ),

  task_detail: Type.Optional(
    Type.String({ description: "Optional markdown body for a task detail doc" }),
  ),
  task_json: Type.Optional(
    Type.String({ description: "JSON array of tasks (required for task_set)" }),
  ),
});

/**
 * supi_tndm_cli — thin wrapper around the tndm CLI.
 *
 * Actions map to tndm subcommands:
 *   create     → tndm ticket create <title> [--status] [--priority] [--type] [--tags] [--depends-on] [--effort] [--content]
 *   update     → tndm ticket update <id> [--title] [--status] [--priority] [--type] [--tags] [--add-tags] [--remove-tags] [--depends-on] [--effort] [--content]
 *   show       → tndm ticket show <id> --json
 *   list       → tndm ticket list [--all] [--definition <state>] --json
 *   awareness  → tndm awareness --against <ref> --json
 *   task_add       → tndm ticket task add <id> --title <title> --json, optionally followed by task detail ensure + sync when task_detail is provided
 *   task_list      → tndm ticket task list <id> --json
 *   task_complete  → tndm ticket task complete <id> <number> --json
 *   task_remove    → tndm ticket task remove <id> <number> --json
 *   task_edit      → tndm ticket task edit <id> <number> [--title] --json, optionally followed by task detail ensure/clear
 *   task_set       → tndm ticket task set <id> --tasks <json> --json
 *   doc_create and sync are internal operations used by flow tools, not exposed here.
 */
export type TndmCliParams = Static<typeof supi_tndm_cli_params>;

export async function executeTndmCli(params: TndmCliParams) {
  const { action } = params;

  switch (action) {
    case "create": {
      if (!params.title) {
        throw new Error("supi_tndm_cli: title is required for create");
      }
      const args: string[] = ["ticket", "create", params.title];
      addOptionalFlags(args, params, [
        "status",
        "priority",
        "type",
        "tags",
        "depends_on",
        "effort",
        "content",
      ]);

      const result = await tndm(args);
      return {
        content: [{ type: "text" as const, text: result.stdout || result.stderr }],
        details: { action: "create", ticketId: result.stdout.trim() },
      };
    }

    case "update": {
      if (!params.id) {
        throw new Error("supi_tndm_cli: id is required for update");
      }
      const args: string[] = ["ticket", "update", params.id];
      addOptionalFlags(args, params, [
        "title",
        "status",
        "priority",
        "type",
        "tags",
        "add_tags",
        "remove_tags",
        "depends_on",
        "effort",
        "content",
      ]);

      const result = await tndm(args);
      return {
        content: [{ type: "text" as const, text: result.stdout || "Ticket updated" }],
        details: { action: "update", ticketId: params.id },
      };
    }

    case "show": {
      if (!params.id) {
        throw new Error("supi_tndm_cli: id is required for show");
      }
      const result = await tndmJson<Record<string, unknown>>([
        "ticket",
        "show",
        params.id,
      ]);
      return {
        content: [{ type: "text" as const, text: JSON.stringify(result, null, 2) }],
        details: { action: "show", ticket: result },
      };
    }

    case "list": {
      const args: string[] = ["ticket", "list"];
      if (params.all) args.push("--all");
      if (params.definition) args.push("--definition", params.definition);

      const rawResult = await tndmJson<
        Record<string, unknown>[] | { schema_version?: number; tickets?: Record<string, unknown>[] }
      >(args);
      const envelope = Array.isArray(rawResult)
        ? { schema_version: 1, tickets: rawResult }
        : rawResult;
      const tickets = Array.isArray(envelope.tickets) ? envelope.tickets : [];
      return {
        content: [
          {
            type: "text" as const,
            text:
              tickets.length > 0
                ? JSON.stringify(envelope, null, 2)
                : "No tickets found.",
          },
        ],
        details: { action: "list", tickets, envelope },
      };
    }

    case "awareness": {
      if (!params.against) {
        throw new Error("supi_tndm_cli: --against is required for awareness");
      }
      const result = await tndmJson<Record<string, unknown>>([
        "awareness",
        "--against",
        params.against,
      ]);
      return {
        content: [{ type: "text" as const, text: JSON.stringify(result, null, 2) }],
        details: { action: "awareness", awareness: result },
      };
    }

    // ── Task actions ────────────────────────────────────────────

    case "task_add": {
      if (!params.id) throw new Error("supi_tndm_cli: id is required for task_add");
      if (!params.task_title) throw new Error("supi_tndm_cli: task_title is required for task_add");
      const args: string[] = ["ticket", "task", "add", params.id, "--title", params.task_title];
      const result = await tndmJson<Record<string, unknown>>(args);
      let finalResult = result;

      if (params.task_detail !== undefined) {
        const taskNumber = extractLatestTaskNumber(result);
        const detailResult = await ensureTaskDetailDoc(params.id, taskNumber);
        writeTaskDetailDoc(detailResult.path, taskNumber, params.task_title, params.task_detail);
        await tndm(["ticket", "sync", params.id]);
        finalResult = await loadTicket(params.id);
      }

      return {
        content: [{ type: "text" as const, text: JSON.stringify(finalResult, null, 2) }],
        details: { action: "task_add", ticketId: params.id, result: finalResult },
      };
    }

    case "task_list": {
      if (!params.id) throw new Error("supi_tndm_cli: id is required for task_list");
      const result = await tndmJson<Record<string, unknown>[]>([
        "ticket", "task", "list", params.id,
      ]);
      return {
        content: [{ type: "text" as const, text: JSON.stringify(result, null, 2) }],
        details: { action: "task_list", ticketId: params.id, tasks: result },
      };
    }

    case "task_complete": {
      if (!params.id) throw new Error("supi_tndm_cli: id is required for task_complete");
      if (params.task_number === undefined) throw new Error("supi_tndm_cli: task_number is required for task_complete");
      const result = await tndmJson<Record<string, unknown>>([
        "ticket", "task", "complete", params.id, String(params.task_number),
      ]);
      return {
        content: [{ type: "text" as const, text: JSON.stringify(result, null, 2) }],
        details: { action: "task_complete", ticketId: params.id, taskNumber: params.task_number, result },
      };
    }

    case "task_remove": {
      if (!params.id) throw new Error("supi_tndm_cli: id is required for task_remove");
      if (params.task_number === undefined) throw new Error("supi_tndm_cli: task_number is required for task_remove");
      const result = await tndmJson<Record<string, unknown>>([
        "ticket", "task", "remove", params.id, String(params.task_number),
      ]);
      return {
        content: [{ type: "text" as const, text: JSON.stringify(result, null, 2) }],
        details: { action: "task_remove", ticketId: params.id, taskNumber: params.task_number, result },
      };
    }

    case "task_edit": {
      if (!params.id) throw new Error("supi_tndm_cli: id is required for task_edit");
      if (params.task_number === undefined) throw new Error("supi_tndm_cli: task_number is required for task_edit");
      const args: string[] = ["ticket", "task", "edit", params.id, String(params.task_number)];
      if (params.task_title !== undefined) args.push("--title", params.task_title);

      const hasManifestFieldChanges = args.length > 5;
      let finalResult: Record<string, unknown> | undefined;

      if (params.task_detail !== undefined) {
        const detailResult = await ensureTaskDetailDoc(params.id, params.task_number);
        const taskSnapshot = hasManifestFieldChanges
          ? await tndmJson<Record<string, unknown>>(args)
          : await loadTicket(params.id);
        const taskTitle =
          params.task_title ??
          extractTaskTitle(taskSnapshot, params.task_number) ??
          `Task ${params.task_number}`;
        writeTaskDetailDoc(
          detailResult.path,
          params.task_number,
          taskTitle,
          params.task_detail,
        );
        await tndm(["ticket", "sync", params.id]);
        finalResult = await loadTicket(params.id);
      } else if (hasManifestFieldChanges) {
        finalResult = await tndmJson<Record<string, unknown>>(args);
      } else {
        finalResult = await tndmJson<Record<string, unknown>>(args);
      }

      return {
        content: [{ type: "text" as const, text: JSON.stringify(finalResult, null, 2) }],
        details: { action: "task_edit", ticketId: params.id, taskNumber: params.task_number, result: finalResult },
      };
    }

    case "task_set": {
      if (!params.id) throw new Error("supi_tndm_cli: id is required for task_set");
      if (!params.task_json) throw new Error("supi_tndm_cli: task_json is required for task_set");
      const result = await tndmJson<Record<string, unknown>>([
        "ticket", "task", "set", params.id, "--tasks", params.task_json,
      ]);
      return {
        content: [{ type: "text" as const, text: JSON.stringify(result, null, 2) }],
        details: { action: "task_set", ticketId: params.id, result },
      };
    }
  }
}

function addOptionalFlags(
  args: string[],
  params: TndmCliParams,
  flags: Array<keyof TndmCliParams>,
): void {
  for (const flag of flags) {
    const value = params[flag];
    if (value === undefined || value === null || value === false) continue;
    const flagName = String(flag).replace(/_/g, "-");
    args.push(`--${flagName}`, String(value));
  }
}

// NOTE: This assumes task numbers are auto-incremented (1, 2, 3…). If task_add ever
// supports explicit task numbering, this will need to use a more precise source
// of truth (e.g. a top-level `task_number` field in the JSON response).
function extractLatestTaskNumber(result: Record<string, unknown>): number {
  const tasks = extractTasks(result);
  const numbers = tasks
    .map((task) => task.number)
    .filter((value): value is number => typeof value === "number");

  if (numbers.length === 0) {
    throw new Error("supi_tndm_cli: task_add did not return a task list; cannot attach task detail");
  }

  return Math.max(...numbers);
}

function extractTaskTitle(result: Record<string, unknown>, taskNumber: number): string | undefined {
  return extractTasks(result).find((task) => task.number === taskNumber)?.title;
}

function extractTasks(result: Record<string, unknown>): Array<{ number?: number; title?: string }> {
  if (Array.isArray(result.tasks)) {
    return result.tasks.filter(
      (task): task is { number?: number; title?: string } => typeof task === "object" && task !== null,
    );
  }

  const ticket = result.ticket;
  if (typeof ticket === "object" && ticket !== null) {
    const state = (ticket as { state?: unknown }).state;
    if (typeof state === "object" && state !== null && Array.isArray((state as { tasks?: unknown }).tasks)) {
      return ((state as { tasks: unknown[] }).tasks).filter(
        (task): task is { number?: number; title?: string } => typeof task === "object" && task !== null,
      );
    }
  }

  return [];
}

async function loadTicket(id: string): Promise<Record<string, unknown>> {
  return tndmJson<Record<string, unknown>>(["ticket", "show", id]);
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
