import type { Static } from "typebox";
import { tndmJson } from "../cli.js";
import {
  applyTaskMutation,
  extractLatestTaskNumber,
  extractTaskTitle,
  formatContent,
  loadTicket,
  type ToolResult,
} from "./ticket-helpers.js";
import type { supi_tndm_cli_params } from "./tndm-cli.js";

type TndmCliParams = Static<typeof supi_tndm_cli_params>;

// ─── Task action handlers ─────────────────────────────────────

export async function handleTaskAdd(
  params: TndmCliParams,
  signal?: AbortSignal,
): Promise<ToolResult> {
  if (!params.id) throw new Error("supi_tndm_cli: id is required for task_add");
  if (!params.task_title) throw new Error("supi_tndm_cli: task_title is required for task_add");

  const args: string[] = ["ticket", "task", "add", params.id, "--title", params.task_title];
  const result = await tndmJson<Record<string, unknown>>(args, signal);
  let finalResult = result;

  if (params.task_detail !== undefined) {
    const taskNumber = extractLatestTaskNumber(result);
    finalResult = await applyTaskMutation(
      params.id, taskNumber, params.task_title, params.task_detail, signal,
    );
  }

  return {
    content: [{ type: "text" as const, text: JSON.stringify(finalResult, null, 2) }],
    details: { action: "task_add", ticketId: params.id, result: finalResult },
  };
}

export async function handleTaskEdit(
  params: TndmCliParams,
  signal?: AbortSignal,
): Promise<ToolResult> {
  if (!params.id) throw new Error("supi_tndm_cli: id is required for task_edit");
  if (params.task_number === undefined) throw new Error("supi_tndm_cli: task_number is required for task_edit");

  const args: string[] = ["ticket", "task", "edit", params.id, String(params.task_number)];
  if (params.task_title !== undefined) args.push("--title", params.task_title);

  const hasManifestFieldChanges = args.length > 5;
  let finalResult: Record<string, unknown> | undefined;

  if (params.task_detail !== undefined) {
    const applyTitleEdit = hasManifestFieldChanges;
    let taskTitle: string;
    if (params.task_title !== undefined) {
      taskTitle = params.task_title;
    } else {
      const ticket = await loadTicket(params.id, signal);
      taskTitle = extractTaskTitle(ticket, params.task_number) ?? `Task ${params.task_number}`;
    }
    finalResult = await applyTaskMutation(
      params.id, params.task_number, taskTitle, params.task_detail, signal, applyTitleEdit,
    );
  } else {
    finalResult = await tndmJson<Record<string, unknown>>(args, signal);
  }

  return {
    content: [{ type: "text" as const, text: formatContent(JSON.stringify(finalResult, null, 2)) }],
    details: { action: "task_edit", ticketId: params.id, taskNumber: params.task_number, result: finalResult },
  };
}

export async function handleTaskRemove(
  params: TndmCliParams,
  signal?: AbortSignal,
): Promise<ToolResult> {
  if (!params.id) throw new Error("supi_tndm_cli: id is required for task_remove");
  if (params.task_number === undefined) throw new Error("supi_tndm_cli: task_number is required for task_remove");

  const result = await tndmJson<Record<string, unknown>>(
    ["ticket", "task", "remove", params.id, String(params.task_number)],
    signal,
  );
  return {
    content: [{ type: "text" as const, text: formatContent(JSON.stringify(result, null, 2)) }],
    details: { action: "task_remove", ticketId: params.id, taskNumber: params.task_number, result },
  };
}

export async function handleTaskComplete(
  params: TndmCliParams,
  signal?: AbortSignal,
): Promise<ToolResult> {
  if (!params.id) throw new Error("supi_tndm_cli: id is required for task_complete");
  if (params.task_number === undefined) throw new Error("supi_tndm_cli: task_number is required for task_complete");

  const result = await tndmJson<Record<string, unknown>>(
    ["ticket", "task", "complete", params.id, String(params.task_number)],
    signal,
  );
  return {
    content: [{ type: "text" as const, text: formatContent(JSON.stringify(result, null, 2)) }],
    details: { action: "task_complete", ticketId: params.id, taskNumber: params.task_number, result },
  };
}

export async function handleTaskSet(
  params: TndmCliParams,
  signal?: AbortSignal,
): Promise<ToolResult> {
  if (!params.id) throw new Error("supi_tndm_cli: id is required for task_set");
  if (!params.task_json) throw new Error("supi_tndm_cli: task_json is required for task_set");

  const result = await tndmJson<Record<string, unknown>>(
    ["ticket", "task", "set", params.id, "--tasks", params.task_json],
    signal,
  );
  return {
    content: [{ type: "text" as const, text: formatContent(JSON.stringify(result, null, 2)) }],
    details: { action: "task_set", ticketId: params.id, result },
  };
}

export async function handleTaskList(
  params: TndmCliParams,
  signal?: AbortSignal,
): Promise<ToolResult> {
  if (!params.id) throw new Error("supi_tndm_cli: id is required for task_list");

  const result = await tndmJson<Record<string, unknown>[]>(
    ["ticket", "task", "list", params.id],
    signal,
  );
  return {
    content: [{ type: "text" as const, text: formatContent(JSON.stringify(result, null, 2)) }],
    details: { action: "task_list", ticketId: params.id, tasks: result },
  };
}
