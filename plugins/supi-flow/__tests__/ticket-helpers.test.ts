import { mkdirSync, mkdtempSync, realpathSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { afterEach, beforeEach, describe, expect, it } from "vitest";

// Need to import from the module; we use dynamic imports to ensure
// the cache is fresh each time, and rely on _resetRepoRootCache to
// clear between tests.
import {
  _resetRepoRootCache,
  findRepoRoot,
  resolveTicketPath,
} from "../extensions/tools/ticket-helpers.js";

beforeEach(() => {
  _resetRepoRootCache();
});

describe("findRepoRoot", () => {
  it("finds root via .git directory", () => {
    const repoRoot = mkdtempSync(join(tmpdir(), "tndm-helper-git-"));
    const nestedDir = join(repoRoot, "a", "b", "c");

    mkdirSync(join(repoRoot, ".git"));
    mkdirSync(nestedDir, { recursive: true });

    const found = findRepoRoot(nestedDir);
    expect(found).toBe(repoRoot);
  });

  it("finds root via .tndm directory (no .git)", () => {
    const repoRoot = mkdtempSync(join(tmpdir(), "tndm-helper-tndm-"));
    const nestedDir = join(repoRoot, "x", "y");

    mkdirSync(join(repoRoot, ".tndm"));
    mkdirSync(nestedDir, { recursive: true });

    const found = findRepoRoot(nestedDir);
    expect(found).toBe(repoRoot);
  });

  it("throws when no root found", () => {
    const emptyDir = mkdtempSync(join(tmpdir(), "tndm-helper-empty-"));

    expect(() => findRepoRoot(emptyDir)).toThrow(
      "failed to locate repository root",
    );
  });

  it("caches result across calls", () => {
    const repoRoot = mkdtempSync(join(tmpdir(), "tndm-helper-cache-"));
    mkdirSync(join(repoRoot, ".tndm"));

    // First call populates the cache
    const first = findRepoRoot(repoRoot);
    expect(first).toBe(repoRoot);

    // Second call returns cached value (doesn't re-traverse)
    const second = findRepoRoot(repoRoot);
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
      // Reset cache so findRepoRoot picks up the temp dir
      _resetRepoRootCache();

      const relative = ".tndm/tickets/TEST/content.md";
      const absolute = resolveTicketPath(relative);

      expect(absolute).toBe(join(repoRoot, relative));
    } finally {
      process.chdir(originalCwd);
      _resetRepoRootCache();
    }
  });

  it("passes through absolute paths unchanged", () => {
    const absolute = "/tmp/some/absolute/path.md";
    expect(resolveTicketPath(absolute)).toBe(absolute);
  });
});
