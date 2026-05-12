import { type Static, Type } from "typebox";
import { StringEnum } from "@earendil-works/pi-ai";
import { gitAddCommit, tndm, tndmJson } from "../cli.js";

// ─── supi_flow_start ───────────────────────────────────────────

export const supiFlowStartParams = Type.Object({
  title: Type.String({ description: "Ticket title describing the change" }),
  priority: Type.Optional(
    StringEnum(["p0", "p1", "p2", "p3", "p4"] as const, {
      description: "Priority (default: p2)",
    }),
  ),
  type: Type.Optional(
    StringEnum(["task", "bug", "feature", "chore", "epic"] as const, {
      description: "Ticket type (default: task)",
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
  if (params.context) args.push("--content", params.context);

  const result = await tndm(args);
  const ticketId = result.stdout.trim();

  return {
    content: [
      {
        type: "text" as const,
        text: `Created ticket ${ticketId} with status=todo and flow:brainstorm tag.`,
      },
    ],
    details: { action: "flow_start", ticketId, status: "todo", tags: "flow:brainstorm" },
  };
}

// ─── supi_flow_plan ────────────────────────────────────────────

export const supiFlowPlanParams = Type.Object({
  ticket_id: Type.String({ description: "Ticket ID (e.g. TNDM-A1B2C3)" }),
  plan_content: Type.String({
    description:
      "Markdown plan content with tasks numbered as '**Task {N}**'.\n\n- [ ] **Task 1**: Description\n  - File: path/to/file\n  - Verification: command",
  }),
  append: Type.Optional(
    Type.Boolean({
      description:
        "If true, append to existing content. If false (default), replace content entirely.",
    }),
  ),
});

export type FlowPlanParams = Static<typeof supiFlowPlanParams>;

export async function executeFlowPlan(params: FlowPlanParams) {
  let content = params.plan_content;

  if (params.append) {
    try {
      const existing = await tndmJson<{ id: string; content?: string }>([
        "ticket",
        "show",
        params.ticket_id,
      ]);
      if (existing.content) {
        content = existing.content + "\n\n" + content;
      }
    } catch {
      // If reading fails, just use the new content
    }
  }

  await tndm([
    "ticket",
    "update",
    params.ticket_id,
    "--content",
    content,
    "--add-tags",
    "flow:planned",
    "--remove-tags",
    "flow:brainstorm",
  ]);

  return {
    content: [
      {
        type: "text" as const,
        text: `Plan stored in ${params.ticket_id}. Tags updated to flow:planned.`,
      },
    ],
    details: { action: "flow_plan", ticketId: params.ticket_id, tags: "flow:planned" },
  };
}

// ─── supi_flow_complete_task ───────────────────────────────────

export const supiFlowCompleteTaskParams = Type.Object({
  ticket_id: Type.String({ description: "Ticket ID (e.g. TNDM-A1B2C3)" }),
  task_number: Type.Number({
    description: "1-based task number to mark as complete (e.g. 1, 2, 3)",
  }),
});

export type FlowCompleteTaskParams = Static<typeof supiFlowCompleteTaskParams>;

type CheckTaskResult =
  | { kind: "unchecked"; updatedContent: string }
  | { kind: "already_checked" }
  | { kind: "not_found" };

function checkTask(content: string, taskNumber: number): CheckTaskResult {
  // Match a task line like "- [ ] **Task N:**" or "  - [ ] **Task N:**"
  const lines = content.split("\n");
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    const trimmed = line.trimStart();

    const uncheckedMatch = trimmed.match(
      new RegExp(`^- \\[ \\] \\*\\*Task ${taskNumber}\\*\\*:`),
    );
    if (uncheckedMatch) {
      // Replace the [ ] with [x] in the trimmed version
      const indent = line.slice(0, line.length - trimmed.length);
      lines[i] = indent + trimmed.replace("- [ ]", "- [x]");
      return { kind: "unchecked", updatedContent: lines.join("\n") };
    }

    const checkedMatch = trimmed.match(
      new RegExp(`^- \\[x\\] \\*\\*Task ${taskNumber}\\*\\*:`),
    );
    if (checkedMatch) {
      return { kind: "already_checked" };
    }
  }
  return { kind: "not_found" };
}

export async function executeFlowCompleteTask(params: FlowCompleteTaskParams) {
  const existing = await tndmJson<{ id: string; content: string }>([
    "ticket",
    "show",
    params.ticket_id,
  ]);

  if (!existing.content) {
    return {
      content: [
        {
          type: "text" as const,
          text: `No content found in ticket ${params.ticket_id}. No tasks to complete.`,
        },
      ],
      details: { action: "flow_complete_task", ticketId: params.ticket_id, error: "No content" },
    };
  }

  const result = checkTask(existing.content, params.task_number);

  switch (result.kind) {
    case "unchecked":
      await tndm(["ticket", "update", params.ticket_id, "--content", result.updatedContent]);
      return {
        content: [
          {
            type: "text" as const,
            text: `Task ${params.task_number} checked off in ${params.ticket_id}.`,
          },
        ],
        details: {
          action: "flow_complete_task",
          ticketId: params.ticket_id,
          taskNumber: params.task_number,
          completed: true,
        },
      };

    case "already_checked":
      return {
        content: [
          {
            type: "text" as const,
            text: `Task ${params.task_number} is already checked off in ${params.ticket_id}.`,
          },
        ],
        details: {
          action: "flow_complete_task",
          ticketId: params.ticket_id,
          taskNumber: params.task_number,
          completed: true,
          skipped: true,
        },
      };

    case "not_found":
      throw new Error(
        `Task ${params.task_number} not found in ticket ${params.ticket_id}.` +
        ` Task must exist as '- [ ] **Task N:**' or '- [x] **Task N:**'.`,
      );
  }
}

// ─── supi_flow_close ───────────────────────────────────────────

export const supiFlowCloseParams = Type.Object({
  ticket_id: Type.String({ description: "Ticket ID (e.g. TNDM-A1B2C3)" }),
  verification_results: Type.Optional(
    Type.String({
      description:
        "Verification results / evidence from the agent. Appended to the ticket content under ## Verification Results.",
    }),
  ),
});

export type FlowCloseParams = Static<typeof supiFlowCloseParams>;

export async function executeFlowClose(params: FlowCloseParams) {
  let content = "";
  try {
    const existing = await tndmJson<{ id: string; content: string }>([
      "ticket",
      "show",
      params.ticket_id,
    ]);
    if (existing.content) {
      content = existing.content;
    }
  } catch {
    // Continue even if read fails
  }

  if (params.verification_results) {
    // Update existing ## Verification Results section or append new one
    const sectionStart = content.indexOf("## Verification Results");
    if (sectionStart !== -1) {
      const afterHeading = content.slice(sectionStart);
      const nextHeadingPos = afterHeading.search(/\n## /);
      const sectionEnd = nextHeadingPos !== -1 ? sectionStart + nextHeadingPos : content.length;
      content =
        content.slice(0, sectionStart) +
        `## Verification Results\n\n${params.verification_results}` +
        content.slice(sectionEnd);
    } else {
      content += `\n\n## Verification Results\n\n${params.verification_results}`;
    }
  }

  const updateArgs: string[] = [
    "ticket",
    "update",
    params.ticket_id,
    "--status",
    "done",
    "--add-tags",
    "flow:done",
    "--remove-tags",
    "flow:applying",
  ];
  if (content) {
    updateArgs.push("--content", content);
  }

  await tndm(updateArgs);

  let commitHash = "";
  try {
    const commitResult = await gitAddCommit(`chore(tndm): close ${params.ticket_id}`);
    commitHash = commitResult.commitHash;
  } catch {
    // Non-fatal if commit fails
  }

  return {
    content: [
      {
        type: "text" as const,
        text: `Ticket ${params.ticket_id} closed (status=done, flow:done).${
          commitHash ? ` Committed as ${commitHash}.` : ""
        }`,
      },
    ],
    details: {
      action: "flow_close",
      ticketId: params.ticket_id,
      status: "done",
      tags: "flow:done",
      commitHash,
    },
  };
}
