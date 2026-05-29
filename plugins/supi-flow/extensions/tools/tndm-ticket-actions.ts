import type { Static } from "typebox";
import { tndm, tndmJson } from "../cli.js";
import { formatContent, type ToolResult } from "./ticket-helpers.js";
import type { supi_tndm_cli_params } from "./tndm-cli.js";

type TndmCliParams = Static<typeof supi_tndm_cli_params>;

// ─── Shared helpers ───────────────────────────────────────────

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

// ─── Ticket action handlers ───────────────────────────────────

export async function handleCreate(
  params: TndmCliParams,
  signal?: AbortSignal,
): Promise<ToolResult> {
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

  const result = await tndm(args, signal);
  return {
    content: [{ type: "text" as const, text: result.stdout || result.stderr }],
    details: { action: "create", ticketId: result.stdout.trim() },
  };
}

export async function handleUpdate(
  params: TndmCliParams,
  signal?: AbortSignal,
): Promise<ToolResult> {
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

  const result = await tndm(args, signal);
  return {
    content: [{ type: "text" as const, text: result.stdout || "Ticket updated" }],
    details: { action: "update", ticketId: params.id },
  };
}

export async function handleShow(
  params: TndmCliParams,
  signal?: AbortSignal,
): Promise<ToolResult> {
  if (!params.id) {
    throw new Error("supi_tndm_cli: id is required for show");
  }
  const result = await tndmJson<Record<string, unknown>>(
    ["ticket", "show", params.id],
    signal,
  );
  return {
    content: [{ type: "text" as const, text: formatContent(JSON.stringify(result, null, 2)) }],
    details: { action: "show", ticket: result },
  };
}

export async function handleList(
  params: TndmCliParams,
  signal?: AbortSignal,
): Promise<ToolResult> {
  const args: string[] = ["ticket", "list"];
  if (params.all) args.push("--all");
  if (params.definition) args.push("--definition", params.definition);

  const rawResult = await tndmJson<
    Record<string, unknown>[] | { schema_version?: number; tickets?: Record<string, unknown>[] }
  >(args, signal);
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
            ? formatContent(JSON.stringify(envelope, null, 2))
            : "No tickets found.",
      },
    ],
    details: { action: "list", tickets, envelope },
  };
}

export async function handleAwareness(
  params: TndmCliParams,
  signal?: AbortSignal,
): Promise<ToolResult> {
  if (!params.against) {
    throw new Error("supi_tndm_cli: --against is required for awareness");
  }
  const result = await tndmJson<Record<string, unknown>>(
    ["awareness", "--against", params.against],
    signal,
  );
  return {
    content: [{ type: "text" as const, text: formatContent(JSON.stringify(result, null, 2)) }],
    details: { action: "awareness", awareness: result },
  };
}
