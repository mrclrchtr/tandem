import { beforeEach, describe, expect, it, vi } from "vitest";
import { execFile } from "node:child_process";

// Mock execFile before importing the module under test
vi.mock("node:child_process", () => {
  const mockExecFile = vi.fn();
  return {
    execFile: mockExecFile,
  };
});

// Dynamic import so mocks are set up first
const { tndm, tndmJson, tndmVersion } = await import("../extensions/cli.js");

beforeEach(() => {
  vi.clearAllMocks();
});

describe("tndm", () => {
  it("passes args to execFile and returns trimmed stdout/stderr", async () => {
    const mock = vi.mocked(execFile);
    mock.mockImplementation((_file, _args, _opts, cb) => {
      if (typeof cb === "function") {
        cb(null, "hello\nworld\n", "  error  ");
      }
      return {} as never;
    });

    const result = await tndm(["ticket", "list", "--json"]);
    expect(result.stdout).toBe("hello\nworld");
    expect(result.stderr).toBe("error");
    expect(mock).toHaveBeenCalledWith(
      "tndm",
      ["ticket", "list", "--json"],
      expect.objectContaining({}),
      expect.any(Function),
    );
  });

  it("throws on non-zero exit", async () => {
    const mock = vi.mocked(execFile);
    mock.mockImplementation((_file, _args, _opts, cb) => {
      if (typeof cb === "function") {
        cb(new Error("command failed"), "", "stderr output");
      }
      return {} as never;
    });

    await expect(tndm(["bad", "command"])).rejects.toThrow(
      '"tndm bad command" failed: stderr output',
    );
  });

  it("falls back to error.message when stderr is empty", async () => {
    const mock = vi.mocked(execFile);
    mock.mockImplementation((_file, _args, _opts, cb) => {
      if (typeof cb === "function") {
        cb(new Error("some error"), "", "");
      }
      return {} as never;
    });

    await expect(tndm(["bad"])).rejects.toThrow(
      '"tndm bad" failed: some error',
    );
  });

  it("gives helpful message when tndm is not found (ENOENT)", async () => {
    const mock = vi.mocked(execFile);
    const enoent = new Error("spawn tndm ENOENT");
    (enoent as NodeJS.ErrnoException).code = "ENOENT";
    mock.mockImplementation((_file, _args, _opts, cb) => {
      if (typeof cb === "function") {
        cb(enoent, "", "");
      }
      return {} as never;
    });

    await expect(tndm(["ticket", "list"])).rejects.toThrow(
      /tndm is not installed or not on your PATH/,
    );
  });
});

describe("tndmVersion", () => {
  it("parses version from tndm --version output", async () => {
    const mock = vi.mocked(execFile);
    mock.mockImplementation((_file, _args, _opts, cb) => {
      if (typeof cb === "function") cb(null, "tndm 0.9.0\n", "");
      return {} as never;
    });

    await expect(tndmVersion()).resolves.toBe("0.9.0");
  });

  it("returns null when output does not match version pattern", async () => {
    const mock = vi.mocked(execFile);
    mock.mockImplementation((_file, _args, _opts, cb) => {
      if (typeof cb === "function") cb(null, "unexpected output", "");
      return {} as never;
    });

    await expect(tndmVersion()).resolves.toBeNull();
  });

  it("returns null when tndm is not found (ENOENT)", async () => {
    const mock = vi.mocked(execFile);
    const enoent = new Error("spawn tndm ENOENT");
    (enoent as NodeJS.ErrnoException).code = "ENOENT";
    mock.mockImplementation((_file, _args, _opts, cb) => {
      if (typeof cb === "function") cb(enoent, "", "");
      return {} as never;
    });

    await expect(tndmVersion()).resolves.toBeNull();
  });
});

describe("tndmJson", () => {
  it("parses JSON from tndm --json output", async () => {
    const mock = vi.mocked(execFile);
    mock.mockImplementation((_file, _args, _opts, cb) => {
      if (typeof cb === "function") {
        cb(null, '{"id":"TNDM-A1B2C3","status":"todo"}', "");
      }
      return {} as never;
    });

    const result = await tndmJson<{ id: string; status: string }>([
      "ticket",
      "show",
      "TNDM-A1B2C3",
    ]);
    expect(result).toEqual({ id: "TNDM-A1B2C3", status: "todo" });
  });

  it("appends --json to args", async () => {
    const mock = vi.mocked(execFile);
    mock.mockImplementation((_file, _args, _opts, cb) => {
      if (typeof cb === "function") {
        cb(null, "{}", "");
      }
      return {} as never;
    });

    await tndmJson(["ticket", "list"]);
    expect(mock).toHaveBeenCalledWith(
      "tndm",
      ["ticket", "list", "--json"],
      expect.objectContaining({}),
      expect.any(Function),
    );
  });

  it("throws on empty output", async () => {
    const mock = vi.mocked(execFile);
    mock.mockImplementation((_file, _args, _opts, cb) => {
      if (typeof cb === "function") cb(null, "", "");
      return {} as never;
    });

    await expect(tndmJson(["ticket", "show", "X"])).rejects.toThrow(
      "tndm returned empty output for: ticket show X",
    );
  });

  it("throws on invalid JSON", async () => {
    const mock = vi.mocked(execFile);
    mock.mockImplementation((_file, _args, _opts, cb) => {
      if (typeof cb === "function") cb(null, "not json", "");
      return {} as never;
    });

    await expect(tndmJson(["ticket", "show", "X"])).rejects.toThrow(
      "tndm returned invalid JSON for: ticket show X",
    );
  });
});
