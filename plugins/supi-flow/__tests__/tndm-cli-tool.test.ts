import { beforeEach, describe, expect, it, vi } from "vitest";
import { mkdtempSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";

vi.mock("../extensions/cli.js", () => {
  const mockTndm = vi.fn();
  const mockTndmJson = vi.fn();
  return {
    tndm: mockTndm,
    tndmJson: mockTndmJson,
  };
});

const { tndm, tndmJson } = await import("../extensions/cli.js");
const { executeTndmCli } = await import("../extensions/tools/tndm-cli.js");

beforeEach(() => {
  vi.resetAllMocks();
});

describe("executeTndmCli list", () => {
  it("handles the ticket list envelope returned by current tndm", async () => {
    const envelope = {
      schema_version: 1,
      tickets: [{ id: "TNDM-LIST", title: "List ticket" }],
    };

    vi.mocked(tndmJson).mockResolvedValue(envelope);

    const result = await executeTndmCli({
      action: "list",
    });

    const details = result.details as unknown as { tickets: unknown; envelope: unknown };

    expect(vi.mocked(tndmJson)).toHaveBeenCalledWith(["ticket", "list"]);
    expect(result.content[0].text).toContain("\"tickets\"");
    expect(details.tickets).toEqual(envelope.tickets);
    expect(details.envelope).toEqual(envelope);
  });
});

describe("executeTndmCli task_add", () => {
  it("task_add delegates to Rust CLI without extra detail calls when no detail provided", async () => {
    vi.mocked(tndmJson).mockResolvedValue({ ok: true });

    await executeTndmCli({
      action: "task_add",
      id: "TNDM-ADD",
      task_title: "Simple task",
    });

    expect(vi.mocked(tndmJson)).toHaveBeenCalledTimes(1);
    expect(vi.mocked(tndmJson)).toHaveBeenCalledWith([
      "ticket",
      "task",
      "add",
      "TNDM-ADD",
      "--title",
      "Simple task",
    ]);
    expect(vi.mocked(tndm)).not.toHaveBeenCalled();
  });

  it("creates and links a task detail doc when detail markdown is provided", async () => {
    const tmpDir = mkdtempSync(join(tmpdir(), "tndm-cli-tool-"));
    const docPath = join(tmpDir, "tasks", "task-01.md");
    const finalTicket = {
      schema_version: 1,
      id: "TNDM-DETAIL",
      tasks: [{ number: 1, title: "Detailed task", status: "todo", detail_path: "tasks/task-01.md" }],
    };

    vi.mocked(tndmJson)
      .mockResolvedValueOnce({
        schema_version: 1,
        id: "TNDM-DETAIL",
        tasks: [{ number: 1, title: "Detailed task", status: "todo" }],
      })
      .mockResolvedValueOnce({ path: docPath, detail_path: "tasks/task-01.md" })
      .mockResolvedValueOnce(finalTicket);
    vi.mocked(tndm).mockResolvedValue({ stdout: "", stderr: "" });

    const result = await executeTndmCli({
      action: "task_add",
      id: "TNDM-DETAIL",
      task_title: "Detailed task",
      task_detail: "Implementation notes go here.",
    });

    expect(vi.mocked(tndmJson)).toHaveBeenNthCalledWith(1, [
      "ticket",
      "task",
      "add",
      "TNDM-DETAIL",
      "--title",
      "Detailed task",
    ]);
    expect(vi.mocked(tndmJson)).toHaveBeenNthCalledWith(2, [
      "ticket",
      "task",
      "detail",
      "ensure",
      "TNDM-DETAIL",
      "1",
    ]);
    expect(vi.mocked(tndmJson)).toHaveBeenNthCalledWith(3, [
      "ticket",
      "show",
      "TNDM-DETAIL",
    ]);
    expect(readFileSync(docPath, "utf-8")).toContain("Implementation notes go here.");
    expect(vi.mocked(tndm)).toHaveBeenCalledWith(["ticket", "sync", "TNDM-DETAIL"]);
    expect(result.details.result).toEqual(finalTicket);
    expect(result.content[0].text).toContain("detail_path");
  });

});

describe("executeTndmCli task_edit", () => {
  it("infers task titles from top-level show envelopes when writing detail", async () => {
    const tmpDir = mkdtempSync(join(tmpdir(), "tndm-cli-tool-edit-"));
    const docPath = join(tmpDir, "tasks", "task-02.md");
    const finalTicket = {
      schema_version: 1,
      id: "TNDM-EDITDETAIL",
      tasks: [{ number: 2, title: "Existing task", status: "todo", detail_path: "tasks/task-02.md" }],
    };

    vi.mocked(tndmJson)
      .mockResolvedValueOnce({ path: docPath, detail_path: "tasks/task-02.md" })
      .mockResolvedValueOnce({
        schema_version: 1,
        id: "TNDM-EDITDETAIL",
        tasks: [{ number: 2, title: "Existing task", status: "todo" }],
      })
      .mockResolvedValueOnce(finalTicket);
    vi.mocked(tndm).mockResolvedValue({ stdout: "", stderr: "" });

    const result = await executeTndmCli({
      action: "task_edit",
      id: "TNDM-EDITDETAIL",
      task_number: 2,
      task_detail: "Updated detail body.",
    });

    expect(vi.mocked(tndmJson)).toHaveBeenNthCalledWith(1, [
      "ticket",
      "task",
      "detail",
      "ensure",
      "TNDM-EDITDETAIL",
      "2",
    ]);
    expect(vi.mocked(tndmJson)).toHaveBeenNthCalledWith(2, [
      "ticket",
      "show",
      "TNDM-EDITDETAIL",
    ]);
    expect(vi.mocked(tndmJson)).toHaveBeenNthCalledWith(3, [
      "ticket",
      "show",
      "TNDM-EDITDETAIL",
    ]);
    expect(readFileSync(docPath, "utf-8")).toContain("# Task 2: Existing task");
    expect(readFileSync(docPath, "utf-8")).toContain("Updated detail body.");
    expect(vi.mocked(tndm)).toHaveBeenCalledWith(["ticket", "sync", "TNDM-EDITDETAIL"]);
    expect(result.details.result).toEqual(finalTicket);
  });
});
