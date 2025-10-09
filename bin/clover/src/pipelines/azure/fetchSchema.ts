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

    await scanForVersions(servicePath, versions);

    if (versions.stable.length > 0 || versions.preview.length > 0) {
      serviceMap.set(serviceEntry.name, versions);
    }
  }

  return selectLatestVersionFiles(serviceMap);
}

async function scanForVersions(
  dir: string,
  versions: ServiceVersions,
  depth = 0,
): Promise<void> {
  if (depth > 10) return;

  for await (const entry of Deno.readDir(dir)) {
    const fullPath = join(dir, entry.name);

    if (!entry.isDirectory) continue;

    if (entry.name === "stable" || entry.name === "preview") {
      await collectVersionFiles(fullPath, entry.name, versions);
    } else if (!entry.name.startsWith(".") && entry.name !== "examples") {
      await scanForVersions(fullPath, versions, depth + 1);
    }
  }
}

async function collectVersionFiles(
  versionTypePath: string,
  versionType: "stable" | "preview",
  versions: ServiceVersions,
): Promise<void> {
  for await (const versionEntry of Deno.readDir(versionTypePath)) {
    if (!versionEntry.isDirectory) continue;
    if (!versionEntry.name.match(/^\d{4}-\d{2}-\d{2}/)) continue;

    const versionPath = join(versionTypePath, versionEntry.name);
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

function selectLatestVersionFiles(
  serviceMap: Map<string, ServiceVersions>,
): string[] {
  const latestFiles: string[] = [];

  for (const [serviceName, versions] of serviceMap) {
    const selectedVersion = selectLatestVersion(versions);
    if (selectedVersion) {
      console.log(
        `${serviceName}: using ${selectedVersion.type} ${selectedVersion.version}`,
      );
      latestFiles.push(...selectedVersion.files);
    }
  }

  return latestFiles;
}

function selectLatestVersion(versions: ServiceVersions): {
  type: string;
  version: string;
  files: string[];
} | null {
  if (versions.stable.length > 0) {
    versions.stable.sort((a, b) => b.version.localeCompare(a.version));
    return {
      type: "stable",
      version: versions.stable[0].version,
      files: versions.stable[0].files,
    };
  }

  if (versions.preview.length > 0) {
    versions.preview.sort((a, b) => b.version.localeCompare(a.version));
    return {
      type: "preview",
      version: versions.preview[0].version,
      files: versions.preview[0].files,
    };
  }

  return null;
}

export async function processSwaggerFiles(
  swaggerFiles: string[],
): Promise<JsonSchema[]> {
  const swaggers: JsonSchema[] = [];
  let processed = 0;

  console.log(`Processing ${swaggerFiles.length} swagger files...`);

  for (const filePath of swaggerFiles) {
    try {
      const swagger = await dereferenceSwagger(filePath);
      swaggers.push(swagger);
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

function flattenAllOf(schema: JsonSchema): JsonSchema {
  if (!schema || typeof schema !== "object") return schema;

  if (schema.allOf && Array.isArray(schema.allOf)) {
    const merged: JsonSchema = {};
    for (const part of schema.allOf) {
      const flattened = flattenAllOf(part);
      if (flattened.properties) {
        Object.assign(merged, flattened.properties);
      }
    }
    if (schema.properties) {
      Object.assign(merged, schema.properties);
    }
    return { ...schema, properties: merged, allOf: undefined };
  }

  return schema;
}

async function dereferenceSwagger(filePath: string): Promise<JsonSchema> {
  const fileUrl = new URL(`file://${filePath}`);

  const swagger = (await $RefParser.dereference(fileUrl.href, {
    dereference: {
      circular: "ignore",
      onDereference: (_path: string, value: any) => {
        if (value && typeof value === "object") {
          const flattened = flattenAllOf(value);
          Object.assign(value, flattened);
        }
      },
    },
  })) as JsonSchema;

  const apiVersion = extractApiVersion(filePath);
  if (apiVersion) {
    if (!swagger.info) swagger.info = {};
    (swagger.info as JsonSchema).version = apiVersion;
  }

  return swagger;
}

function extractApiVersion(filePath: string): string | null {
  const versionMatch = filePath.match(/\/(stable|preview)\/([^/]+)\//);
  if (!versionMatch) return null;

  const [, versionType, versionDate] = versionMatch;
  return versionType === "preview" ? `${versionDate}-preview` : versionDate;
}

export function consolidateSpecsByService(
  swaggers: JsonSchema[],
): Map<string, JsonSchema> {
  console.log(`Consolidating ${swaggers.length} swagger files by service...`);

  const serviceSpecs = new Map<string, JsonSchema>();

  for (const swagger of swaggers) {
    const serviceName = extractServiceName(swagger);
    if (!serviceName) continue;

    const apiVersion = (swagger.info as JsonSchema)?.version as
      | string
      | undefined;

    if (!serviceSpecs.has(serviceName)) {
      serviceSpecs.set(serviceName, createServiceSpec(serviceName, apiVersion));
    }

    const serviceSpec = serviceSpecs.get(serviceName)!;
    updateServiceSpec(serviceSpec, swagger, apiVersion);
  }

  logConsolidationSummary(serviceSpecs);
  return serviceSpecs;
}

function extractServiceName(swagger: JsonSchema): string | null {
  const info = swagger.info as JsonSchema | undefined;
  const infoTitle = info?.title as string | undefined;

  if (infoTitle) {
    const match = infoTitle.match(/Microsoft\.([A-Za-z]+)/);
    if (match?.[1]) {
      return match[1].toLowerCase();
    }

    const clientMatch = infoTitle.match(/([A-Za-z]+)ManagementClient/);
    if (clientMatch?.[1]) {
      return clientMatch[1].toLowerCase();
    }
  }

  if (swagger.paths) {
    const firstPath = Object.keys(swagger.paths as JsonSchema)[0];
    if (firstPath) {
      const match = firstPath.match(/\/providers\/Microsoft\.([^/]+)/);
      if (match?.[1]) {
        return match[1].toLowerCase();
      }
    }
  }

  return null;
}

function createServiceSpec(
  serviceName: string,
  apiVersion?: string,
): JsonSchema {
  return {
    swagger: "2.0",
    info: {
      title: `Azure ${serviceName}`,
      version: apiVersion || new Date().toISOString().split("T")[0],
      description: `Azure ${serviceName} service specifications`,
    },
    host: "management.azure.com",
    schemes: ["https"],
    paths: {},
    definitions: {},
    parameters: {},
  };
}

function updateServiceSpec(
  serviceSpec: JsonSchema,
  swagger: JsonSchema,
  apiVersion?: string,
): void {
  if (apiVersion && shouldUpdateVersion(serviceSpec, apiVersion)) {
    (serviceSpec.info as JsonSchema).version = apiVersion;
  }

  if (swagger.paths) {
    const prunedPaths = pruneSwaggerPaths(swagger.paths as JsonSchema);
    Object.assign(serviceSpec.paths as JsonSchema, prunedPaths);
  }

  if (swagger.definitions) {
    Object.assign(
      serviceSpec.definitions as JsonSchema,
      swagger.definitions,
    );
  }

  if (swagger.parameters) {
    Object.assign(serviceSpec.parameters as JsonSchema, swagger.parameters);
  }
}

function shouldUpdateVersion(
  serviceSpec: JsonSchema,
  newVersion: string,
): boolean {
  const currentVersion = (serviceSpec.info as JsonSchema).version as string;

  if (!currentVersion) return true;

  const isNewStable = !newVersion.includes("preview");
  const isCurrentPreview = currentVersion.includes("preview");

  if (isNewStable && isCurrentPreview) return true;

  const sameStability = newVersion.includes("preview") === isCurrentPreview;
  return sameStability && newVersion > currentVersion;
}

function pruneSwaggerPaths(paths: JsonSchema): JsonSchema {
  const prunedPaths: JsonSchema = {};

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

  return prunedPaths;
}

function pruneOperation(operation: JsonSchema): JsonSchema {
  const pruned = { ...operation };

  delete pruned["x-ms-examples"];
  delete pruned["x-ms-pageable"];
  delete pruned["x-ms-long-running-operation"];
  delete pruned["x-ms-long-running-operation-options"];

  if (pruned.responses) {
    const responses = pruned.responses as JsonSchema;
    const newResponses: JsonSchema = {};

    const successCodes = ["200", "201", "202", "204"];
    for (const code of successCodes) {
      if (responses[code]) {
        newResponses[code] = responses[code];
      }
    }

    pruned.responses = newResponses;
  }

  return pruned;
}

function logConsolidationSummary(
  serviceSpecs: Map<string, JsonSchema>,
): void {
  let totalPaths = 0;

  for (const [service, spec] of serviceSpecs) {
    const pathCount = Object.keys(spec.paths as object).length;
    totalPaths += pathCount;
    console.log(`  ${service}: ${pathCount} paths`);
  }

  console.log(
    `Consolidated into ${serviceSpecs.size} services with ${totalPaths} total paths`,
  );
}

export async function cleanupRepo(repoPath: string): Promise<void> {
  try {
    console.log(`Cleaning up ${repoPath}...`);
    await Deno.remove(repoPath, { recursive: true });
    console.log("Cleanup complete");
  } catch (error) {
    console.warn(`Failed to cleanup ${repoPath}:`, error);
  }
}
