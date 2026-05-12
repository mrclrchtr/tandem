import { type Static, Type } from "typebox";
import { StringEnum } from "@earendil-works/pi-ai";
import { tndm, tndmJson } from "../cli.js";

export const actionEnum = StringEnum([
  "create",
  "update",
  "show",
  "list",
  "awareness",
  "doc_create",
  "sync",
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

  // Document params
  name: Type.Optional(
    Type.String({ description: "Document name for doc_create (e.g. 'plan', 'archive')" }),
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

      const result = await tndmJson<Record<string, unknown>[]>(args);
      return {
        content: [
          {
            type: "text" as const,
            text:
              result.length > 0
                ? JSON.stringify(result, null, 2)
                : "No tickets found.",
          },
        ],
        details: { action: "list", tickets: result },
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

    case "doc_create": {
      if (!params.id) {
        throw new Error("supi_tndm_cli: id is required for doc_create");
      }
      if (!params.name) {
        throw new Error("supi_tndm_cli: name is required for doc_create");
      }
      const result = await tndm(["ticket", "doc", "create", params.id, params.name]);
      return {
        content: [
          {
            type: "text" as const,
            text: result.stdout || `Document '${params.name}' created for ${params.id}`,
          },
        ],
        details: {
          action: "doc_create",
          ticketId: params.id,
          name: params.name,
          path: result.stdout.trim(),
        },
      };
    }

    case "sync": {
      if (!params.id) {
        throw new Error("supi_tndm_cli: id is required for sync");
      }
      const result = await tndm(["ticket", "sync", params.id]);
      return {
        content: [
          { type: "text" as const, text: result.stdout || `Ticket ${params.id} synced` },
        ],
        details: { action: "sync", ticketId: params.id },
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
