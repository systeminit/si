import { extname } from "@std/path";
import { stringify as stringifyYaml } from "@std/yaml";
import { GetSchemaV1Response } from "@systeminit/api-client";
import type { TemplateComponent, TemplateContext } from "./context.ts";

export interface BaselineCache {
  components: TemplateComponent[];
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
