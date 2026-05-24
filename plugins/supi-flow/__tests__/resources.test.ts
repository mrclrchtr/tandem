import { describe, expect, it } from "vitest";

import flowExtension from "../extensions/index";

type RegisteredTool = {
  name: string;
  promptGuidelines?: string[];
};

function setup(): Map<string, (...args: unknown[]) => unknown> {
  const handlers = new Map<string, (...args: unknown[]) => unknown>();
  const tools: RegisteredTool[] = [];
  const pi = {
    on(event: string, handler: (...args: unknown[]) => unknown) {
      handlers.set(event, handler);
    },
    registerTool(tool: RegisteredTool) {
      tools.push(tool);
    },
    registerCommand() {},
  };
  flowExtension(pi as never);
  handlers.set("_tools", () => tools);
  return handlers;
}

function getRegisteredToolEntries(
  handlers: Map<string, (...args: unknown[]) => unknown>,
): RegisteredTool[] {
  const fn = handlers.get("_tools");
  return fn ? (fn() as RegisteredTool[]) : [];
}

function getRegisteredTools(
  handlers: Map<string, (...args: unknown[]) => unknown>,
): string[] {
  return getRegisteredToolEntries(handlers).map((tool) => tool.name);
}

function getRegisteredTool(
  handlers: Map<string, (...args: unknown[]) => unknown>,
  name: string,
): RegisteredTool | undefined {
  return getRegisteredToolEntries(handlers).find((tool) => tool.name === name);
}

describe("supi-flow extension", () => {
  it("does not register a resources_discover handler (resources are discovered via conventional directory discovery)", () => {
    const handlers = setup();
    expect(handlers.has("resources_discover")).toBe(false);
  });

  it("registers all 7 tools", () => {
    const handlers = setup();
    const tools = getRegisteredTools(handlers);
    expect(tools).toContain("supi_tndm_cli");
    expect(tools).toContain("supi_flow_start");
    expect(tools).toContain("supi_flow_plan");
    expect(tools).toContain("supi_flow_apply");
    expect(tools).toContain("supi_flow_task");
    expect(tools).toContain("supi_flow_complete_task");
    expect(tools).toContain("supi_flow_close");
    expect(tools.length).toBe(7);
  });

  it("registers supi_flow_apply guidance that defers task detail doc reads until the active task starts", () => {
    const handlers = setup();
    const tool = getRegisteredTool(handlers, "supi_flow_apply");
    expect(tool).toBeDefined();
    const guidance = tool?.promptGuidelines?.join(" ") ?? "";
    expect(guidance).toContain("load the approved overview and task manifest");
    expect(guidance).toContain("read linked task detail docs only when the active task begins");
  });

  it("registers session_start handler for version check", () => {
    const handlers = setup();
    expect(handlers.has("session_start")).toBe(true);
  });
});
