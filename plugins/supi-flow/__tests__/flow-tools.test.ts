import { beforeEach, describe, expect, it, vi } from "vitest";
import { readFileSync } from "node:fs";
import { mkdtempSync } from "node:fs";
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
  vi.clearAllMocks();
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

// ─── executeFlowClose ──────────────────────────────────────────

describe("executeFlowClose", () => {
  function setup(): string {
    const tmpDir = mkdtempSync(join(tmpdir(), "tndm-close-"));
    const archivePath = join(tmpDir, "archive.md");

    vi.mocked(tndmJson).mockResolvedValue({ path: archivePath });
    return archivePath;
  }

  it("updates status and tags in a single atomic call", async () => {
    setup();

    await flowTools.executeFlowClose({
      ticket_id: "TNDM-TEST",
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

    // Should have called doc create for archive via tndmJson
    expect(vi.mocked(tndmJson)).toHaveBeenCalledWith([
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
