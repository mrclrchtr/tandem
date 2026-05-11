import { beforeEach, describe, expect, it, vi } from "vitest";

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

// ─── executeFlowPlan ───────────────────────────────────────────

describe("executeFlowPlan", () => {
  it("stores plan with --add-tags and --remove-tags", async () => {
    vi.mocked(tndm).mockResolvedValue({ stdout: "", stderr: "" });

    await flowTools.executeFlowPlan({
      ticket_id: "TNDM-TEST",
      plan_content: "- [ ] **Task 1**: Do thing",
    });

    expect(vi.mocked(tndm)).toHaveBeenCalledWith([
      "ticket",
      "update",
      "TNDM-TEST",
      "--content",
      "- [ ] **Task 1**: Do thing",
      "--add-tags",
      "flow:planned",
      "--remove-tags",
      "flow:brainstorm",
    ]);
  });

  it("appends to existing content when append=true", async () => {
    vi.mocked(tndmJson).mockResolvedValue({
      id: "TNDM-TEST",
      content: "Existing content",
    });
    vi.mocked(tndm).mockResolvedValue({ stdout: "", stderr: "" });

    await flowTools.executeFlowPlan({
      ticket_id: "TNDM-TEST",
      plan_content: "- [ ] **Task 1**: Do thing",
      append: true,
    });

    const updateCall = vi.mocked(tndm).mock.calls[0][0];
    expect(updateCall.join(" ")).toContain("Existing content");
    expect(updateCall.join(" ")).toContain("Task 1");
  });
});

// ─── executeFlowCompleteTask ───────────────────────────────────

describe("executeFlowCompleteTask", () => {
  it("checks off an unchecked task", async () => {
    vi.mocked(tndmJson).mockResolvedValue({
      id: "TNDM-TEST",
      content: "- [ ] **Task 1**: Do the thing\n- [ ] **Task 2**: Do another",
    });
    vi.mocked(tndm).mockResolvedValue({ stdout: "", stderr: "" });

    const result = await flowTools.executeFlowCompleteTask({
      ticket_id: "TNDM-TEST",
      task_number: 1,
    });

    expect(result.details.completed).toBe(true);
    expect(vi.mocked(tndm)).toHaveBeenCalledWith([
      "ticket",
      "update",
      "TNDM-TEST",
      "--content",
      "- [x] **Task 1**: Do the thing\n- [ ] **Task 2**: Do another",
    ]);
  });

  it("soft-fails when task is already checked off", async () => {
    vi.mocked(tndmJson).mockResolvedValue({
      id: "TNDM-TEST",
      content: "- [x] **Task 1**: Already done",
    });

    const result = await flowTools.executeFlowCompleteTask({
      ticket_id: "TNDM-TEST",
      task_number: 1,
    });

    expect(result.details.completed).toBe(true);
    expect(result.details.skipped).toBe(true);
    // Should NOT call tndm update since no change needed
    expect(vi.mocked(tndm)).not.toHaveBeenCalled();
  });

  it("hard-fails when task number does not exist", async () => {
    vi.mocked(tndmJson).mockResolvedValue({
      id: "TNDM-TEST",
      content: "- [ ] **Task 1**: The only task",
    });

    await expect(
      flowTools.executeFlowCompleteTask({
        ticket_id: "TNDM-TEST",
        task_number: 99,
      }),
    ).rejects.toThrow("Task 99 not found");
  });

  it("soft-fails when ticket has no content", async () => {
    vi.mocked(tndmJson).mockResolvedValue({
      id: "TNDM-TEST",
      content: "",
    });

    const result = await flowTools.executeFlowCompleteTask({
      ticket_id: "TNDM-TEST",
      task_number: 1,
    });

    expect(result.details.error).toBe("No content");
  });
});

// ─── executeFlowClose ──────────────────────────────────────────

describe("executeFlowClose", () => {
  it("uses --add-tags and --remove-tags instead of --tags", async () => {
    vi.mocked(tndmJson).mockResolvedValue({
      id: "TNDM-TEST",
      content: "Done stuff",
    });
    vi.mocked(tndm).mockResolvedValue({ stdout: "", stderr: "" });
    vi.mocked(gitAddCommit).mockResolvedValue({ commitHash: "" });

    await flowTools.executeFlowClose({
      ticket_id: "TNDM-TEST",
    });

    const updateArgs = vi.mocked(tndm).mock.calls[0][0];
    expect(updateArgs).toContain("--add-tags");
    expect(updateArgs).toContain("flow:done");
    expect(updateArgs).toContain("--remove-tags");
    expect(updateArgs).toContain("flow:applying");
    expect(updateArgs).not.toContain("--tags");
  });

  it("appends verification results when no section exists", async () => {
    vi.mocked(tndmJson).mockResolvedValue({
      id: "TNDM-TEST",
      content: "## Context\n\nThe work.",
    });
    vi.mocked(tndm).mockResolvedValue({ stdout: "", stderr: "" });
    vi.mocked(gitAddCommit).mockResolvedValue({ commitHash: "abc123" });

    await flowTools.executeFlowClose({
      ticket_id: "TNDM-TEST",
      verification_results: "All tests pass.",
    });

    const updateArgs = vi.mocked(tndm).mock.calls[0][0];
    const contentIndex = updateArgs.indexOf("--content");
    const content = updateArgs[contentIndex + 1];
    expect(content).toContain("## Verification Results");
    expect(content).toContain("All tests pass.");
  });

  it("updates existing verification results section instead of duplicating", async () => {
    vi.mocked(tndmJson).mockResolvedValue({
      id: "TNDM-TEST",
      content:
        "## Context\n\nWork.\n\n## Verification Results\n\nOld results here.\n\n## Other\n\nMore.",
    });
    vi.mocked(tndm).mockResolvedValue({ stdout: "", stderr: "" });
    vi.mocked(gitAddCommit).mockResolvedValue({ commitHash: "" });

    await flowTools.executeFlowClose({
      ticket_id: "TNDM-TEST",
      verification_results: "New results.",
    });

    const updateArgs = vi.mocked(tndm).mock.calls[0][0];
    const contentIndex = updateArgs.indexOf("--content");
    const content = updateArgs[contentIndex + 1];

    // Should have only one Verification Results section
    expect(content.match(/## Verification Results/g)).toHaveLength(1);
    // Should contain new results, not old
    expect(content).toContain("New results.");
    expect(content).not.toContain("Old results here.");
    // Should preserve sections around it
    expect(content).toContain("## Context");
    expect(content).toContain("## Other");
  });

  it("commits after close", async () => {
    vi.mocked(tndmJson).mockResolvedValue({
      id: "TNDM-TEST",
      content: "Done.",
    });
    vi.mocked(tndm).mockResolvedValue({ stdout: "", stderr: "" });
    vi.mocked(gitAddCommit).mockResolvedValue({ commitHash: "abc123" });

    const result = await flowTools.executeFlowClose({
      ticket_id: "TNDM-TEST",
    });

    expect(vi.mocked(gitAddCommit)).toHaveBeenCalledWith(
      "chore(tndm): close TNDM-TEST",
    );
    expect(result.details.commitHash).toBe("abc123");
  });
});
