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
const { tndm, tndmJson, gitAddCommit } = await import("../src/cli.js");

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

describe("gitAddCommit", () => {
  it("runs git add and git commit, extracts hash", async () => {
    const mock = vi.mocked(execFile);
    mock
      .mockImplementationOnce((_file, _args, _opts, cb) => {
        if (typeof cb === "function") cb(null, "", "");
        return {} as never;
      })
      .mockImplementationOnce((_file, _args, _opts, cb) => {
        if (typeof cb === "function")
          cb(null, "[main abc1234] close TNDM-A1B2C3", "");
        return {} as never;
      });

    const result = await gitAddCommit("close TNDM-A1B2C3");
    expect(result.commitHash).toBe("abc1234");
    expect(mock).toHaveBeenNthCalledWith(
      1,
      "git",
      ["add", ".tndm/"],
      expect.objectContaining({}),
      expect.any(Function),
    );
    expect(mock).toHaveBeenNthCalledWith(
      2,
      "git",
      ["commit", "-m", "close TNDM-A1B2C3"],
      expect.objectContaining({}),
      expect.any(Function),
    );
  });

  it("handles nothing-to-commit gracefully", async () => {
    const mock = vi.mocked(execFile);
    mock
      .mockImplementationOnce((_file, _args, _opts, cb) => {
        if (typeof cb === "function") cb(null, "", "");
        return {} as never;
      })
      .mockImplementationOnce((_file, _args, _opts, cb) => {
        const err = new Error("nothing to commit") as Error & { code: number };
        err.code = 1;
        if (typeof cb === "function") cb(err, "", "");
        return {} as never;
      });

    const result = await gitAddCommit("close TNDM-A1B2C3");
    expect(result.commitHash).toBe("");
  });
});
