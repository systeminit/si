import { basename, join } from "@std/path";
import { Context } from "../context.ts";
import { unknownValueToErrorMessage } from "../helpers.ts";

export function isRemoteUrl(template: string): boolean {
  return /^https?:\/\//i.test(template);
}

export async function downloadRemoteTemplate(url: string): Promise<string> {
  const ctx = Context.instance();

  if (!isRemoteUrl(url)) {
    throw new Error(
      `Invalid URL format: ${url}. URL must start with http:// or https://`,
    );
  }

  ctx.logger.info(`Downloading remote template from: {url}`, { url });

  try {
    const response = await fetch(url);

    if (!response.ok) {
      throw new Error(
        `Failed to download template: HTTP ${response.status} ${response.statusText}`,
      );
    }

    const content = await response.text();
    const urlPath = new URL(url).pathname;
    const originalFilename = basename(urlPath);
    const extension = originalFilename.match(/\.(ts|js|tsx|jsx)$/i)?.[0] ||
      ".ts";

    const tempDir = await Deno.makeTempDir({ prefix: "si-template-" });
    const timestamp = Date.now();
    const tempFilePath = join(
      tempDir,
      `template-${timestamp}${extension}`,
    );

    await Deno.writeTextFile(tempFilePath, content);

    ctx.logger.info(
      `Template downloaded successfully to: {path}`,
      { path: tempFilePath },
    );

    return tempFilePath;
  } catch (error) {
    ctx.logger.error(
      `Failed to download remote template: {message}`,
      {
        message: error instanceof Error ? error.message : String(error),
      },
    );
    throw error;
  }
}

export async function cleanupDownloadedTemplate(
  filePath: string,
): Promise<void> {
  const ctx = Context.instance();

  try {
    const tempDir = filePath.substring(
      0,
      filePath.lastIndexOf(Deno.build.os === "windows" ? "\\" : "/"),
    );
    await Deno.remove(tempDir, { recursive: true });

    ctx.logger.debug(
      `Cleaned up downloaded template directory: {dir}`,
      { dir: tempDir },
    );
  } catch (error) {
    ctx.logger.debug(
      `Failed to clean up downloaded template: {message}`,
      {
        message: unknownValueToErrorMessage(error),
      },
    );
  }
}
