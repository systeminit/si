import { existsSync } from "node:fs";
import _logger from "./logger.ts";
import _ from "lodash";

const logger = _logger.ns("packageGen").seal();
export const EXISTING_PACKAGES = "existing-packages/spec.json";

async function getModuleMap(baseUrl: string): Promise<Record<string, string>> {
  try {
    const url = new URL("builtins", baseUrl).toString();
    logger.debug(`Fetching from URL: ${url}`);
    const response = await fetch(url);

    if (!response.ok) {
      logger.warn(
        `Issue getting modules from module index: ${response.status} ${response.statusText}`,
      );
      return {};
    }

    const data = await response.json();
    if (!data?.modules) {
      logger.warn("No modules in response");
      return {};
    }

    const moduleMap: Record<string, string> = {};
    for (const module of data.modules) {
      const schemas = module.metadata.schemas;
      if (!schemas || schemas.length !== 1) {
        continue;
      }
      const name = schemas[0];
      if (
        name && module?.schemaId
      ) {
        moduleMap[name] = module.schemaId;
      }
    }

    return moduleMap;
  } catch (error) {
    logger.error(
      `Error fetching modules: ${
        error instanceof Error ? error.message : String(error)
      }`,
    );
    return {};
  }
}

export async function getExistingSpecs(
  options: {
    moduleIndexUrl: string;
    forceUpdateExistingPackages?: boolean;
  },
): Promise<Record<string, string>> {
  logger.info("Getting existing specs...");

  if (!existsSync(EXISTING_PACKAGES) || options.forceUpdateExistingPackages) {
    logger.info(`Fetching builtin modules from ${options.moduleIndexUrl}`);
    const moduleMap = await getModuleMap(options.moduleIndexUrl);

    // Ensure parent directory exists
    const dir = EXISTING_PACKAGES.split("/").slice(0, -1).join("/");
    await Deno.mkdir(dir, { recursive: true });

    await Deno.writeTextFile(
      EXISTING_PACKAGES,
      JSON.stringify(moduleMap, null, 2),
    );
  }

  const fullPath = await Deno.realPath(EXISTING_PACKAGES);
  return (await import(fullPath, {
    with: { type: "json" },
  })).default;
}
