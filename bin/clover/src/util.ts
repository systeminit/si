import _logger from "./logger.ts";

const logger = _logger.ns("util").seal();

/**
 * Converts HTML tags in text to markdown-friendly format.
 * Primarily handles <br> tags by converting them to newlines.
 * @param text - The text that may contain HTML tags
 * @returns Text with HTML tags converted to markdown
 */
export function htmlToMarkdown(text: string | null | undefined): string | null {
  if (!text) return text ?? null;

  // Replace <br>, <br/>, <br /> with newlines
  let result = text.replace(/<br\s*\/?>/gi, "\n");

  // Replace multiple consecutive newlines (from <br><br>) with double newline
  result = result.replace(/\n\n+/g, "\n\n");

  // Trim any leading/trailing whitespace that may have been introduced
  result = result.trim();

  return result;
}

export async function emptyDirectory(dirPath: string) {
  logger.log(`Cleaning out ${dirPath}`);
  for await (const entry of Deno.readDir(dirPath)) {
    if (entry.name === ".gitignore") continue;

    const entryPath = `${dirPath}/${entry.name}`;

    if (entry.isDirectory) {
      await Deno.remove(entryPath, { recursive: true });
    } else {
      await Deno.remove(entryPath);
    }
  }
}
