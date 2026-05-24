/**
 * Integration tests for supi-flow against the real `tndm` CLI.
 *
 * Gated by TNDM_INTEGRATION_TEST=1 env var.
 * Requires `tndm` and `git` on PATH.
 *
 * Usage: TNDM_INTEGRATION_TEST=1 pnpm exec vitest run __tests__/integration.test.ts
 */

import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { execFileSync } from "node:child_process";
import { afterAll, beforeAll, describe, expect, it } from "vitest";

// Gate: only run when TNDM_INTEGRATION_TEST env var is set.
// Set via vitest.config.ts `env` or shell env when using --pool forks.
const describeIntegration =
  process.env.TNDM_INTEGRATION_TEST === "1" ? describe : describe.skip;

let repoRoot: string;

describeIntegration("supi-flow integration", () => {
  beforeAll(() => {
    repoRoot = mkdtempSync(join(tmpdir(), "supi-flow-int-"));
    process.chdir(repoRoot);
    execFileSync("git", ["init"], { stdio: "pipe" });
    // .tndm/ directory is auto-created by tndm ticket operations
  });

  afterAll(() => {
    rmSync(repoRoot, { recursive: true, force: true });
  });

  it("creates a ticket via executeFlowStart", async () => {
    const { executeFlowStart } = await import("../extensions/tools/flow-tools.js");

    const result = await executeFlowStart({ title: "Integration test ticket" });

    expect(result.details).toMatchObject({
      action: "flow_start",
      status: "todo",
      tags: "flow:brainstorm",
    });
    expect(result.details.ticketId).toMatch(/^TNDM-/);
  });

  it("adds a task with detail via executeTndmCli and verifies file on disk", async () => {
    const { executeFlowStart } = await import("../extensions/tools/flow-tools.js");
    const { executeTndmCli } = await import("../extensions/tools/tndm-cli.js");
    const { existsSync, readFileSync } = await import("node:fs");
    const { join } = await import("node:path");

    const startResult = await executeFlowStart({ title: "Detail doc test" });
    const ticketId = startResult.details.ticketId as string;

    const addResult = await executeTndmCli({
      action: "task_add",
      id: ticketId,
      task_title: "Test task with detail",
      task_detail: "## Implementation\n\nWrite the thing.",
    });

    const tasks = (addResult.details.result as { tasks?: unknown[] }).tasks;
    expect(Array.isArray(tasks)).toBe(true);
    expect(tasks![0]).toMatchObject({ number: 1, title: "Test task with detail" });

    // Verify the detail file exists on disk
    const detailPath = join(repoRoot, ".tndm", "tickets", ticketId, "tasks", "task-01.md");
    expect(existsSync(detailPath)).toBe(true);
    expect(readFileSync(detailPath, "utf-8")).toContain("Write the thing.");
  });

  it("lists tasks via executeTndmCli with correct shape", async () => {
    const { executeFlowStart } = await import("../extensions/tools/flow-tools.js");
    const { executeTndmCli } = await import("../extensions/tools/tndm-cli.js");

    const startResult = await executeFlowStart({ title: "Task list test" });
    const ticketId = startResult.details.ticketId as string;

    await executeTndmCli({
      action: "task_add",
      id: ticketId,
      task_title: "First task",
    });
    await executeTndmCli({
      action: "task_add",
      id: ticketId,
      task_title: "Second task",
    });

    const listResult = await executeTndmCli({
      action: "task_list",
      id: ticketId,
    });

    const tasks = (listResult.details as { tasks: unknown[] }).tasks;
    expect(Array.isArray(tasks)).toBe(true);
    expect(tasks).toHaveLength(2);
    expect(tasks[0]).toMatchObject({ number: 1, title: "First task", status: "todo" });
    expect(tasks[1]).toMatchObject({ number: 2, title: "Second task", status: "todo" });
  });

  it("edits a task detail via executeFlowTask", async () => {
    const { executeFlowStart } = await import("../extensions/tools/flow-tools.js");
    const { executeTndmCli } = await import("../extensions/tools/tndm-cli.js");
    const { readFileSync, existsSync } = await import("node:fs");
    const { join } = await import("node:path");

    const startResult = await executeFlowStart({ title: "Edit detail test" });
    const ticketId = startResult.details.ticketId as string;

    await executeTndmCli({
      action: "task_add",
      id: ticketId,
      task_title: "Editable task",
      task_detail: "Original content.",
    });

    // Wait for file to be written by getTaskDetailDoc
    // The file was written during task_add above. Now edit via executeFlowTask
    const { executeFlowTask } = await import("../extensions/tools/flow-tools.js");

    await executeFlowTask({
      ticket_id: ticketId,
      operation: "edit",
      task_number: 1,
      detail: "Revised content for the task.",
    });

    const detailPath = join(repoRoot, ".tndm", "tickets", ticketId, "tasks", "task-01.md");
    expect(existsSync(detailPath)).toBe(true);
    const content = readFileSync(detailPath, "utf-8");
    expect(content).toContain("Revised content");
    expect(content).not.toContain("Original content");
  });

  it("completes a task and verifies status change", async () => {
    const { executeFlowStart } = await import("../extensions/tools/flow-tools.js");
    const { executeTndmCli } = await import("../extensions/tools/tndm-cli.js");

    const startResult = await executeFlowStart({ title: "Complete task test" });
    const ticketId = startResult.details.ticketId as string;

    await executeTndmCli({
      action: "task_add",
      id: ticketId,
      task_title: "Completable task",
    });

    const { executeFlowCompleteTask } = await import("../extensions/tools/flow-tools.js");

    const completeResult = await executeFlowCompleteTask({
      ticket_id: ticketId,
      task_number: 1,
    });

    expect(completeResult.details.completed).toBe(true);

    // Verify via task_list
    const listResult = await executeTndmCli({
      action: "task_list",
      id: ticketId,
    });
    const tasks = (listResult.details as { tasks: unknown[] }).tasks;
    expect(tasks[0]).toMatchObject({ number: 1, status: "done" });
  });

  it("closes a flow with verification results and verifies archive.md", async () => {
    const { executeFlowStart } = await import("../extensions/tools/flow-tools.js");
    const { executeTndmCli } = await import("../extensions/tools/tndm-cli.js");
    const { readFileSync, existsSync } = await import("node:fs");
    const { join } = await import("node:path");

    const startResult = await executeFlowStart({ title: "Close flow test" });
    const ticketId = startResult.details.ticketId as string;

    // Add a task with detail
    await executeTndmCli({
      action: "task_add",
      id: ticketId,
      task_title: "Closable task",
    });

    // Complete the task
    await executeTndmCli({
      action: "task_complete",
      id: ticketId,
      task_number: 1,
    });

    // Write content.md and transition to flow:applying (simulating executeFlowApply)
    const { writeFileSync, mkdirSync } = await import("node:fs");
    const ticketDir = join(repoRoot, ".tndm", "tickets", ticketId);
    mkdirSync(ticketDir, { recursive: true });
    writeFileSync(join(ticketDir, "content.md"), "# Overview\n\nClose this flow.", "utf-8");

    const { tndm } = await import("../extensions/cli.js");
    await tndm([
      "ticket", "update", ticketId,
      "--status", "in_progress",
      "--add-tags", "flow:applying",
      "--remove-tags", "flow:brainstorm",
    ]);

    const { executeFlowClose } = await import("../extensions/tools/flow-tools.js");

    const closeResult = await executeFlowClose({
      ticket_id: ticketId,
      verification_results: "All integration tests passed.",
    });

    expect(closeResult.details.status).toBe("done");

    // Verify archive.md on disk
    const archivePath = join(ticketDir, "archive.md");
    expect(existsSync(archivePath)).toBe(true);
    expect(readFileSync(archivePath, "utf-8")).toContain("All integration tests passed.");

    // Verify ticket is done
    const showResult = await executeTndmCli({
      action: "show",
      id: ticketId,
    });
    const ticketData = (showResult.details as { ticket: Record<string, unknown> }).ticket;
    expect(ticketData.status).toBe("done");
    expect((ticketData.tags as string[])).toContain("flow:done");
  });
});
