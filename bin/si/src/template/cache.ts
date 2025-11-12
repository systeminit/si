import { extname } from "@std/path";
import { stringify as stringifyYaml } from "@std/yaml";
import type { GetSchemaV1Response } from "@systeminit/api-client";
import type { TemplateComponent, TemplateContext } from "./context.ts";

/**
 * Container for cached baseline data and schema metadata.
 *
 * Used to persist baseline components and their schemas to disk, enabling:
 * - Faster subsequent template runs (skip API queries for baseline)
 * - Offline development and testing with cached data
 * - Debugging and inspection of baseline state
 *
 * The cache can be written in JSON or YAML format based on file extension.
 *
 * @example
 * ```ts
 * // Cache structure when written to file
 * {
 *   "components": [
 *     {
 *       "id": "01HQXYZ...",
 *       "schemaId": "01HQABC...",
 *       "name": "my-component",
 *       "resourceId": "res-123",
 *       "attributes": { "/si/name": "my-component", ... }
 *     }
 *   ],
 *   "schemas": {
 *     "01HQABC...": {
 *       "name": "AWS EC2 Instance",
 *       ...
 *     }
 *   }
 * }
 * ```
 */
export interface BaselineCache {
  /** Array of template components that make up the baseline */
  components: TemplateComponent[];
  /** Map of schema ID to schema metadata for all schemas used by baseline components */
  schemas: Record<string, GetSchemaV1Response>;
}

/**
 * Cache the baseline and schema cache to a file.
 * Format (JSON/YAML) is determined by file extension.
 *
 * @param ctx - The template context
 * @param filePath - Path where cache file should be written
 */
export async function cacheBaseline(
  ctx: TemplateContext,
  filePath: string,
): Promise<void> {
  const baseline = ctx.baseline();

  if (!baseline) {
    ctx.logger.warn("Cannot cache baseline: baseline is not set");
    return;
  }

  ctx.logger.info(`Caching baseline to {filePath}`, { filePath });

  // Convert schema cache Map to plain object
  const schemas = Object.fromEntries(ctx.schemaCache());

  const cacheData: BaselineCache = {
    components: baseline,
    schemas: schemas,
  };

  // Determine format from extension
  const ext = extname(filePath).toLowerCase();
  let content: string;

  if (ext === ".json") {
    content = JSON.stringify(cacheData, null, 2);
  } else if (ext === ".yaml" || ext === ".yml") {
    content = stringifyYaml(cacheData);
  } else {
    throw new Error(
      `Unsupported cache file format: ${ext}. Use .json, .yaml, or .yml`,
    );
  }

  await Deno.writeTextFile(filePath, content);

  ctx.logger.info(
    `Cached {componentCount} components and {schemaCount} schemas to {filePath}`,
    {
      componentCount: baseline.length,
      schemaCount: Object.keys(schemas).length,
      filePath,
    },
  );
}
