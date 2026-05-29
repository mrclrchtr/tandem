import { existsSync } from "node:fs";
import { readFile } from "node:fs/promises";
import { dirname, isAbsolute, join, resolve } from "node:path";
import {
  DEFAULT_MAX_BYTES,
  DEFAULT_MAX_LINES,
  formatSize,
  truncateHead,
} from "@earendil-works/pi-coding-agent";
import { tndm, tndmJson } from "../cli.js";
import { writeTaskDetailDoc } from "./doc-writes.js";

export type FlowTaskListEntry = {
  number?: number;
  title?: string;
  status?: string;
  detail_path?: string;
};

export type ToolResult = { content: Array<{ type: "text"; text: string }>; details: Record<string, unknown> };

// ─── Flow tag constants ────────────────────────────────────────

export const FLOW_TAGS_ALL =
  "flow:brainstorm,flow:planned,flow:applying,flow:done";
export const FLOW_TAG_BRAINSTORM = "flow:brainstorm";
export const FLOW_TAG_PLANNED = "flow:planned";
export const FLOW_TAG_APPLYING = "flow:applying";
export const FLOW_TAG_DONE = "flow:done";

// ─── findRepoRoot ──────────────────────────────────────────────

/**
 * Walk up from startDir looking for `.git` or `.tndm`.
 *
 * `.tndm` is the on-disk directory name matching the `tndm` CLI binary,
 * following the convention of other git-aware tools (e.g., `.github`).
 */
export function findRepoRoot(startDir = process.cwd()): string {
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

// ─── resolveTicketPath ────────────────────────────────────────

export function resolveTicketPath(ticketPath: string): string {
  if (isAbsolute(ticketPath)) {
    return ticketPath;
  }

  return resolve(findRepoRoot(), ticketPath);
}

// ─── unwrapTicket (envelope helpers) ───────────────────────────

export function unwrapTicket(result: Record<string, unknown>): Record<string, unknown> {
  const ticket = result.ticket;
  if (typeof ticket === "object" && ticket !== null) {
    return ticket as Record<string, unknown>;
  }
  return result;
}

export function extractTicketTags(result: Record<string, unknown>): string[] {
  const ticket = unwrapTicket(result);
  if (!Array.isArray(ticket.tags)) {
    return [];
  }

  return ticket.tags.filter((tag): tag is string => typeof tag === "string");
}

export function extractTicketStatus(result: Record<string, unknown>): string | undefined {
  const ticket = unwrapTicket(result);
  return typeof ticket.status === "string" ? ticket.status : undefined;
}

export function extractContentPath(result: Record<string, unknown>): string | undefined {
  const ticket = unwrapTicket(result);
  return typeof ticket.content_path === "string" ? ticket.content_path : undefined;
}

// ─── Task extraction helpers ──────────────────────────────────

export function filterFlowTasks(tasks: unknown[]): FlowTaskListEntry[] {
  return tasks
    .filter(
      (task): task is FlowTaskListEntry => typeof task === "object" && task !== null,
    )
    .map((task) => {
      // Derive detail_path from canonical naming convention
      if (task.number !== undefined && !task.detail_path) {
        const num = String(task.number).padStart(2, "0");
        return { ...task, detail_path: `tasks/task-${num}.md` };
      }
      return task;
    });
}

export function extractTasks(result: Record<string, unknown>): FlowTaskListEntry[] {
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

export function extractLatestTaskNumber(result: Record<string, unknown>): number {
  const tasks = extractTasks(result);
  const numbers = tasks
    .map((task) => task.number)
    .filter((value): value is number => typeof value === "number");

  if (numbers.length === 0) {
    throw new Error("task_add did not return a task list");
  }

  return Math.max(...numbers);
}

export function extractTaskTitle(
  result: Record<string, unknown>,
  taskNumber: number,
): string | undefined {
  return extractTasks(result).find((task) => task.number === taskNumber)?.title;
}

// ─── CLI wrappers ─────────────────────────────────────────────

export async function loadTicket(
  id: string,
  signal?: AbortSignal,
): Promise<Record<string, unknown>> {
  return tndmJson<Record<string, unknown>>(["ticket", "show", id], signal);
}

export async function ensureTaskDetailDoc(
  id: string,
  taskNumber: number,
  signal?: AbortSignal,
): Promise<{ path: string }> {
  return tndmJson<{ path: string }>(
    ["ticket", "task", "detail", "ensure", id, String(taskNumber)],
    signal,
  );
}

// ─── loadTaskList ──────────────────────────────────────────────

export async function loadTaskList(
  id: string,
  signal?: AbortSignal,
): Promise<FlowTaskListEntry[]> {
  const tasks = await tndmJson<unknown>(["ticket", "task", "list", id], signal);
  if (!Array.isArray(tasks)) {
    throw new Error(`supi_flow: task list for ticket ${id} did not return an array`);
  }
  return filterFlowTasks(tasks);
}

// ─── applyTaskMutation ────────────────────────────────────────

/**
 * Apply a task detail mutation end-to-end.
 *
 * 1. Ensure the detail doc via tndm.
 * 2. Optionally apply a title edit to the manifest.
 * 3. Write the markdown body.
 * 4. Sync the ticket.
 * 5. Reload and return the updated ticket snapshot.
 *
 * Replaces writeTaskDetailAndReload — use this instead.
 */
export async function applyTaskMutation(
  id: string,
  taskNumber: number,
  title: string,
  detail: string,
  signal?: AbortSignal,
  applyTitleEdit?: boolean,
): Promise<Record<string, unknown>> {
  const detailResult = await ensureTaskDetailDoc(id, taskNumber, signal);

  if (applyTitleEdit) {
    await tndmJson<Record<string, unknown>>(
      ["ticket", "task", "edit", id, String(taskNumber), "--title", title],
      signal,
    );
  }

  await writeTaskDetailDoc(detailResult.path, taskNumber, title, detail);
  await tndm(["ticket", "sync", id], signal);
  return loadTicket(id, signal);
}

// ─── formatContent ────────────────────────────────────────────

/**
 * Truncate tool output text that exceeds PI's built-in limits
 * and append a notice so the model knows when output is partial.
 */
export function formatContent(raw: string): string {
  const truncation = truncateHead(raw, {
    maxLines: DEFAULT_MAX_LINES,
    maxBytes: DEFAULT_MAX_BYTES,
  });

  if (!truncation.truncated) return raw;

  return (
    truncation.content +
    `\n\n[Output truncated: showing ${truncation.outputLines} of ${truncation.totalLines} lines` +
    ` (${formatSize(truncation.outputBytes)} of ${formatSize(truncation.totalBytes)}).` +
    ` Full output available in details.]`
  );
}

// ─── Content reading (async) ──────────────────────────────────

export async function readRequiredTicketContent(
  ticketId: string,
  contentPath: string | undefined,
  toolName: string,
): Promise<string> {
  if (!contentPath) {
    throw new Error(`${toolName}: ticket ${ticketId} is missing content_path`);
  }

  let overview: string;
  try {
    overview = await readFile(resolveTicketPath(contentPath), "utf-8");
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    throw new Error(`${toolName}: failed to read content.md for ticket ${ticketId}: ${message}`);
  }

  if (!overview.trim()) {
    throw new Error(`${toolName}: approved overview in content.md must not be blank`);
  }

  return overview;
}
