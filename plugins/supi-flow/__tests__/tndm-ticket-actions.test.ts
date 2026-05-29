import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("../extensions/cli.js", () => ({
  tndm: vi.fn(),
  tndmJson: vi.fn(),
}));

const { tndm, tndmJson } = await import("../extensions/cli.js");
const { handleCreate, handleUpdate, handleShow, handleList, handleAwareness } = await import(
  "../extensions/tools/tndm-ticket-actions.js"
);

beforeEach(() => {
  vi.resetAllMocks();
});

// ─── handleCreate ─────────────────────────────────────────────

describe("handleCreate", () => {
  it("creates a ticket with title and returns stdout", async () => {
    vi.mocked(tndm).mockResolvedValue({ stdout: "TNDM-CREATE", stderr: "" });

    const result = await handleCreate({
      action: "create",
      title: "My ticket",
    });

    expect(vi.mocked(tndm)).toHaveBeenCalledWith(
      ["ticket", "create", "My ticket"],
      undefined,
    );
    expect(result.content[0].text).toBe("TNDM-CREATE");
    expect(result.details).toEqual({
      action: "create",
      ticketId: "TNDM-CREATE",
    });
  });

  it("throws when title is missing", async () => {
    await expect(
      handleCreate({ action: "create" }),
    ).rejects.toThrow("title is required");
  });
});

// ─── handleUpdate ─────────────────────────────────────────────

describe("handleUpdate", () => {
  it("updates a ticket with flags", async () => {
    vi.mocked(tndm).mockResolvedValue({ stdout: "", stderr: "" });

    const result = await handleUpdate({
      action: "update",
      id: "TNDM-UPD",
      status: "done",
      priority: "p1",
    });

    expect(vi.mocked(tndm)).toHaveBeenCalledWith(
      ["ticket", "update", "TNDM-UPD", "--status", "done", "--priority", "p1"],
      undefined,
    );
    expect(result.details).toEqual({
      action: "update",
      ticketId: "TNDM-UPD",
    });
  });

  it("throws when id is missing", async () => {
    await expect(
      handleUpdate({ action: "update" }),
    ).rejects.toThrow("id is required");
  });
});

// ─── handleShow ───────────────────────────────────────────────

describe("handleShow", () => {
  it("returns formatted JSON for show", async () => {
    vi.mocked(tndmJson).mockResolvedValue({
      id: "TNDM-SHOW",
      status: "todo",
      title: "Test",
    });

    const result = await handleShow({
      action: "show",
      id: "TNDM-SHOW",
    });

    expect(vi.mocked(tndmJson)).toHaveBeenCalledWith(
      ["ticket", "show", "TNDM-SHOW"],
      undefined,
    );
    expect(result.content[0].text).toContain("TNDM-SHOW");
    expect(result.details).toEqual({
      action: "show",
      ticket: { id: "TNDM-SHOW", status: "todo", title: "Test" },
    });
  });

  it("throws when id is missing", async () => {
    await expect(
      handleShow({ action: "show" }),
    ).rejects.toThrow("id is required");
  });
});

// ─── handleList ───────────────────────────────────────────────

describe("handleList", () => {
  it("handles the ticket list envelope returned by current tndm", async () => {
    const envelope = {
      schema_version: 1,
      tickets: [{ id: "TNDM-LIST", title: "List ticket" }],
    };

    vi.mocked(tndmJson).mockResolvedValue(envelope);

    const result = await handleList({ action: "list" });

    expect(vi.mocked(tndmJson)).toHaveBeenCalledWith(["ticket", "list"], undefined);
    expect(result.content[0].text).toContain("\"tickets\"");
    expect(result.details).toEqual({
      action: "list",
      tickets: envelope.tickets,
      envelope,
    });
  });

  it("handles empty list", async () => {
    vi.mocked(tndmJson).mockResolvedValue([]);

    const result = await handleList({ action: "list" });

    expect(result.content[0].text).toBe("No tickets found.");
    expect(result.details).toEqual({
      action: "list",
      tickets: [],
      envelope: { schema_version: 1, tickets: [] },
    });
  });

  it("passes --all and --definition flags", async () => {
    vi.mocked(tndmJson).mockResolvedValue([]);

    await handleList({
      action: "list",
      all: true,
      definition: "ready",
    });

    expect(vi.mocked(tndmJson)).toHaveBeenCalledWith(
      ["ticket", "list", "--all", "--definition", "ready"],
      undefined,
    );
  });
});

// ─── handleAwareness ──────────────────────────────────────────

describe("handleAwareness", () => {
  it("returns awareness data for a given ref", async () => {
    vi.mocked(tndmJson).mockResolvedValue({
      branch: "main",
      ahead: 3,
      behind: 0,
    });

    const result = await handleAwareness({
      action: "awareness",
      against: "main",
    });

    expect(vi.mocked(tndmJson)).toHaveBeenCalledWith(
      ["awareness", "--against", "main"],
      undefined,
    );
    expect(result.content[0].text).toContain("main");
    expect(result.details).toEqual({
      action: "awareness",
      awareness: { branch: "main", ahead: 3, behind: 0 },
    });
  });

  it("throws when --against is missing", async () => {
    await expect(
      handleAwareness({ action: "awareness" }),
    ).rejects.toThrow("--against is required");
  });
});

// ─── Truncation ───────────────────────────────────────────────

describe("truncation", () => {
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

    const result = await handleShow({
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
