import { type Static, Type } from "typebox";
import { StringEnum } from "@earendil-works/pi-ai";
import { type ToolResult } from "./ticket-helpers.js";
import { handleCreate, handleUpdate, handleShow, handleList, handleAwareness } from "./tndm-ticket-actions.js";
import { handleTaskAdd, handleTaskEdit, handleTaskRemove, handleTaskComplete, handleTaskSet, handleTaskList } from "./tndm-task-actions.js";

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
  id: Type.Optional(Type.String()),

  // Create / Update params
  title: Type.Optional(Type.String()),
  status: Type.Optional(
    StringEnum(["todo", "in_progress", "blocked", "done"] as const),
  ),
  priority: Type.Optional(
    StringEnum(["p0", "p1", "p2", "p3", "p4"] as const),
  ),
  type: Type.Optional(
    StringEnum(["task", "bug", "feature", "chore", "epic"] as const),
  ),
  tags: Type.Optional(
    Type.String({
      description: "Comma-separated tags to replace",
    }),
  ),
  add_tags: Type.Optional(
    Type.String({
      description: "Comma-separated tags to add",
    }),
  ),
  remove_tags: Type.Optional(
    Type.String({
      description: "Comma-separated tags to remove",
    }),
  ),
  depends_on: Type.Optional(
    Type.String({ description: "Comma-separated dependency IDs" }),
  ),
  effort: Type.Optional(
    StringEnum(["xs", "s", "m", "l", "xl"] as const, {
      description: "Effort estimate",
    }),
  ),
  content: Type.Optional(
    Type.String({ description: "Markdown ticket content" }),
  ),

  // List params
  all: Type.Optional(Type.Boolean({ description: "Include done tickets" })),
  definition: Type.Optional(
    StringEnum(["ready", "questions", "unknown"] as const, {
      description: "Definition state filter",
    }),
  ),

  // Awareness params
  against: Type.Optional(
    Type.String({ description: "Git ref for awareness" }),
  ),

  // Task params
  task_title: Type.Optional(Type.String()),
  task_number: Type.Optional(Type.Number()),

  task_detail: Type.Optional(
    Type.String({ description: "Task detail markdown" }),
  ),
  task_json: Type.Optional(
    Type.String({ description: "JSON task array" }),
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

const handlers: Record<string, (p: TndmCliParams, s?: AbortSignal) => Promise<ToolResult>> = {
  create: handleCreate,
  update: handleUpdate,
  show: handleShow,
  list: handleList,
  awareness: handleAwareness,
  task_add: handleTaskAdd,
  task_edit: handleTaskEdit,
  task_remove: handleTaskRemove,
  task_complete: handleTaskComplete,
  task_set: handleTaskSet,
  task_list: handleTaskList,
};

export async function executeTndmCli(
  params: TndmCliParams,
  signal?: AbortSignal,
) {
  const handler = handlers[params.action];
  if (!handler) throw new Error(`supi_tndm_cli: unknown action "${params.action}"`);
  return handler(params, signal);
}
