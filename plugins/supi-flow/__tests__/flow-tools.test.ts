import { beforeEach, describe, expect, it, vi } from "vitest";
import { mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";

// Mock cli.ts modules used by flow-tools
vi.mock("../extensions/cli.js", () => {
  const mockTndm = vi.fn();
  const mockTndmJson = vi.fn();
  return {
    tndm: mockTndm,
    tndmJson: mockTndmJson,
  };
});

const { tndm, tndmJson } = await import("../extensions/cli.js");
const flowTools = await import("../extensions/tools/flow-tools.js");

beforeEach(() => {
  vi.resetAllMocks();
});

// ─── executeFlowStart ──────────────────────────────────────────

describe("executeFlowStart", () => {
  it("creates a ticket with title, status todo, and flow:brainstorm tag", async () => {
    vi.mocked(tndmJson).mockResolvedValue({
      id: "TNDM-TEST",
      content_path: "/tmp/.tndm/tickets/TNDM-TEST/content.md",
    });

    const result = await flowTools.executeFlowStart({
      title: "My change",
    });

    expect(vi.mocked(tndmJson)).toHaveBeenCalledWith([
      "ticket",
      "create",
      "My change",
      "--status",
      "todo",
      "--tags",
      "flow:brainstorm",
    ]);
    // No context, so tndm should not be called
    expect(vi.mocked(tndm)).not.toHaveBeenCalled();
    expect(result.content[0].text).toContain("Created ticket TNDM-TEST");
    expect(result.content[0].text).toContain("at /tmp/.tndm/tickets/TNDM-TEST");
    expect(result.details).toEqual({
      action: "flow_start",
      ticketId: "TNDM-TEST",
      ticketPath: "/tmp/.tndm/tickets/TNDM-TEST",
      status: "todo",
      tags: "flow:brainstorm",
    });
  });

  it("writes optional context to the canonical ticket content", async () => {
    vi.mocked(tndmJson).mockResolvedValue({
      id: "TNDM-OPT",
      content_path: "/tmp/.tndm/tickets/TNDM-OPT/content.md",
    });
    vi.mocked(tndm).mockResolvedValue({ stdout: "", stderr: "" });

    const result = await flowTools.executeFlowStart({
      title: "Optimized change",
      priority: "p1",
      type: "feature",
      context: "Design summary for the change",
    });

    expect(vi.mocked(tndmJson)).toHaveBeenCalledWith([
      "ticket",
      "create",
      "Optimized change",
      "--status",
      "todo",
      "--tags",
      "flow:brainstorm",
      "--priority",
      "p1",
      "--type",
      "feature",
    ]);
    expect(vi.mocked(tndm)).toHaveBeenCalledWith([
      "ticket",
      "update",
      "TNDM-OPT",
      "--content",
      "Design summary for the change",
    ]);
    expect(vi.mocked(tndm)).toHaveBeenCalledTimes(1);
    expect(result.details.ticketId).toBe("TNDM-OPT");
    expect(result.content[0].text).toContain("at /tmp/.tndm/tickets/TNDM-OPT");
    expect(result.details.ticketPath).toBe("/tmp/.tndm/tickets/TNDM-OPT");
  });
});

// ─── executeFlowPlan ───────────────────────────────────────────

describe("executeFlowPlan", () => {
  it("stores overview markdown in content.md without requiring tasks", async () => {
    vi.mocked(tndm).mockResolvedValue({ stdout: "", stderr: "" });

    const planContent = `## Overview

Ship the simplified workflow in phases.

No tasks yet.`;

    await flowTools.executeFlowPlan({
      ticket_id: "TNDM-TEST",
      plan_content: planContent,
    });

    expect(vi.mocked(tndmJson)).not.toHaveBeenCalled();
    expect(vi.mocked(tndm)).toHaveBeenNthCalledWith(1, [
      "ticket",
      "update",
      "TNDM-TEST",
      "--content",
      planContent,
    ]);
    expect(vi.mocked(tndm)).toHaveBeenNthCalledWith(2, [
      "ticket",
      "update",
      "TNDM-TEST",
      "--remove-tags",
      "flow:brainstorm,flow:planned,flow:applying,flow:done",
      "--add-tags",
      "flow:planned",
    ]);
  });

  it("does not treat checklist-style task text as executable task parsing", async () => {
    vi.mocked(tndm).mockResolvedValue({ stdout: "", stderr: "" });

    const planContent = `## Execution ideas

- [ ] **Task 1**: Draft parser work
  - Files: src/parser.ts
  - Verification: pnpm exec vitest run`;

    await flowTools.executeFlowPlan({
      ticket_id: "TNDM-RAW",
      plan_content: planContent,
    });

    expect(vi.mocked(tndmJson)).not.toHaveBeenCalled();
    expect(vi.mocked(tndm)).toHaveBeenNthCalledWith(1, [
      "ticket",
      "update",
      "TNDM-RAW",
      "--content",
      planContent,
    ]);
  });

  it("returns overview persistence details instead of task counts", async () => {
    vi.mocked(tndm).mockResolvedValue({ stdout: "", stderr: "" });

    const result = await flowTools.executeFlowPlan({
      ticket_id: "TNDM-CNT",
      plan_content: "## Overview\n\nStore this overview only.",
    });

    expect(result.details).toEqual({
      action: "flow_plan",
      ticketId: "TNDM-CNT",
      tags: "flow:planned",
      contentStored: true,
    });
  });

  it("rejects blank overview content before mutating the ticket", async () => {
    await expect(
      flowTools.executeFlowPlan({
        ticket_id: "TNDM-BLANK",
        plan_content: " \n\t ",
      }),
    ).rejects.toThrow("plan_content must not be blank");

    expect(vi.mocked(tndm)).not.toHaveBeenCalled();
  });
});

// ─── executeFlowApply ──────────────────────────────────────────

describe("executeFlowApply", () => {
  it("transitions a planned ticket to applying and returns overview/task context", async () => {
    const tmpDir = mkdtempSync(join(tmpdir(), "flow-apply-"));
    const contentPath = join(tmpDir, "content.md");
    writeFileSync(contentPath, "## Approved Overview\n\nShip it.", "utf-8");

    vi.mocked(tndmJson)
      .mockResolvedValueOnce({
        schema_version: 1,
        id: "TNDM-APPLY",
        status: "todo",
        tags: ["flow:planned"],
        content_path: contentPath,
      })
      .mockResolvedValueOnce([
        {
          number: 1,
          title: "Implement apply",
          status: "todo",
          files: ["extensions/tools/flow-tools.ts"],
          verification: "pnpm exec vitest run __tests__/flow-tools.test.ts",
          detail_path: "tasks/task-01.md",
        },
      ]);
    vi.mocked(tndm).mockResolvedValue({ stdout: "", stderr: "" });

    const result = await flowTools.executeFlowApply({
      ticket_id: "TNDM-APPLY",
    });

    expect(vi.mocked(tndmJson)).toHaveBeenNthCalledWith(1, [
      "ticket",
      "show",
      "TNDM-APPLY",
    ]);
    expect(vi.mocked(tndmJson)).toHaveBeenNthCalledWith(2, [
      "ticket",
      "task",
      "list",
      "TNDM-APPLY",
    ]);
    expect(vi.mocked(tndm)).toHaveBeenCalledWith([
      "ticket",
      "update",
      "TNDM-APPLY",
      "--status",
      "in_progress",
      "--remove-tags",
      "flow:planned",
      "--add-tags",
      "flow:applying",
    ]);
    expect(result.content[0].text).toContain("flow:applying");
    expect(result.details).toMatchObject({
      action: "flow_apply",
      ticketId: "TNDM-APPLY",
      transitioned: true,
      contentPath,
      overview: "## Approved Overview\n\nShip it.",
      tasks: [
        expect.objectContaining({
          number: 1,
          title: "Implement apply",
          detail_path: "tasks/task-01.md",
        }),
      ],
    });
  });

  it("treats already-applying tickets as idempotent re-entry", async () => {
    const tmpDir = mkdtempSync(join(tmpdir(), "flow-apply-existing-"));
    const contentPath = join(tmpDir, "content.md");
    writeFileSync(contentPath, "## Approved Overview\n\nResume work.", "utf-8");

    vi.mocked(tndmJson)
      .mockResolvedValueOnce({
        schema_version: 1,
        id: "TNDM-APPLYING",
        status: "in_progress",
        tags: ["flow:applying"],
        content_path: contentPath,
      })
      .mockResolvedValueOnce([
        {
          number: 1,
          title: "Resume apply",
          status: "todo",
        },
      ]);

    const result = await flowTools.executeFlowApply({
      ticket_id: "TNDM-APPLYING",
    });

    expect(vi.mocked(tndm)).not.toHaveBeenCalled();
    expect(result.details).toMatchObject({
      action: "flow_apply",
      ticketId: "TNDM-APPLYING",
      transitioned: false,
      overview: "## Approved Overview\n\nResume work.",
    });
  });
});

// ─── executeFlowTask ───────────────────────────────────────────

describe("executeFlowTask", () => {
  it("adds one task at a time and auto-writes canonical detail docs", async () => {
    const tmpDir = mkdtempSync(join(tmpdir(), "flow-task-add-"));
    const docPath = join(tmpDir, "tasks", "task-01.md");
    const finalTicket = {
      schema_version: 1,
      id: "TNDM-TASK",
      tasks: [
        {
          number: 1,
          title: "Detailed task",
          status: "todo",
          files: ["src/task.ts", "tests/task.test.ts"],
          verification: "pnpm exec vitest run",
          notes: "Needs task detail",
          detail_path: "tasks/task-01.md",
        },
      ],
    };

    vi.mocked(tndmJson)
      .mockResolvedValueOnce({
        schema_version: 1,
        id: "TNDM-TASK",
        tasks: [
          {
            number: 1,
            title: "Detailed task",
            status: "todo",
            files: ["src/task.ts", "tests/task.test.ts"],
            verification: "pnpm exec vitest run",
            notes: "Needs task detail",
          },
        ],
      })
      .mockResolvedValueOnce({ path: docPath, detail_path: "tasks/task-01.md" })
      .mockResolvedValueOnce(finalTicket);
    vi.mocked(tndm).mockResolvedValue({ stdout: "", stderr: "" });

    const result = await flowTools.executeFlowTask({
      ticket_id: "TNDM-TASK",
      operation: "add",
      title: "Detailed task",
      files: ["src/task.ts", "tests/task.test.ts"],
      verification: "pnpm exec vitest run",
      notes: "Needs task detail",
      detail: "Implementation notes go here.",
    });

    expect(vi.mocked(tndmJson)).toHaveBeenNthCalledWith(1, [
      "ticket",
      "task",
      "add",
      "TNDM-TASK",
      "--title",
      "Detailed task",
      "--file",
      "src/task.ts",
      "--file",
      "tests/task.test.ts",
      "--verification",
      "pnpm exec vitest run",
      "--notes",
      "Needs task detail",
    ]);
    expect(vi.mocked(tndmJson)).toHaveBeenNthCalledWith(2, [
      "ticket",
      "task",
      "detail",
      "ensure",
      "TNDM-TASK",
      "1",
    ]);
    expect(vi.mocked(tndmJson)).toHaveBeenNthCalledWith(3, [
      "ticket",
      "show",
      "TNDM-TASK",
    ]);
    expect(vi.mocked(tndm)).toHaveBeenCalledWith(["ticket", "sync", "TNDM-TASK"]);
    expect(readFileSync(docPath, "utf-8")).toContain("Implementation notes go here.");
    expect(result.details.taskNumber).toBe(1);
    expect(result.details.result).toEqual(finalTicket);
    expect(result.content[0].text).toContain("Task 1 added");
  });

  it("edits only task detail without issuing a no-op manifest edit", async () => {
    const tmpDir = mkdtempSync(join(tmpdir(), "flow-task-edit-"));
    const docPath = join(tmpDir, "tasks", "task-02.md");
    const existingTicket = {
      schema_version: 1,
      id: "TNDM-TASK",
      tasks: [{ number: 2, title: "Existing task", status: "todo" }],
    };
    const finalTicket = {
      schema_version: 1,
      id: "TNDM-TASK",
      tasks: [
        {
          number: 2,
          title: "Existing task",
          status: "todo",
          detail_path: "tasks/task-02.md",
        },
      ],
    };

    vi.mocked(tndmJson)
      .mockResolvedValueOnce({ path: docPath, detail_path: "tasks/task-02.md" })
      .mockResolvedValueOnce(existingTicket)
      .mockResolvedValueOnce(finalTicket);
    vi.mocked(tndm).mockResolvedValue({ stdout: "", stderr: "" });

    const result = await flowTools.executeFlowTask({
      ticket_id: "TNDM-TASK",
      operation: "edit",
      task_number: 2,
      detail: "Revised task detail.",
    });

    expect(vi.mocked(tndmJson)).toHaveBeenNthCalledWith(1, [
      "ticket",
      "task",
      "detail",
      "ensure",
      "TNDM-TASK",
      "2",
    ]);
    expect(vi.mocked(tndmJson)).toHaveBeenNthCalledWith(2, [
      "ticket",
      "show",
      "TNDM-TASK",
    ]);
    expect(vi.mocked(tndmJson)).toHaveBeenNthCalledWith(3, [
      "ticket",
      "show",
      "TNDM-TASK",
    ]);
    expect(vi.mocked(tndm)).toHaveBeenCalledWith(["ticket", "sync", "TNDM-TASK"]);
    expect(readFileSync(docPath, "utf-8")).toContain("Revised task detail.");
    expect(result.details.taskNumber).toBe(2);
    expect(result.details.result).toEqual(finalTicket);
    expect(result.content[0].text).toContain("Task 2 updated");
  });

  it("removes one task at a time", async () => {
    vi.mocked(tndmJson).mockResolvedValue({ ok: true });

    const result = await flowTools.executeFlowTask({
      ticket_id: "TNDM-TASK",
      operation: "remove",
      task_number: 3,
    });

    expect(vi.mocked(tndmJson)).toHaveBeenCalledWith([
      "ticket",
      "task",
      "remove",
      "TNDM-TASK",
      "3",
    ]);
    expect(result.details.removed).toBe(true);
    expect(result.content[0].text).toContain("Task 3 removed");
  });
});

// ─── executeFlowCompleteTask ───────────────────────────────────

describe("executeFlowCompleteTask", () => {
  it("calls task complete CLI and returns success", async () => {
    vi.mocked(tndmJson).mockResolvedValue({
      id: "TNDM-TEST",
      tasks: [{ number: 1, title: "Do thing", status: "done" }],
    });

    const result = await flowTools.executeFlowCompleteTask({
      ticket_id: "TNDM-TEST",
      task_number: 1,
    });

    expect(vi.mocked(tndmJson)).toHaveBeenCalledWith([
      "ticket",
      "task",
      "complete",
      "TNDM-TEST",
      "1",
    ]);

    expect(result.details.completed).toBe(true);
    expect(result.details.taskNumber).toBe(1);
  });

  it("throws when task number does not exist", async () => {
    vi.mocked(tndmJson).mockRejectedValue(
      new Error("task 99 not found"),
    );

    await expect(
      flowTools.executeFlowCompleteTask({
        ticket_id: "TNDM-TEST",
        task_number: 99,
      }),
    ).rejects.toThrow("Task 99 not found");
  });

  it("re-throws unexpected errors", async () => {
    vi.mocked(tndmJson).mockRejectedValue(
      new Error("tndm is not installed"),
    );

    await expect(
      flowTools.executeFlowCompleteTask({
        ticket_id: "TNDM-TEST",
        task_number: 1,
      }),
    ).rejects.toThrow("tndm is not installed");
  });
});

describe("supiFlowCloseParams", () => {
  it("describes verification_results as archive.md content", () => {
    const schema = flowTools.supiFlowCloseParams.properties
      .verification_results as { description?: string };

    expect(schema.description).toContain("archive.md");
    expect(schema.description).not.toContain("ticket content");
  });
});

// ─── executeFlowClose ──────────────────────────────────────────

describe("executeFlowClose", () => {
  function setup(): string {
    const tmpDir = mkdtempSync(join(tmpdir(), "tndm-close-"));
    const archivePath = join(tmpDir, "archive.md");

    vi.mocked(tndmJson)
      .mockResolvedValueOnce([
        { number: 1, title: "Done task", status: "done" },
      ])
      .mockResolvedValueOnce({ path: archivePath });
    return archivePath;
  }

  it("requires nonblank verification results before mutating the ticket", async () => {
    await expect(
      flowTools.executeFlowClose({
        ticket_id: "TNDM-TEST",
      } as never),
    ).rejects.toThrow("verification_results is required");

    expect(vi.mocked(tndm)).not.toHaveBeenCalled();
    expect(vi.mocked(tndmJson)).not.toHaveBeenCalled();
  });

  it("refuses to close when structured tasks are still incomplete", async () => {
    vi.mocked(tndmJson).mockResolvedValue([
      { number: 1, title: "Incomplete task", status: "todo" },
    ]);

    await expect(
      flowTools.executeFlowClose({
        ticket_id: "TNDM-TEST",
        verification_results: "Ran checks.",
      }),
    ).rejects.toThrow("incomplete tasks");

    expect(vi.mocked(tndm)).not.toHaveBeenCalled();
  });

  it("updates status and tags in a single atomic call", async () => {
    setup();

    await flowTools.executeFlowClose({
      ticket_id: "TNDM-TEST",
      verification_results: "All tests pass.",
    });

    // Should remove all flow-state tags, set status, and add flow:done in one call
    expect(vi.mocked(tndm)).toHaveBeenCalledWith([
      "ticket",
      "update",
      "TNDM-TEST",
      "--remove-tags",
      "flow:brainstorm,flow:planned,flow:applying,flow:done",
      "--status",
      "done",
      "--add-tags",
      "flow:done",
    ]);
  });

  it("writes verification results to archive.md and syncs", async () => {
    const archivePath = setup();

    await flowTools.executeFlowClose({
      ticket_id: "TNDM-TEST",
      verification_results: "All tests pass.",
    });

    expect(vi.mocked(tndmJson)).toHaveBeenNthCalledWith(1, [
      "ticket",
      "task",
      "list",
      "TNDM-TEST",
    ]);
    expect(vi.mocked(tndmJson)).toHaveBeenNthCalledWith(2, [
      "ticket",
      "doc",
      "create",
      "TNDM-TEST",
      "archive",
    ]);

    // Should have written verification results to the archive file
    const content = readFileSync(archivePath, "utf-8");
    expect(content).toContain("# Archive");
    expect(content).toContain("All tests pass.");

    // Should have synced
    expect(vi.mocked(tndm)).toHaveBeenCalledWith(["ticket", "sync", "TNDM-TEST"]);
  });

});
