import { describe, expect, it, vi } from "vitest";

vi.mock("../extensions/cli.js", () => ({
  tndm: vi.fn(),
  tndmJson: vi.fn(),
}));

const { tndmJson } = await import("../extensions/cli.js");
const { executeTndmCli } = await import("../extensions/tools/tndm-cli.js");

describe("executeTndmCli dispatch", () => {
  it("dispatches show action through the handler table", async () => {
    // Mock for show action
    vi.mocked(tndmJson).mockResolvedValue({
      id: "TNDM-DISPATCH",
      status: "todo",
    });

    const result = await executeTndmCli({
      action: "show",
      id: "TNDM-DISPATCH",
    });

    expect(result.details).toEqual({
      action: "show",
      ticket: { id: "TNDM-DISPATCH", status: "todo" },
    });
  });

  it("throws for unknown action (not in actionEnum)", async () => {
    await expect(
      executeTndmCli({ action: "nonexistent" as never }),
    ).rejects.toThrow('unknown action "nonexistent"');
  });
});
