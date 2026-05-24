import { existsSync, readFileSync } from "node:fs";
import { readFile } from "node:fs/promises";
import { dirname, isAbsolute, join, resolve } from "node:path";
import { tndmJson } from "../cli.js";

export type FlowTaskListEntry = {
  number?: number;
  title?: string;
  status?: string;
  detail_path?: string;
};

// ─── findRepoRoot (memoized) ───────────────────────────────────

let _repoRoot: string | null = null;

/**
 * Walk up from startDir looking for `.git` or `.tndm` — memoized after first call.
 * Export with underscore prefix for testing only.
 */
export function findRepoRoot(startDir = process.cwd()): string {
  if (_repoRoot !== null) return _repoRoot;

  let current = resolve(startDir);

  while (true) {
    if (existsSync(join(current, ".git")) || existsSync(join(current, ".tndm"))) {
      _repoRoot = current;
      return current;
    }

    const parent = dirname(current);
    if (parent === current) {
      throw new Error(`failed to locate repository root from ${startDir}`);
    }
    current = parent;
  }
}

/** Reset the memoized repo root cache (for testing only). */
export function _resetRepoRootCache(): void {
  _repoRoot = null;
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
