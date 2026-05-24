import { mkdir, writeFile } from "node:fs/promises";
import { dirname } from "node:path";
import { withFileMutationQueue } from "@earendil-works/pi-coding-agent";

/**
 * Write a canonical task detail doc at the real path returned by `tndm`,
 * participating in PI's file-mutation queue so concurrent tool calls
 * writing to the same file are serialized correctly.
 */
export async function writeTaskDetailDoc(
  path: string,
  taskNumber: number,
  title: string,
  detail: string,
): Promise<void> {
  return withFileMutationQueue(path, async () => {
    await mkdir(dirname(path), { recursive: true });
    await writeFile(path, `# Task ${taskNumber}: ${title}\n\n${detail}\n`, "utf-8");
  });
}

/**
 * Write an archive.md doc at the real path returned by `tndm`,
 * participating in PI's file-mutation queue.
 */
export async function writeArchiveDoc(
  path: string,
  verificationResults: string,
): Promise<void> {
  return withFileMutationQueue(path, async () => {
    await mkdir(dirname(path), { recursive: true });
    await writeFile(path, `# Archive\n\n${verificationResults}\n`, "utf-8");
  });
}
