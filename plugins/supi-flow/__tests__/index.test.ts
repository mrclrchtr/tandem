import { beforeAll, beforeEach, describe, expect, it, vi } from "vitest";
import { execFile } from "node:child_process";

vi.mock("node:child_process", () => ({
  execFile: vi.fn(),
}));

let checkTndmVersion: any;
let FLOW_VERSION: string;

beforeAll(async () => {
  const mod = await import("../extensions/index.js");
  checkTndmVersion = mod.checkTndmVersion;
  FLOW_VERSION = mod.FLOW_VERSION;
});

beforeEach(() => {
  vi.clearAllMocks();
});

describe("checkTndmVersion", () => {
  it("skips check for non-startup/non-reload reasons", async () => {
    const notify = vi.fn();
    for (const reason of ["new", "resume", "fork"] as const) {
      await checkTndmVersion({ reason }, { ui: { notify } });
    }
    expect(notify).not.toHaveBeenCalled();
  });

  it("notifies on version mismatch at startup", async () => {
    vi.mocked(execFile).mockImplementation((_file, _args, _opts, cb) => {
      if (typeof cb === "function") cb(null, "tndm 9.9.9\n", "");
      return {} as never;
    });
    const notify = vi.fn();
    await checkTndmVersion({ reason: "startup" }, { ui: { notify } });
    expect(notify).toHaveBeenCalledWith(
      expect.stringContaining("tndm v9.9.9"),
      "warning",
    );
  });

  it("notifies on version mismatch at reload", async () => {
    vi.mocked(execFile).mockImplementation((_file, _args, _opts, cb) => {
      if (typeof cb === "function") cb(null, "tndm 9.9.9\n", "");
      return {} as never;
    });
    const notify = vi.fn();
    await checkTndmVersion({ reason: "reload" }, { ui: { notify } });
    expect(notify).toHaveBeenCalledWith(
      expect.stringContaining("tndm v9.9.9"),
      "warning",
    );
  });

  it("does not notify when versions match", async () => {
    vi.mocked(execFile).mockImplementation((_file, _args, _opts, cb) => {
      if (typeof cb === "function") cb(null, `tndm ${FLOW_VERSION}\n`, "");
      return {} as never;
    });
    const notify = vi.fn();
    await checkTndmVersion({ reason: "startup" }, { ui: { notify } });
    expect(notify).not.toHaveBeenCalled();
  });

  it("does not notify when tndm is not installed", async () => {
    const enoent = new Error("spawn tndm ENOENT");
    (enoent as NodeJS.ErrnoException).code = "ENOENT";
    vi.mocked(execFile).mockImplementation((_file, _args, _opts, cb) => {
      if (typeof cb === "function") cb(enoent, "", "");
      return {} as never;
    });
    const notify = vi.fn();
    await checkTndmVersion({ reason: "startup" }, { ui: { notify } });
    expect(notify).not.toHaveBeenCalled();
  });
});
