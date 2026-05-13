import { execFile } from "node:child_process";

type ExecResult = { stdout: string; stderr: string };

function toString(data: string | Buffer): string {
  return typeof data === "string" ? data : data.toString("utf-8");
}

async function run(
  file: string,
  args: string[],
  options?: { maxBuffer?: number; timeout?: number },
): Promise<ExecResult> {
  return new Promise<ExecResult>((resolve, reject) => {
    const child = execFile(file, args, options, (error, stdout, stderr) => {
      if (error) {
        const msg = toString(stderr).trim() || error.message;
        const wrapped = new Error(`"${file} ${args.join(" ")}" failed: ${msg}`);
        // Preserve ENOENT and similar system error codes for callers
        const errno = error as NodeJS.ErrnoException;
        if (errno.code) {
          (wrapped as NodeJS.ErrnoException).code = errno.code;
        }
        reject(wrapped);
        return;
      }
      resolve({
        stdout: toString(stdout).trim(),
        stderr: toString(stderr).trim(),
      });
    });
  });
}

/**
 * Run a tndm subcommand and return stdout/stderr.
 * Throws on non-zero exit, timeout, or other exec error.
 */
export async function tndm(args: string[]): Promise<ExecResult> {
  try {
    return await run("tndm", args, { timeout: 30_000 });
  } catch (error) {
    if (
      error instanceof Error &&
      (error as NodeJS.ErrnoException).code === "ENOENT"
    ) {
      throw new Error(
        "tndm is not installed or not on your PATH.\n\n" +
          "Install it with one of:\n" +
          "  brew install mrclrchtr/tap/tndm\n" +
          "  cargo install tandem-cli\n" +
          "  curl -LsSf https://github.com/mrclrchtr/tandem/releases/latest/download/tandem-cli-installer.sh | sh\n",
      );
    }
    throw error;
  }
}

/**
 * Run tndm --version and return the parsed semver string, or null if unavailable.
 * Never throws — callers handle absence gracefully.
 */
export async function tndmVersion(): Promise<string | null> {
  try {
    const { stdout } = await run("tndm", ["--version"], { timeout: 5_000 });
    const match = stdout.match(/tndm\s+(\d+\.\d+\.\d+)/);
    return match ? match[1] : null;
  } catch {
    return null;
  }
}

/**
 * Run a tndm subcommand with `--json` and parse the structured output.
 * Throws if exit is non-zero or JSON is invalid.
 */
export async function tndmJson<T = Record<string, unknown>>(
  args: string[],
): Promise<T> {
  const { stdout } = await tndm([...args, "--json"]);
  if (!stdout) {
    throw new Error(`tndm returned empty output for: ${args.join(" ")}`);
  }
  try {
    return JSON.parse(stdout) as T;
  } catch {
    throw new Error(
      `tndm returned invalid JSON for: ${args.join(" ")}\nOutput: ${stdout}`,
    );
  }
}
