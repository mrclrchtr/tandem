import { beforeEach, describe, expect, it, vi } from "vitest";
import { mkdirSync, mkdtempSync, readFileSync, writeFileSync } from "node:fs";
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
  it("creates a plan document, writes content, syncs, and updates tags", async () => {
    // Mock doc create returning a temp path
    const tmpDir = mkdtempSync(join(tmpdir(), "tndm-plan-test-"));
    const docPath = join(tmpDir, "plan.md");
    vi.mocked(tndmJson).mockResolvedValue({ path: docPath });

    await flowTools.executeFlowPlan({
      ticket_id: "TNDM-TEST",
      plan_content: "- [ ] **Task 1**: Do thing",
    });

    // Should have called doc create first via tndmJson
    expect(vi.mocked(tndmJson)).toHaveBeenCalledWith([
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

    // Should have updated tags: remove all flow-state tags first, then add flow:planned
    expect(vi.mocked(tndm)).toHaveBeenCalledWith([
      "ticket",
      "update",
      "TNDM-TEST",
      "--remove-tags",
      "flow:brainstorm,flow:planned,flow:applying,flow:done",
    ]);
    expect(vi.mocked(tndm)).toHaveBeenCalledWith([
      "ticket",
      "update",
      "TNDM-TEST",
      "--add-tags",
      "flow:planned",
    ]);
  });

  it("appends to existing content when append=true", async () => {
    const tmpDir = mkdtempSync(join(tmpdir(), "tndm-plan-append-"));
    const docPath = join(tmpDir, "plan.md");
    writeFileSync(docPath, "Existing content\n", "utf-8");

    vi.mocked(tndmJson).mockResolvedValue({ path: docPath });

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
    const docPath = join(tmpDir, "plan.md");
    writeFileSync(docPath, initialContent, "utf-8");

    vi.mocked(tndmJson).mockResolvedValue({
      id: "TNDM-TEST",
      content_path: join(tmpDir, "content.md"),
      documents: [
        { name: "content", path: "content.md" },
        { name: "plan", path: "plan.md" },
      ],
    });
    vi.mocked(tndm).mockResolvedValue({ stdout: "", stderr: "" });

    return docPath;
  }

  it("checks off an unchecked task via the registered plan document path and sync", async () => {
    const tmpDir = mkdtempSync(join(tmpdir(), "tndm-complete-docs-"));
    const planDir = join(tmpDir, "nested");
    mkdirSync(planDir, { recursive: true });
    const docPath = join(planDir, "plan.md");
    writeFileSync(docPath, "- [ ] **Task 1**: Do the thing\n- [ ] **Task 2**: Do another", "utf-8");

    vi.mocked(tndmJson).mockResolvedValue({
      id: "TNDM-TEST",
      content_path: join(tmpDir, "content.md"),
      documents: [
        { name: "content", path: "content.md" },
        { name: "plan", path: "nested/plan.md" },
      ],
    });
    vi.mocked(tndm).mockResolvedValue({ stdout: "", stderr: "" });

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

  it("soft-fails when no plan document is registered", async () => {
    vi.mocked(tndmJson).mockResolvedValue({
      id: "TNDM-TEST",
      content_path: "/nonexistent/path/content.md",
      documents: [{ name: "content", path: "content.md" }],
    });

    const result = await flowTools.executeFlowCompleteTask({
      ticket_id: "TNDM-TEST",
      task_number: 1,
    });

    expect(result.details.error).toContain("plan file");
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

  it("updates status and tags", async () => {
    setup();

    await flowTools.executeFlowClose({
      ticket_id: "TNDM-TEST",
    });

    // Should first remove all flow-state tags
    expect(vi.mocked(tndm)).toHaveBeenCalledWith([
      "ticket",
      "update",
      "TNDM-TEST",
      "--remove-tags",
      "flow:brainstorm,flow:planned,flow:applying,flow:done",
    ]);
    // Then set status and add flow:done
    expect(vi.mocked(tndm)).toHaveBeenCalledWith([
      "ticket",
      "update",
      "TNDM-TEST",
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
