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
    applyTaskMutation: vi.fn(),
  };
});

const { tndm, tndmJson } = await import("../extensions/cli.js");
const helpers = await import("../extensions/tools/ticket-helpers.js");
const { handleTaskAdd, handleTaskEdit, handleTaskRemove, handleTaskComplete, handleTaskSet, handleTaskList } = await import(
  "../extensions/tools/tndm-task-actions.js"
);

beforeEach(() => {
  vi.resetAllMocks();
});

// ─── handleTaskAdd ────────────────────────────────────────────

describe("handleTaskAdd", () => {
  it("task_add delegates to Rust CLI without extra detail calls when no detail provided", async () => {
    vi.mocked(tndmJson).mockResolvedValue({ ok: true });

    await handleTaskAdd({
      action: "task_add",
      id: "TNDM-ADD",
      task_title: "Simple task",
    });

    expect(vi.mocked(tndmJson)).toHaveBeenCalledTimes(1);
    expect(vi.mocked(tndmJson)).toHaveBeenCalledWith(
      ["ticket", "task", "add", "TNDM-ADD", "--title", "Simple task"],
      undefined,
    );
    expect(vi.mocked(helpers.applyTaskMutation)).not.toHaveBeenCalled();
  });

  it("delegates detail-doc writing to applyTaskMutation when detail is provided", async () => {
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
    vi.mocked(helpers.applyTaskMutation).mockResolvedValueOnce(finalTicket);

    const result = await handleTaskAdd({
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
    expect(vi.mocked(helpers.applyTaskMutation)).toHaveBeenCalledWith(
      "TNDM-DETAIL", 1, "Detailed task", "Implementation notes go here.", undefined,
    );

    expect(result.details.result).toEqual(finalTicket);
  });

  it("throws when id is missing", async () => {
    await expect(
      handleTaskAdd({ action: "task_add", task_title: "Test" }),
    ).rejects.toThrow("id is required");
  });

  it("throws when task_title is missing", async () => {
    await expect(
      handleTaskAdd({ action: "task_add", id: "TNDM-NO-TITLE" }),
    ).rejects.toThrow("task_title is required");
  });
});

// ─── handleTaskEdit ───────────────────────────────────────────

describe("handleTaskEdit", () => {
  it("delegates detail-only edit to applyTaskMutation", async () => {
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
    vi.mocked(helpers.applyTaskMutation).mockResolvedValueOnce(finalTicket);

    const result = await handleTaskEdit({
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
    expect(vi.mocked(helpers.applyTaskMutation)).toHaveBeenCalledWith(
      "TNDM-EDITDETAIL", 2, "Existing task", "Updated detail body.", undefined, false,
    );

    expect(result.details.result).toEqual(finalTicket);
  });

  it("calls task edit CLI directly when no detail is provided", async () => {
    vi.mocked(tndmJson).mockResolvedValueOnce({ id: "TNDM-EDITSIMPLE" });

    const result = await handleTaskEdit({
      action: "task_edit",
      id: "TNDM-EDITSIMPLE",
      task_number: 3,
      task_title: "New title",
    });

    expect(vi.mocked(tndmJson)).toHaveBeenCalledWith(
      ["ticket", "task", "edit", "TNDM-EDITSIMPLE", "3", "--title", "New title"],
      undefined,
    );
    expect(vi.mocked(helpers.applyTaskMutation)).not.toHaveBeenCalled();
  });

  it("throws when id is missing", async () => {
    await expect(
      handleTaskEdit({ action: "task_edit", task_number: 1 }),
    ).rejects.toThrow("id is required");
  });

  it("throws when task_number is missing", async () => {
    await expect(
      handleTaskEdit({ action: "task_edit", id: "TNDM-NO-NUM" }),
    ).rejects.toThrow("task_number is required");
  });
});

// ─── handleTaskRemove ─────────────────────────────────────────

describe("handleTaskRemove", () => {
  it("removes one task at a time", async () => {
    vi.mocked(tndmJson).mockResolvedValue({ ok: true });

    const result = await handleTaskRemove({
      action: "task_remove",
      id: "TNDM-REMOVE",
      task_number: 3,
    });

    expect(vi.mocked(tndmJson)).toHaveBeenCalledWith(
      ["ticket", "task", "remove", "TNDM-REMOVE", "3"],
      undefined,
    );
    expect(result.details).toEqual({
      action: "task_remove",
      ticketId: "TNDM-REMOVE",
      taskNumber: 3,
      result: { ok: true },
    });
  });

  it("throws when id is missing", async () => {
    await expect(
      handleTaskRemove({ action: "task_remove", task_number: 1 }),
    ).rejects.toThrow("id is required");
  });

  it("throws when task_number is missing", async () => {
    await expect(
      handleTaskRemove({ action: "task_remove", id: "TNDM-NO-NUM" }),
    ).rejects.toThrow("task_number is required");
  });
});

// ─── handleTaskComplete ───────────────────────────────────────

describe("handleTaskComplete", () => {
  it("calls task complete CLI and returns success", async () => {
    vi.mocked(tndmJson).mockResolvedValue({
      id: "TNDM-COMPLETE",
      tasks: [{ number: 1, title: "Do thing", status: "done" }],
    });

    const result = await handleTaskComplete({
      action: "task_complete",
      id: "TNDM-COMPLETE",
      task_number: 1,
    });

    expect(vi.mocked(tndmJson)).toHaveBeenCalledWith(
      ["ticket", "task", "complete", "TNDM-COMPLETE", "1"],
      undefined,
    );
    expect(result.details).toEqual({
      action: "task_complete",
      ticketId: "TNDM-COMPLETE",
      taskNumber: 1,
      result: {
        id: "TNDM-COMPLETE",
        tasks: [{ number: 1, title: "Do thing", status: "done" }],
      },
    });
  });

  it("throws when id is missing", async () => {
    await expect(
      handleTaskComplete({ action: "task_complete", task_number: 1 }),
    ).rejects.toThrow("id is required");
  });

  it("throws when task_number is missing", async () => {
    await expect(
      handleTaskComplete({ action: "task_complete", id: "TNDM-NO-NUM" }),
    ).rejects.toThrow("task_number is required");
  });
});

// ─── handleTaskSet ────────────────────────────────────────────

describe("handleTaskSet", () => {
  it("calls task set CLI with JSON", async () => {
    vi.mocked(tndmJson).mockResolvedValue({ ok: true });

    const taskJson = JSON.stringify([{ number: 1, title: "Task 1", status: "todo" }]);
    const result = await handleTaskSet({
      action: "task_set",
      id: "TNDM-SET",
      task_json: taskJson,
    });

    expect(vi.mocked(tndmJson)).toHaveBeenCalledWith(
      ["ticket", "task", "set", "TNDM-SET", "--tasks", taskJson],
      undefined,
    );
    expect(result.details.action).toBe("task_set");
  });

  it("throws when id is missing", async () => {
    await expect(
      handleTaskSet({ action: "task_set", task_json: "[]" }),
    ).rejects.toThrow("id is required");
  });

  it("throws when task_json is missing", async () => {
    await expect(
      handleTaskSet({ action: "task_set", id: "TNDM-NO-JSON" }),
    ).rejects.toThrow("task_json is required");
  });
});

// ─── handleTaskList ───────────────────────────────────────────

describe("handleTaskList", () => {
  it("returns task list", async () => {
    const tasks = [
      { number: 1, title: "Task one", status: "todo" },
      { number: 2, title: "Task two", status: "done" },
    ];
    vi.mocked(tndmJson).mockResolvedValue(tasks);

    const result = await handleTaskList({
      action: "task_list",
      id: "TNDM-LIST",
    });

    expect(vi.mocked(tndmJson)).toHaveBeenCalledWith(
      ["ticket", "task", "list", "TNDM-LIST"],
      undefined,
    );
    expect(result.details).toEqual({
      action: "task_list",
      ticketId: "TNDM-LIST",
      tasks,
    });
  });

  it("throws when id is missing", async () => {
    await expect(
      handleTaskList({ action: "task_list" }),
    ).rejects.toThrow("id is required");
  });
});
