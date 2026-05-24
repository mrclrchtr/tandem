import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";

import { tndmVersion } from "./cli.js";
import { toolSpecs } from "./tools/tool-specs.js";

const baseDir = dirname(dirname(fileURLToPath(import.meta.url)));
const pkg = JSON.parse(readFileSync(join(baseDir, "package.json"), "utf-8"));
export const FLOW_VERSION: string = pkg.version;

/**
 * Check tndm version against supi-flow version. Notifies on mismatch.
 * Exported for testing.
 */
export async function checkTndmVersion(
  event: { reason: string },
  ctx: { ui: { notify: (message: string, type?: "info" | "warning" | "error") => void } },
): Promise<void> {
  if (event.reason !== "startup" && event.reason !== "reload") return;
  const tndmVer = await tndmVersion();
  if (!tndmVer) return;
  if (tndmVer !== FLOW_VERSION) {
    ctx.ui.notify(
      `tndm v${tndmVer} found, but supi-flow expects v${FLOW_VERSION}. ` +
        `Install matching version: brew install mrclrchtr/tap/tndm`,
      "warning",
    );
  }
}

export default function (pi: ExtensionAPI) {
  // ── Version check on startup ────────────────────────────────
  pi.on("session_start", async (event, ctx) => {
    await checkTndmVersion(event, ctx);
  });

  // ── Register tools from shared specs ────────────────────────
  // PI validates schema and execution shape at registration; the generic
  // inference cannot unify different details return types across tools,
  // so we bridge with `as never` and trust the runtime contract.
  for (const spec of toolSpecs) {
    pi.registerTool(spec as never);
  }
}
