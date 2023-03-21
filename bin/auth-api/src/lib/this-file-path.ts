import { fileURLToPath } from "node:url";
import path from "node:path";

// replacement for __filename and __dirname in esm
// see https://humanwhocodes.com/snippets/2023/01/mimicking-dirname-filename-nodejs-esm/
// these must be called with `import.meta.url` passed into the arg

// ex: `const __filename = getThisFilename(import.meta.url);`

export function getThisFilename(importMetaUrl: string) {
  return fileURLToPath(importMetaUrl);
}
export function getThisDirname(importMetaUrl: string) {
  return path.dirname(getThisFilename(importMetaUrl));
}
