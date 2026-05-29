import { mkdirSync, mkdtempSync, realpathSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { beforeEach, describe, expect, it, vi } from "vitest";

// ── Mocks ─────────────────────────────────────────────────────

vi.mock("../extensions/cli.js", () => ({
  tndm: vi.fn(),
  tndmJson: vi.fn(),
  tndmVersion: vi.fn(),
}));

vi.mock("../extensions/tools/doc-writes.js", () => ({
  writeTaskDetailDoc: vi.fn(),
  writeArchiveDoc: vi.fn(),
}));

const { tndm, tndmJson } = await import("../extensions/cli.js");
const { writeTaskDetailDoc } = await import("../extensions/tools/doc-writes.js");
const helpers = await import("../extensions/tools/ticket-helpers.js");

beforeEach(() => {
  vi.clearAllMocks();
});

// ─── findRepoRoot / resolveTicketPath (existing) ──────────────

describe("findRepoRoot", () => {
  it("finds root via .git directory", () => {
    const repoRoot = mkdtempSync(join(tmpdir(), "tndm-helper-git-"));
    const nestedDir = join(repoRoot, "a", "b", "c");

    mkdirSync(join(repoRoot, ".git"));
    mkdirSync(nestedDir, { recursive: true });

    const found = helpers.findRepoRoot(nestedDir);
    expect(found).toBe(repoRoot);
  });

  it("finds root via .tndm directory (no .git)", () => {
    const repoRoot = mkdtempSync(join(tmpdir(), "tndm-helper-tndm-"));
    const nestedDir = join(repoRoot, "x", "y");

    mkdirSync(join(repoRoot, ".tndm"));
    mkdirSync(nestedDir, { recursive: true });

    const found = helpers.findRepoRoot(nestedDir);
    expect(found).toBe(repoRoot);
  });

  it("throws when no root found", () => {
    const emptyDir = mkdtempSync(join(tmpdir(), "tndm-helper-empty-"));

    expect(() => helpers.findRepoRoot(emptyDir)).toThrow(
      "failed to locate repository root",
    );
  });

  it("returns same root on repeated calls", () => {
    const repoRoot = mkdtempSync(join(tmpdir(), "tndm-helper-repeat-"));
    mkdirSync(join(repoRoot, ".tndm"));

    // Both calls walk independently and find the same root
    const first = helpers.findRepoRoot(repoRoot);
    expect(first).toBe(repoRoot);

    const second = helpers.findRepoRoot(repoRoot);
    expect(second).toBe(first);
  });
});

describe("resolveTicketPath", () => {
  it("resolves relative path against repo root", () => {
    const repoRoot = realpathSync(mkdtempSync(join(tmpdir(), "tndm-helper-resolve-")));
    mkdirSync(join(repoRoot, ".git"));
    const originalCwd = process.cwd();

    try {
      process.chdir(repoRoot);
      const relative = ".tndm/tickets/TEST/content.md";
      const absolute = helpers.resolveTicketPath(relative);

      expect(absolute).toBe(join(repoRoot, relative));
    } finally {
      process.chdir(originalCwd);
    }
  });

  it("passes through absolute paths unchanged", () => {
    const absolute = "/tmp/some/absolute/path.md";
    expect(helpers.resolveTicketPath(absolute)).toBe(absolute);
  });
});

// ─── Flow tag constants ───────────────────────────────────────

describe("flow tag constants", () => {
  it("FLOW_TAGS_ALL includes all four flow-state tags", () => {
    expect(helpers.FLOW_TAGS_ALL).toBe(
      "flow:brainstorm,flow:planned,flow:applying,flow:done",
    );
  });

  it("individual tag constants match expected values", () => {
    expect(helpers.FLOW_TAG_BRAINSTORM).toBe("flow:brainstorm");
    expect(helpers.FLOW_TAG_PLANNED).toBe("flow:planned");
    expect(helpers.FLOW_TAG_APPLYING).toBe("flow:applying");
    expect(helpers.FLOW_TAG_DONE).toBe("flow:done");
  });
});

// ─── loadTaskList ─────────────────────────────────────────────

describe("loadTaskList", () => {
  it("delegates to tndmJson task_list and filters returned entries", async () => {
    const rawTasks = [
      { number: 1, title: "Task one", status: "todo" },
      { number: 2, title: "Task two", status: "done" },
      // non-object entry should be filtered out by filterFlowTasks
      "unexpected string",
    ];
    vi.mocked(tndmJson).mockResolvedValueOnce(rawTasks as never);

    const result = await helpers.loadTaskList("TNDM-LL", undefined);

    expect(vi.mocked(tndmJson)).toHaveBeenCalledWith(
      ["ticket", "task", "list", "TNDM-LL"],
      undefined,
    );
    expect(result).toHaveLength(2);
    expect(result[0]).toMatchObject({ number: 1, title: "Task one", status: "todo" });
    expect(result[1]).toMatchObject({ number: 2, title: "Task two", status: "done" });
  });

  it("passes signal through to tndmJson", async () => {
    vi.mocked(tndmJson).mockResolvedValueOnce([] as never);
    const controller = new AbortController();

    await helpers.loadTaskList("TNDM-SIG", controller.signal);

    expect(vi.mocked(tndmJson)).toHaveBeenCalledWith(
      ["ticket", "task", "list", "TNDM-SIG"],
      controller.signal,
    );
  });

  it("throws when tndmJson does not return an array", async () => {
    vi.mocked(tndmJson).mockResolvedValueOnce({ not: "an array" } as never);

    await expect(helpers.loadTaskList("TNDM-BAD")).rejects.toThrow(
      /did not return an array/,
    );
  });

  it("derives detail_path from task number when not present", async () => {
    vi.mocked(tndmJson).mockResolvedValueOnce([
      { number: 5, title: "Five", status: "todo" },
    ] as never);

    const result = await helpers.loadTaskList("TNDM-DP");

    expect(result[0].detail_path).toBe("tasks/task-05.md");
  });
});

// ─── applyTaskMutation ────────────────────────────────────────

describe("applyTaskMutation", () => {
  it("calls ensure → write → sync → reload in order", async () => {
    const detailPath = "/repo/.tndm/tickets/TNDM-ATM/tasks/task-01.md";
    const finalTicket = { id: "TNDM-ATM", tasks: [{ number: 1, title: "Test", status: "todo" }] };

    vi.mocked(tndmJson)
      .mockResolvedValueOnce({ path: detailPath } as never)  // ensureTaskDetailDoc
      .mockResolvedValueOnce(finalTicket as never);            // loadTicket

    const result = await helpers.applyTaskMutation(
      "TNDM-ATM", 1, "Test", "Some detail",
    );

    expect(vi.mocked(tndmJson)).toHaveBeenNthCalledWith(1, [
      "ticket", "task", "detail", "ensure", "TNDM-ATM", "1",
    ], undefined);
    expect(vi.mocked(writeTaskDetailDoc)).toHaveBeenCalledWith(
      detailPath, 1, "Test", "Some detail",
    );
    expect(vi.mocked(tndm)).toHaveBeenCalledWith(
      ["ticket", "sync", "TNDM-ATM"], undefined,
    );
    expect(vi.mocked(tndmJson)).toHaveBeenNthCalledWith(2, [
      "ticket", "show", "TNDM-ATM",
    ], undefined);
    expect(result).toEqual(finalTicket);
  });

  it("does NOT call task edit when applyTitleEdit is false", async () => {
    vi.mocked(tndmJson)
      .mockResolvedValueOnce({ path: "/tmp/path.md" } as never)
      .mockResolvedValueOnce({ id: "TNDM-NE2" } as never);

    await helpers.applyTaskMutation(
      "TNDM-NE2", 1, "Title", "Detail", undefined, false,
    );

    expect(vi.mocked(tndmJson)).toHaveBeenCalledTimes(2);
    expect(vi.mocked(tndmJson)).toHaveBeenNthCalledWith(1, [
      "ticket", "task", "detail", "ensure", "TNDM-NE2", "1",
    ], undefined);
    expect(vi.mocked(tndmJson)).toHaveBeenNthCalledWith(2, [
      "ticket", "show", "TNDM-NE2",
    ], undefined);
  });

  it("calls task edit when applyTitleEdit is true", async () => {
    vi.mocked(tndmJson)
      .mockResolvedValueOnce({ path: "/tmp/path.md" } as never) // ensure
      .mockResolvedValueOnce({ id: "TNDM-TE2" } as never)       // title edit
      .mockResolvedValueOnce({ id: "TNDM-TE2" } as never);      // loadTicket

    await helpers.applyTaskMutation(
      "TNDM-TE2", 3, "Updated title", "Detail", undefined, true,
    );

    expect(vi.mocked(tndmJson)).toHaveBeenNthCalledWith(2, [
      "ticket", "task", "edit", "TNDM-TE2", "3", "--title", "Updated title",
    ], undefined);
  });

  it("passes signal to every internal call", async () => {
    const controller = new AbortController();
    const signal = controller.signal;

    vi.mocked(tndmJson)
      .mockResolvedValueOnce({ path: "/tmp/sig.md" } as never)
      .mockResolvedValueOnce({ id: "TNDM-SIG2" } as never);

    await helpers.applyTaskMutation(
      "TNDM-SIG2", 1, "Sig", "Detail", signal,
    );

    expect(vi.mocked(tndmJson)).toHaveBeenNthCalledWith(1, [
      "ticket", "task", "detail", "ensure", "TNDM-SIG2", "1",
    ], signal);
    expect(vi.mocked(tndm)).toHaveBeenCalledWith(
      ["ticket", "sync", "TNDM-SIG2"], signal,
    );
    expect(vi.mocked(tndmJson)).toHaveBeenNthCalledWith(2, [
      "ticket", "show", "TNDM-SIG2",
    ], signal);
  });
});
