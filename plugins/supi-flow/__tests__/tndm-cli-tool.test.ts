import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("../extensions/cli.js", () => ({
  tndm: vi.fn(),
  tndmJson: vi.fn(),
}));

vi.mock("../extensions/tools/ticket-helpers.js", async () => {
  const actual = await vi.importActual<typeof import("../extensions/tools/ticket-helpers.js")>(
    "../extensions/tools/ticket-helpers.js",
  );
  return {
    ...actual,
    writeTaskDetailAndReload: vi.fn(),
  };
});

const { tndm, tndmJson } = await import("../extensions/cli.js");
const helpers = await import("../extensions/tools/ticket-helpers.js");
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

    expect(vi.mocked(tndmJson)).toHaveBeenCalledWith(["ticket", "list"], undefined);
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
    ], undefined);
    expect(vi.mocked(tndm)).not.toHaveBeenCalled();
    expect(vi.mocked(helpers.writeTaskDetailAndReload)).not.toHaveBeenCalled();
  });

  it("delegates detail-doc writing to shared helper when detail is provided", async () => {
    const addResult = {
      schema_version: 1,
      id: "TNDM-DETAIL",
      tasks: [{ number: 1, title: "Detailed task", status: "todo" }],
    };
    const finalTicket = {
      schema_version: 1,
      id: "TNDM-DETAIL",
      tasks: [{ number: 1, title: "Detailed task", status: "todo", detail_path: "tasks/task-01.md" }],
    };

    vi.mocked(tndmJson).mockResolvedValueOnce(addResult);
    vi.mocked(helpers.writeTaskDetailAndReload).mockResolvedValueOnce(finalTicket);

    const result = await executeTndmCli({
      action: "task_add",
      id: "TNDM-DETAIL",
      task_title: "Detailed task",
      task_detail: "Implementation notes go here.",
    });

    // Step 1: task add via CLI
    expect(vi.mocked(tndmJson)).toHaveBeenNthCalledWith(1, [
      "ticket", "task", "add", "TNDM-DETAIL", "--title", "Detailed task",
    ], undefined);

    // Step 2: shared helper handles ensure → write → sync → reload
    expect(vi.mocked(helpers.writeTaskDetailAndReload)).toHaveBeenCalledWith(
      "TNDM-DETAIL", 1, "Detailed task", "Implementation notes go here.", undefined,
    );

    expect(result.details.result).toEqual(finalTicket);
  });
});

describe("executeTndmCli task_edit", () => {
  it("delegates detail-only edit to shared helper", async () => {
    const existingTicket = {
      schema_version: 1,
      id: "TNDM-EDITDETAIL",
      tasks: [{ number: 2, title: "Existing task", status: "todo" }],
    };
    const finalTicket = {
      schema_version: 1,
      id: "TNDM-EDITDETAIL",
      tasks: [{ number: 2, title: "Existing task", status: "todo", detail_path: "tasks/task-02.md" }],
    };

    // loadTicket to extract title
    vi.mocked(tndmJson).mockResolvedValueOnce(existingTicket);
    vi.mocked(helpers.writeTaskDetailAndReload).mockResolvedValueOnce(finalTicket);

    const result = await executeTndmCli({
      action: "task_edit",
      id: "TNDM-EDITDETAIL",
      task_number: 2,
      task_detail: "Updated detail body.",
    });

    // Step 1: loadTicket to get existing title
    expect(vi.mocked(tndmJson)).toHaveBeenCalledWith(
      ["ticket", "show", "TNDM-EDITDETAIL"], undefined,
    );

    // Step 2: shared helper with applyTitleEdit=false (no title change)
    expect(vi.mocked(helpers.writeTaskDetailAndReload)).toHaveBeenCalledWith(
      "TNDM-EDITDETAIL", 2, "Existing task", "Updated detail body.", undefined, false,
    );

    expect(result.details.result).toEqual(finalTicket);
  });
});

describe("executeTndmCli truncation", () => {
  it("truncates large model-facing output while keeping full details", async () => {
    const largeTasks = Array.from({ length: 2000 }, (_, i) => ({
      number: i + 1,
      title: `Task ${i + 1}` + " x".repeat(30),
      status: "todo",
    }));
    const largePayload = {
      schema_version: 1,
      id: "TNDM-LARGE",
      tasks: largeTasks,
    };

    vi.mocked(tndmJson).mockResolvedValueOnce(largePayload);

    const result = await executeTndmCli({
      action: "show",
      id: "TNDM-LARGE",
    });

    // Content should be truncated
    expect(result.content[0].text).toContain("[Output truncated");

    // Details must keep the full untruncated object
    const details = result.details as Record<string, unknown>;
    expect(details.action).toBe("show");
    expect(details.ticket).toBe(largePayload);
  });
});
