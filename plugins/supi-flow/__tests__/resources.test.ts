import { describe, expect, it } from "vitest";

import flowExtension from "../extensions/index";

function setup(): Map<string, (...args: unknown[]) => unknown> {
  const handlers = new Map<string, (...args: unknown[]) => unknown>();
  const tools: string[] = [];
  const pi = {
    on(event: string, handler: (...args: unknown[]) => unknown) {
      handlers.set(event, handler);
    },
    registerTool(tool: { name: string }) {
      tools.push(tool.name);
    },
    registerCommand() {},
  };
  flowExtension(pi as never);
  handlers.set("_tools", () => tools);
  return handlers;
}

function getRegisteredTools(
  handlers: Map<string, (...args: unknown[]) => unknown>,
): string[] {
  const fn = handlers.get("_tools");
  return fn ? (fn() as string[]) : [];
}

describe("supi-flow extension", () => {
  it("does not register a resources_discover handler (resources are discovered via conventional directory discovery)", () => {
    const handlers = setup();
    expect(handlers.has("resources_discover")).toBe(false);
  });

  it("registers all 5 tools", () => {
    const handlers = setup();
    const tools = getRegisteredTools(handlers);
    expect(tools).toContain("supi_tndm_cli");
    expect(tools).toContain("supi_flow_start");
    expect(tools).toContain("supi_flow_plan");
    expect(tools).toContain("supi_flow_complete_task");
    expect(tools).toContain("supi_flow_close");
    expect(tools.length).toBe(5);
  });

  it("registers session_start handler for version check", () => {
    const handlers = setup();
    expect(handlers.has("session_start")).toBe(true);
  });
});
