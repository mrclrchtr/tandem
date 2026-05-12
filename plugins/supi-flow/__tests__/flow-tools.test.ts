import { beforeEach, describe, expect, it, vi } from "vitest";
import { mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";

// Mock cli.ts modules used by flow-tools
vi.mock("../src/cli.js", () => {
  const mockTndm = vi.fn();
  const mockTndmJson = vi.fn();
  const mockGitAddCommit = vi.fn();
  return {
    tndm: mockTndm,
    tndmJson: mockTndmJson,
    gitAddCommit: mockGitAddCommit,
  };
});

const { tndm, tndmJson, gitAddCommit } = await import("../src/cli.js");
const flowTools = await import("../src/tools/flow-tools.js");

beforeEach(() => {
  vi.clearAllMocks();
});

// ─── executeFlowStart ──────────────────────────────────────────

describe("executeFlowStart", () => {
  it("creates a ticket with title, status todo, and flow:brainstorm tag", async () => {
    vi.mocked(tndm).mockResolvedValue({ stdout: "TNDM-TEST\n", stderr: "" });

    const result = await flowTools.executeFlowStart({
      title: "My change",
    });

    expect(vi.mocked(tndm)).toHaveBeenCalledWith([
      "ticket",
      "create",
      "My change",
      "--status",
      "todo",
      "--tags",
      "flow:brainstorm",
    ]);
    expect(result.content[0].text).toContain("Created ticket TNDM-TEST");
    expect(result.details).toEqual({
      action: "flow_start",
      ticketId: "TNDM-TEST",
      status: "todo",
      tags: "flow:brainstorm",
    });
  });

  it("passes optional priority, type, and context", async () => {
    vi.mocked(tndm).mockResolvedValue({ stdout: "TNDM-OPT\n", stderr: "" });

    const result = await flowTools.executeFlowStart({
      title: "Optimized change",
      priority: "p1",
      type: "feature",
      context: "Design summary for the change",
    });

    expect(vi.mocked(tndm)).toHaveBeenCalledWith([
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
      "--content",
      "Design summary for the change",
    ]);
    expect(result.details.ticketId).toBe("TNDM-OPT");
  });
});

// ─── executeFlowPlan ───────────────────────────────────────────

describe("executeFlowPlan", () => {
  it("creates a plan document, writes content, syncs, and updates tags", async () => {
    // Mock doc create returning a temp path
    const tmpDir = mkdtempSync(join(tmpdir(), "tndm-plan-test-"));
    const docPath = join(tmpDir, "plan.md");
    vi.mocked(tndm).mockImplementation(async (args: string[]) => {
      if (args[0] === "ticket" && args[1] === "doc" && args[2] === "create") {
        return { stdout: docPath + "\n", stderr: "" };
      }
      return { stdout: "", stderr: "" };
    });

    await flowTools.executeFlowPlan({
      ticket_id: "TNDM-TEST",
      plan_content: "- [ ] **Task 1**: Do thing",
    });

    // Should have called doc create first
    expect(vi.mocked(tndm)).toHaveBeenCalledWith([
      "ticket",
      "doc",
      "create",
      "TNDM-TEST",
      "plan",
    ]);

    // Should have written the plan content to the file
    const written = readFileSync(docPath, "utf-8");
    expect(written).toContain("Task 1");

    // Should have called sync
    expect(vi.mocked(tndm)).toHaveBeenCalledWith(["ticket", "sync", "TNDM-TEST"]);

    // Should have updated tags
    expect(vi.mocked(tndm)).toHaveBeenCalledWith([
      "ticket",
      "update",
      "TNDM-TEST",
      "--add-tags",
      "flow:planned",
      "--remove-tags",
      "flow:brainstorm",
    ]);
  });

  it("appends to existing content when append=true", async () => {
    const tmpDir = mkdtempSync(join(tmpdir(), "tndm-plan-append-"));
    const docPath = join(tmpDir, "plan.md");
    writeFileSync(docPath, "Existing content\n", "utf-8");

    vi.mocked(tndm).mockImplementation(async (args: string[]) => {
      if (args[0] === "ticket" && args[1] === "doc" && args[2] === "create") {
        return { stdout: docPath + "\n", stderr: "" };
      }
      return { stdout: "", stderr: "" };
    });

    await flowTools.executeFlowPlan({
      ticket_id: "TNDM-TEST",
      plan_content: "- [ ] **Task 1**: Do thing",
      append: true,
    });

    const written = readFileSync(docPath, "utf-8");
    expect(written).toContain("Existing content");
    expect(written).toContain("Task 1");
  });
});

// ─── executeFlowCompleteTask ───────────────────────────────────

describe("executeFlowCompleteTask", () => {
  function setupContent(initialContent: string): string {
    const tmpDir = mkdtempSync(join(tmpdir(), "tndm-complete-"));
    const docPath = join(tmpDir, "content.md");
    writeFileSync(docPath, initialContent, "utf-8");

    vi.mocked(tndmJson).mockResolvedValue({
      id: "TNDM-TEST",
      content_path: docPath,
    });
    vi.mocked(tndm).mockResolvedValue({ stdout: "", stderr: "" });

    return docPath;
  }

  it("checks off an unchecked task via file edit and sync", async () => {
    const docPath = setupContent("- [ ] **Task 1**: Do the thing\n- [ ] **Task 2**: Do another");

    const result = await flowTools.executeFlowCompleteTask({
      ticket_id: "TNDM-TEST",
      task_number: 1,
    });

    expect(result.details.completed).toBe(true);
    // Should have written to the file
    const content = readFileSync(docPath, "utf-8");
    expect(content).toContain("- [x] **Task 1**");
    expect(content).toContain("- [ ] **Task 2**");
    // Should have synced
    expect(vi.mocked(tndm)).toHaveBeenCalledWith(["ticket", "sync", "TNDM-TEST"]);
  });

  it("soft-fails when task is already checked off", async () => {
    setupContent("- [x] **Task 1**: Already done");

    const result = await flowTools.executeFlowCompleteTask({
      ticket_id: "TNDM-TEST",
      task_number: 1,
    });

    expect(result.details.completed).toBe(true);
    expect(result.details.skipped).toBe(true);
    // Should NOT call sync since no change needed
    expect(vi.mocked(tndm)).not.toHaveBeenCalledWith(["ticket", "sync", "TNDM-TEST"]);
  });

  it("hard-fails when task number does not exist", async () => {
    setupContent("- [ ] **Task 1**: The only task");

    await expect(
      flowTools.executeFlowCompleteTask({
        ticket_id: "TNDM-TEST",
        task_number: 99,
      }),
    ).rejects.toThrow("Task 99 not found");
  });

  it("soft-fails when ticket has no content path", async () => {
    vi.mocked(tndmJson).mockResolvedValue({
      id: "TNDM-TEST",
    });

    const result = await flowTools.executeFlowCompleteTask({
      ticket_id: "TNDM-TEST",
      task_number: 1,
    });

    expect(result.details.error).toContain("content path");
  });

  it("soft-fails when content file does not exist", async () => {
    vi.mocked(tndmJson).mockResolvedValue({
      id: "TNDM-TEST",
      content_path: "/nonexistent/path/content.md",
    });

    const result = await flowTools.executeFlowCompleteTask({
      ticket_id: "TNDM-TEST",
      task_number: 1,
    });

    expect(result.details.error).toContain("content file");
  });
});

// ─── executeFlowClose ──────────────────────────────────────────

describe("executeFlowClose", () => {
  function setupWithContent(initialContent: string): string {
    const tmpDir = mkdtempSync(join(tmpdir(), "tndm-close-"));
    const docPath = join(tmpDir, "content.md");
    writeFileSync(docPath, initialContent, "utf-8");

    vi.mocked(tndmJson).mockImplementation(async () => ({
      id: "TNDM-TEST",
      content_path: docPath,
    }));
    vi.mocked(tndm).mockResolvedValue({ stdout: "", stderr: "" });
    vi.mocked(gitAddCommit).mockResolvedValue({ commitHash: "" });

    return docPath;
  }

  it("updates status and tags without passing content through CLI", async () => {
    setupWithContent("Done stuff");

    await flowTools.executeFlowClose({
      ticket_id: "TNDM-TEST",
    });

    const statusCalls = vi.mocked(tndm).mock.calls.filter(
      (call) => call[0][0] === "ticket" && call[0][1] === "update",
    );
    for (const call of statusCalls) {
      expect(call[0]).not.toContain("--content");
    }
    const latestUpdate = statusCalls[statusCalls.length - 1][0];
    expect(latestUpdate).toContain("flow:done");
    expect(latestUpdate).toContain("flow:applying");
  });

  it("appends verification results to the content file and syncs", async () => {
    const docPath = setupWithContent("## Context\n\nThe work.");

    await flowTools.executeFlowClose({
      ticket_id: "TNDM-TEST",
      verification_results: "All tests pass.",
    });

    // Should have written verification results to the file
    const content = readFileSync(docPath, "utf-8");
    expect(content).toContain("## Verification Results");
    expect(content).toContain("All tests pass.");

    // Should have synced
    expect(vi.mocked(tndm)).toHaveBeenCalledWith(["ticket", "sync", "TNDM-TEST"]);
  });

  it("updates existing verification results section without duplication", async () => {
    const tmpDir = mkdtempSync(join(tmpdir(), "tndm-close-replace-"));
    const docPath = join(tmpDir, "content.md");
    writeFileSync(
      docPath,
      "## Context\n\nWork.\n\n## Verification Results\n\nOld results here.\n\n## Other\n\nMore.",
      "utf-8",
    );

    vi.mocked(tndmJson).mockResolvedValue({
      id: "TNDM-TEST",
      content_path: docPath,
    });
    vi.mocked(tndm).mockResolvedValue({ stdout: "", stderr: "" });

    await flowTools.executeFlowClose({
      ticket_id: "TNDM-TEST",
      verification_results: "New results.",
    });

    // Should have only one Verification Results section
    const content = readFileSync(docPath, "utf-8");
    expect(content.match(/## Verification Results/g)).toHaveLength(1);
    expect(content).toContain("New results.");
    expect(content).not.toContain("Old results here.");
    expect(content).toContain("## Context");
    expect(content).toContain("## Other");
  });

  it("commits after close", async () => {
    setupWithContent("Done.");

    const result = await flowTools.executeFlowClose({
      ticket_id: "TNDM-TEST",
    });

    expect(vi.mocked(gitAddCommit)).toHaveBeenCalledWith(
      "chore(tndm): close TNDM-TEST",
    );
  });
});
