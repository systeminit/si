import { JsonSchema } from "./schema.ts";
import $RefParser from "@apidevtools/json-schema-ref-parser";
import { join } from "https://deno.land/std@0.224.0/path/mod.ts";

const REPO_URL = "https://github.com/Azure/azure-rest-api-specs.git";

export async function cloneAzureSpecs(): Promise<string> {
  const tempDir = await Deno.makeTempDir({ prefix: "azure-specs-" });
  console.log(`Cloning Azure specs to ${tempDir}...`);

  const command = new Deno.Command("git", {
    args: ["clone", "--depth", "1", REPO_URL, tempDir],
  });

  const { code, stderr } = await command.output();

  if (code !== 0) {
    const errorText = new TextDecoder().decode(stderr);
    throw new Error(`Failed to clone Azure specs: ${errorText}`);
  }

  console.log("Clone complete");
  return tempDir;
}

interface ServiceVersions {
  stable: { version: string; files: string[] }[];
  preview: { version: string; files: string[] }[];
}

export async function discoverSwaggerFiles(
  repoPath: string,
): Promise<string[]> {
  const specPath = join(repoPath, "specification");
  const serviceMap = new Map<string, ServiceVersions>();

  for await (const serviceEntry of Deno.readDir(specPath)) {
    if (!serviceEntry.isDirectory || serviceEntry.name.startsWith(".")) {
      continue;
    }

    const servicePath = join(specPath, serviceEntry.name);
    const versions: ServiceVersions = { stable: [], preview: [] };

    async function scanForVersions(dir: string, depth = 0) {
      if (depth > 10) return;

      for await (const entry of Deno.readDir(dir)) {
        const fullPath = join(dir, entry.name);

        if (entry.isDirectory) {
          if (entry.name === "stable" || entry.name === "preview") {
            const versionType = entry.name as "stable" | "preview";

            for await (const versionEntry of Deno.readDir(fullPath)) {
              if (versionEntry.isDirectory && versionEntry.name.match(/^\d{4}-\d{2}-\d{2}/)) {
                const versionPath = join(fullPath, versionEntry.name);
                const files: string[] = [];

                for await (const fileEntry of Deno.readDir(versionPath)) {
                  if (
                    fileEntry.isFile &&
                    fileEntry.name.endsWith(".json") &&
                    !fileEntry.name.includes("example")
                  ) {
                    files.push(join(versionPath, fileEntry.name));
                  }
                }

                if (files.length > 0) {
                  versions[versionType].push({
                    version: versionEntry.name,
                    files,
                  });
                }
              }
            }
          } else if (!entry.name.startsWith(".") && entry.name !== "examples") {
            await scanForVersions(fullPath, depth + 1);
          }
        }
      }
    }

    await scanForVersions(servicePath);

    if (versions.stable.length > 0 || versions.preview.length > 0) {
      serviceMap.set(serviceEntry.name, versions);
    }
  }

  const latestFiles: string[] = [];

  for (const [serviceName, versions] of serviceMap) {
    let selectedFiles: string[] = [];

    if (versions.stable.length > 0) {
      versions.stable.sort((a, b) => b.version.localeCompare(a.version));
      selectedFiles = versions.stable[0].files;
      console.log(`${serviceName}: using stable ${versions.stable[0].version}`);
    } else if (versions.preview.length > 0) {
      versions.preview.sort((a, b) => b.version.localeCompare(a.version));
      selectedFiles = versions.preview[0].files;
      console.log(`${serviceName}: using preview ${versions.preview[0].version}`);
    }

    latestFiles.push(...selectedFiles);
  }

  return latestFiles;
}

export async function processSwaggerFiles(
  swaggerFiles: string[],
): Promise<JsonSchema[]> {
  const swaggers: JsonSchema[] = [];
  let processed = 0;

  console.log(`Processing ${swaggerFiles.length} swagger files...`);

  for (const filePath of swaggerFiles) {
    try {
      const fileUrl = new URL(`file://${filePath}`);

      const dereferencedSwagger = await $RefParser.dereference(fileUrl.href, {
        dereference: {
          circular: "ignore",
        },
      }) as JsonSchema;

      swaggers.push(dereferencedSwagger);
      processed++;

      if (processed % 50 === 0) {
        console.log(`  Processed ${processed}/${swaggerFiles.length} files`);
      }
    } catch (error) {
      console.warn(`Failed to process ${filePath}: ${error}`);
    }
  }

  console.log(`Successfully processed ${processed} swagger files`);
  return swaggers;
}

function pruneOperation(operation: JsonSchema): JsonSchema {
  const pruned = { ...operation };

  // Remove examples - we don't need these
  delete pruned["x-ms-examples"];

  // Remove Azure-specific operation metadata we don't use
  delete pruned["x-ms-pageable"];
  delete pruned["x-ms-long-running-operation"];
  delete pruned["x-ms-long-running-operation-options"];

  // Keep only 200 and 201 responses (both contain schemas), prune others
  if (pruned.responses) {
    const responses = pruned.responses as JsonSchema;
    const newResponses: JsonSchema = {};
    if (responses["200"]) newResponses["200"] = responses["200"];
    if (responses["201"]) newResponses["201"] = responses["201"];
    pruned.responses = newResponses;
  }

  return pruned;
}

export function consolidateSpecsByService(swaggers: JsonSchema[]): Map<string, JsonSchema> {
  console.log(`Consolidating ${swaggers.length} swagger files by service...`);

  // Group by service (from info.title or path pattern)
  const serviceSpecs = new Map<string, JsonSchema>();

  for (const swagger of swaggers) {
    // Extract service name from swagger info or paths
    const info = swagger.info as JsonSchema | undefined;
    const infoTitle = info?.title as string | undefined;
    let serviceName = "unknown";

    if (infoTitle) {
      // Extract service name from title like "Microsoft.Compute" or "ComputeManagementClient"
      const match = infoTitle.match(/Microsoft\.([A-Za-z]+)|([A-Za-z]+)ManagementClient/);
      if (match) {
        serviceName = (match[1] || match[2]).toLowerCase();
      }
    }

    // Fallback: extract from first path
    if (serviceName === "unknown" && swagger.paths) {
      const firstPath = Object.keys(swagger.paths as JsonSchema)[0];
      if (firstPath) {
        const match = firstPath.match(/\/providers\/Microsoft\.([^/]+)/);
        if (match) {
          serviceName = match[1].toLowerCase();
        }
      }
    }

    if (!serviceSpecs.has(serviceName)) {
      serviceSpecs.set(serviceName, {
        swagger: "2.0",
        info: {
          title: `Azure ${serviceName}`,
          version: new Date().toISOString().split("T")[0],
          description: `Azure ${serviceName} service specifications`,
        },
        host: "management.azure.com",
        schemes: ["https"],
        paths: {},
        definitions: {},
        parameters: {},
      });
    }

    const serviceSpec = serviceSpecs.get(serviceName)!;

    if (swagger.paths) {
      const paths = swagger.paths as JsonSchema;
      const prunedPaths: JsonSchema = {};

      // Prune each path's operations
      for (const [path, pathItem] of Object.entries(paths)) {
        const prunedPathItem: JsonSchema = {};

        for (const [method, operation] of Object.entries(pathItem as JsonSchema)) {
          if (["get", "put", "patch", "delete", "post"].includes(method)) {
            prunedPathItem[method] = pruneOperation(operation as JsonSchema);
          } else {
            prunedPathItem[method] = operation;
          }
        }

        prunedPaths[path] = prunedPathItem;
      }

      Object.assign(serviceSpec.paths as JsonSchema, prunedPaths);
    }

    if (swagger.definitions) {
      Object.assign(serviceSpec.definitions as JsonSchema, swagger.definitions);
    }

    if (swagger.parameters) {
      Object.assign(serviceSpec.parameters as JsonSchema, swagger.parameters);
    }
  }

  let totalPaths = 0;
  for (const [service, spec] of serviceSpecs) {
    const pathCount = Object.keys(spec.paths as object).length;
    totalPaths += pathCount;
    console.log(`  ${service}: ${pathCount} paths`);
  }
  console.log(`Consolidated into ${serviceSpecs.size} services with ${totalPaths} total paths`);

  return serviceSpecs;
}

export async function cleanupRepo(repoPath: string): Promise<void> {
  console.log(`Cleaning up ${repoPath}...`);
  await Deno.remove(repoPath, { recursive: true });
  console.log("Cleanup complete");
}
