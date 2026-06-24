import { describe, expect, it } from "vitest";

import flowExtension from "../extensions/index";

type ToolParameter = {
  description?: string;
};

type RegisteredTool = {
  name: string;
  description?: string;
  promptSnippet?: string;
  promptGuidelines?: string[];
  executionMode?: string;
  parameters?: {
    properties?: Record<string, ToolParameter>;
  };
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

function getParameterDescription(
  tool: RegisteredTool | undefined,
  name: string,
): string | undefined {
  return tool?.parameters?.properties?.[name]?.description;
}

describe("supi-flow extension", () => {
  it("does not register a resources_discover handler (resources are discovered via conventional directory discovery)", () => {
    const handlers = setup();
    expect(handlers.has("resources_discover")).toBe(false);
  });

  it("registers supi_tndm_cli with sequential execution mode", () => {
    const handlers = setup();
    const tool = getRegisteredTool(handlers, "supi_tndm_cli");
    expect(tool).toBeDefined();
    expect(tool!.executionMode).toBe("sequential");
  });

  it("registers all 7 tools and keeps prompt snippets on each one", () => {
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

    for (const tool of getRegisteredToolEntries(handlers)) {
      expect(tool.promptSnippet).toBeTruthy();
    }
  });

  it("registers compact routing-first tool snippets and descriptions", () => {
    const handlers = setup();

    expect(getRegisteredTool(handlers, "supi_tndm_cli")).toMatchObject({
      description:
        "Run tndm ticket/task operations instead of shelling out. " +
        "Prefer supi_flow_task for task authoring; use task_* only for repair.",
      promptSnippet: "Run direct tndm ticket/task operations",
    });

    expect(getRegisteredTool(handlers, "supi_flow_start")).toMatchObject({
      description:
        "Create a ticket for non-trivial work (status=todo, tag flow:brainstorm). Writes context to content.md when given. Returns ticket id.",
      promptSnippet: "Create a brainstorm ticket for non-trivial work",
    });

    expect(getRegisteredTool(handlers, "supi_flow_plan")).toMatchObject({
      description:
        "Store the approved overview in content.md and set flow:planned. plan_content must be non-blank. Author tasks then with supi_flow_task.",
      promptSnippet: "Store the approved overview for a flow ticket",
    });

    expect(getRegisteredTool(handlers, "supi_flow_apply")).toMatchObject({
      description:
        "Use when entering the apply phase. Load the approved overview and task manifest. Moves planned tickets into applying.",
      promptSnippet: "Enter apply for an approved flow ticket",
    });

    expect(getRegisteredTool(handlers, "supi_flow_task")).toMatchObject({
      description:
        "Use when authoring or reconciling tasks in a plan. Add, edit, or remove one structured task in a flow ticket. Writes task detail doc when detail is given.",
      promptSnippet: "Manage one structured task in a flow ticket",
    });

    expect(getRegisteredTool(handlers, "supi_flow_complete_task")).toMatchObject({
      description:
        "Mark a verified task done by its 1-based task number. Throws if the task does not exist in the ticket.",
      promptSnippet: "Mark one verified task done in a flow ticket",
    });

    expect(getRegisteredTool(handlers, "supi_flow_close")).toMatchObject({
      description:
        "Use at archive closeout. Close a completed flow ticket with verification evidence, set status=done + flow:done.",
      promptSnippet: "Close a completed flow ticket with evidence",
    });
  });

  it("keeps compact parameter descriptions for the required prompt distinctions", () => {
    const handlers = setup();
    const tndmTool = getRegisteredTool(handlers, "supi_tndm_cli");
    const planTool = getRegisteredTool(handlers, "supi_flow_plan");
    const closeTool = getRegisteredTool(handlers, "supi_flow_close");

    expect(getParameterDescription(tndmTool, "tags")).toBe("Comma-separated tags to replace");
    expect(getParameterDescription(tndmTool, "add_tags")).toBe("Comma-separated tags to add");
    expect(getParameterDescription(tndmTool, "remove_tags")).toBe(
      "Comma-separated tags to remove",
    );
    expect(getParameterDescription(planTool, "plan_content")).toBe(
      "Approved overview markdown for content.md",
    );
    expect(getParameterDescription(closeTool, "verification_results")).toBe(
      "Verification evidence for archive.md",
    );
  });

  it("keeps promptGuidelines minimal and tool-named (only additive routing)", () => {
    const handlers = setup();
    expect(getRegisteredTool(handlers, "supi_tndm_cli")?.promptGuidelines).toEqual([]);
    expect(getRegisteredTool(handlers, "supi_flow_start")?.promptGuidelines).toEqual([
      "Do not use supi_flow_start when the user explicitly wants direct implementation.",
    ]);
    expect(getRegisteredTool(handlers, "supi_flow_plan")?.promptGuidelines).toEqual([]);
    expect(getRegisteredTool(handlers, "supi_flow_apply")?.promptGuidelines).toEqual([]);
    expect(getRegisteredTool(handlers, "supi_flow_task")?.promptGuidelines).toEqual([]);
    expect(getRegisteredTool(handlers, "supi_flow_complete_task")?.promptGuidelines).toEqual([]);
    expect(getRegisteredTool(handlers, "supi_flow_close")?.promptGuidelines).toEqual([]);
  });

  it("registers session_start handler for version check", () => {
    const handlers = setup();
    expect(handlers.has("session_start")).toBe(true);
  });
});
