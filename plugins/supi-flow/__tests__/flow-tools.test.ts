import { beforeEach, describe, expect, it, vi } from "vitest";
import { readFileSync, writeFileSync } from "node:fs";
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
  it("parses single task and calls task set with correct JSON", async () => {
    vi.mocked(tndmJson).mockResolvedValue({});
    vi.mocked(tndm).mockResolvedValue({ stdout: "", stderr: "" });

    await flowTools.executeFlowPlan({
      ticket_id: "TNDM-TEST",
      plan_content: "- [ ] **Task 1**: Do the thing",
    });

    // Should call task set, not doc create
    expect(vi.mocked(tndmJson)).toHaveBeenCalledWith([
      "ticket",
      "task",
      "set",
      "TNDM-TEST",
      "--tasks",
      JSON.stringify([
        { number: 1, title: "Do the thing", status: "todo" },
      ]),
    ]);

    // Should update tags in a single atomic call
    expect(vi.mocked(tndm)).toHaveBeenCalledWith([
      "ticket",
      "update",
      "TNDM-TEST",
      "--remove-tags",
      "flow:brainstorm,flow:planned,flow:applying,flow:done",
      "--add-tags",
      "flow:planned",
    ]);
  });

  it("parses multiple tasks with file, verification, and notes", async () => {
    vi.mocked(tndmJson).mockResolvedValue({});
    vi.mocked(tndm).mockResolvedValue({ stdout: "", stderr: "" });

    await flowTools.executeFlowPlan({
      ticket_id: "TNDM-MULTI",
      plan_content: `
- [ ] **Task 1**: Create the helper
  - File: src/helper.ts
  - Verification: pnpm exec tsc --noEmit

- [ ] **Task 2**: Add tests
  - File: tests/helper.test.ts
  - Verification: pnpm exec vitest run
  - Notes: Cover edge cases
`,
    });

    expect(vi.mocked(tndmJson)).toHaveBeenCalledWith([
      "ticket",
      "task",
      "set",
      "TNDM-MULTI",
      "--tasks",
      JSON.stringify([
        { number: 1, title: "Create the helper", status: "todo", file: "src/helper.ts", verification: "pnpm exec tsc --noEmit" },
        { number: 2, title: "Add tests", status: "todo", file: "tests/helper.test.ts", verification: "pnpm exec vitest run", notes: "Cover edge cases" },
      ]),
    ]);
  });

  it("strips markdown code ticks from task subfields", async () => {
    vi.mocked(tndmJson).mockResolvedValue({});
    vi.mocked(tndm).mockResolvedValue({ stdout: "", stderr: "" });

    await flowTools.executeFlowPlan({
      ticket_id: "TNDM-TICKS",
      plan_content: `
- [ ] **Task 1**: Normalize values
  - File: \`src/lib.rs\`
  - Verification: \`cargo test\`
  - Notes: \`manual check\`
`,
    });

    expect(vi.mocked(tndmJson)).toHaveBeenCalledWith([
      "ticket",
      "task",
      "set",
      "TNDM-TICKS",
      "--tasks",
      JSON.stringify([
        { number: 1, title: "Normalize values", status: "todo", file: "src/lib.rs", verification: "cargo test", notes: "manual check" },
      ]),
    ]);
  });

  it("parses checked tasks as done", async () => {
    vi.mocked(tndmJson).mockResolvedValue({});
    vi.mocked(tndm).mockResolvedValue({ stdout: "", stderr: "" });

    await flowTools.executeFlowPlan({
      ticket_id: "TNDM-DONE",
      plan_content: "- [x] **Task 1**: Already completed",
    });

    expect(vi.mocked(tndmJson)).toHaveBeenCalledWith([
      "ticket",
      "task",
      "set",
      "TNDM-DONE",
      "--tasks",
      JSON.stringify([
        { number: 1, title: "Already completed", status: "done" },
      ]),
    ]);
  });

  it("rejects empty plan_content instead of silently clearing tasks", async () => {
    vi.mocked(tndmJson).mockResolvedValue({});
    vi.mocked(tndm).mockResolvedValue({ stdout: "", stderr: "" });

    await expect(
      flowTools.executeFlowPlan({
        ticket_id: "TNDM-EMPTY",
        plan_content: "Just some text with no task lines",
      }),
    ).rejects.toThrow(/no \*\*Task N\*\*: lines found/);

    // Should not have called task set or anything else
    expect(vi.mocked(tndmJson)).not.toHaveBeenCalled();
    expect(vi.mocked(tndm)).not.toHaveBeenCalled();
  });

  it("returns task count in details", async () => {
    vi.mocked(tndmJson).mockResolvedValue({});
    vi.mocked(tndm).mockResolvedValue({ stdout: "", stderr: "" });

    const result = await flowTools.executeFlowPlan({
      ticket_id: "TNDM-CNT",
      plan_content: "- [ ] **Task 1**: A\n- [ ] **Task 2**: B\n- [ ] **Task 3**: C",
    });

    expect(result.details.taskCount).toBe(3);
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
