import { beforeEach, describe, expect, it, vi } from "vitest";
import { execFile } from "node:child_process";

vi.mock("node:child_process", () => ({
  execFile: vi.fn(),
}));

type CommandHandler = (args: string[], ctx: {
  ui: { notify: (message: string, level: string) => void };
  sessionManager: { getBranch: () => Array<{ type: string; message?: { role?: string; content?: unknown } }> };
}) => Promise<void>;

type RegisteredCommand = {
  description: string;
  handler: CommandHandler;
};

function makePi() {
  const commands = new Map<string, RegisteredCommand>();
  return {
    commands,
    pi: {
      on() {},
      registerTool() {},
      registerCommand(name: string, command: RegisteredCommand) {
        commands.set(name, command);
      },
    },
  };
}

async function loadExtension() {
  const mod = await import("../src/index.js");
  const { pi, commands } = makePi();
  mod.default(pi as never);
  return commands;
}

beforeEach(() => {
  vi.clearAllMocks();
});

describe("supi-flow commands", () => {
  it("/supi-flow-status reports active flow tickets from TNDM ticket data", async () => {
    vi.mocked(execFile).mockImplementation((_file, _args, _opts, cb) => {
      if (typeof cb === "function") {
        cb(
          null,
          JSON.stringify([
            {
              id: "TNDM-PLAN1",
              status: "todo",
              tags: ["flow:planned"],
            },
            {
              id: "TNDM-APPLY1",
              status: "in_progress",
              tags: ["flow:applying"],
            },
            {
              id: "TNDM-DONE01",
              status: "done",
              tags: ["flow:done"],
            },
          ]),
          "",
        );
      }
      return {} as never;
    });

    const commands = await loadExtension();
    const notify = vi.fn();

    await commands.get("supi-flow-status")?.handler([], {
      ui: { notify },
      sessionManager: { getBranch: () => [] },
    });

    expect(notify).toHaveBeenCalledWith(
      expect.stringContaining("TNDM-PLAN1"),
      "info",
    );
    expect(notify).toHaveBeenCalledWith(
      expect.stringContaining("TNDM-APPLY1"),
      "info",
    );
    expect(notify).not.toHaveBeenCalledWith(
      expect.stringContaining("TNDM-DONE01"),
      "info",
    );
  });

  it("/supi-flow help mentions the status command", async () => {
    const commands = await loadExtension();
    const notify = vi.fn();

    await commands.get("supi-flow")?.handler([], {
      ui: { notify },
      sessionManager: { getBranch: () => [] },
    });

    expect(notify).toHaveBeenCalledWith(
      expect.stringContaining("/supi-flow-status"),
      "info",
    );
  });
});
