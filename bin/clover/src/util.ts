import _logger from "./logger.ts";

const logger = _logger.ns("util").seal();

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
