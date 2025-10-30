import type {
  CfArrayProperty,
  CfObjectProperty,
  CfProperty,
  CommonCommandOptions,
  SuperSchema,
} from "../types.ts";
import { JSONSchema } from "../draft_07.ts";
import SwaggerParser from "@apidevtools/swagger-parser";
import { OpenAPIV3_1 } from "openapi-types";
import path from "node:path";
import { Extend } from "../../extend.ts";
import { parseEndpointPath } from "./spec.ts";

/// Azure schema property
export type AzureSchemaProperty = JSONSchema.Interface & AzureSchemaExtensions;

/// Azure-specific schema extensions
export type AzureSchemaExtensions = {
  "x-ms-discriminator-value"?: string;
  discriminator?: string;
  properties?: Record<string, AzureSchemaProperty>;
};

/// Azure schema definition
export type AzureSchemaDefinition = JSONSchema & AzureSchemaExtensions;

/// Azure definitions object
export type AzureDefinitions = Record<string, AzureSchemaDefinition>;

/// OpenAPI.Document without $ref, with some Azure-specific extensions
export type AzureOpenApiDocument =
  & OpenAPIV3_1.Document<
    AzureOpenApiOperationExt
  >
  & {
    definitions?: AzureDefinitions;
  };
/// OpenAPI.Operation without $ref, with some Azure-specific extensions
export type AzureOpenApiOperation = Extend<
  OpenAPIV3_1.OperationObject,
  AzureOpenApiOperationExt
>;
interface AzureOpenApiOperationExt {
  parameters?: AzureOpenApiParameter[];
  responses?: Record<string, AzureOpenApiResponse>;
  "x-ms-pageable"?: {
    nextLinkName: string;
  }
}
export type AzureOpenApiResponse = Extend<
  OpenAPIV3_1.ResponseObject,
  {
    schema?: NormalizedAzureSchema
  }
>;
export type AzureOpenApiParameter = Extend<
  OpenAPIV3_1.ParameterObject,
  {
    type: string;
    schema?: NormalizedAzureSchema;
  }
>;

export type NormalizedAzureSchema = Extend<
  JSONSchema.Interface,
  {
    allOf?: readonly NormalizedAzureSchema[];
    oneOf?: readonly NormalizedAzureSchema[];
    anyOf?: readonly NormalizedAzureSchema[];
    items?: NormalizedAzureSchema;
    properties?: Record<string, NormalizedAzureSchema>;
    patternProperties?: Record<string, NormalizedAzureSchema>;
    additionalProperties?: NormalizedAzureSchema;
    discriminators?: Record<string, Record<string, string>>;
  }
>;

export interface AzureSchema extends SuperSchema {
  requiredProperties: Set<string>;
  apiVersion: string;
  discriminators?: Record<string, Record<string, string>>;
  resourceId: string,
}

export type AzureProperty = CfProperty & AzurePropExtensions;
export type AzureObjectProperty = CfObjectProperty & AzurePropExtensions;
export type AzureArrayProperty = CfArrayProperty & AzurePropExtensions;
interface AzurePropExtensions {
  readOnly?: boolean;
  items?: AzureProperty;
  properties?: Record<string, AzureProperty>;
}

export function isAzureObjectProperty(o: unknown): o is AzureObjectProperty {
  if (!(typeof o === "object" && o !== null)) return false;
  if ("type" in o && o.type === "object") return true;
  if ("properties" in o) return true;
  return false;
}

export function isAzureArrayProperty(o: unknown): o is AzureArrayProperty {
  if (!(typeof o === "object" && o !== null)) return false;
  if ("type" in o && o.type === "array") return true;
  if ("items" in o) return true;
  return false;
}

export async function initAzureRestApiSpecsRepo(options: CommonCommandOptions) {
  // Update the bin/clover/src/provider-schemas/azure-rest-api-specs submodule
  const command = new Deno.Command("git", {
    args: ["submodule", "update", "--init"],
  });
  const { code, stderr } = await command.output();
  if (code !== 0) {
    const errorText = new TextDecoder().decode(stderr);
    throw new Error(`Failed to update Azure specs: ${errorText}`);
  }

  return path.join(options.providerSchemasPath, "azure-rest-api-specs");
}

export async function readAzureSwaggerSpec(filePath: string) {
  const fileUrl = new URL(`file://${filePath}`);

  const swagger = (await SwaggerParser.dereference(
    fileUrl.href,
  )) as AzureOpenApiDocument;

  const apiVersion = extractApiVersion(filePath);
  if (apiVersion) {
    swagger.info.version = apiVersion;
  }

  return swagger;
}

function extractApiVersion(filePath: string): string | null {
  const versionMatch = filePath.match(/\/(stable|preview)\/([^/]+)\//);
  if (!versionMatch) return null;

  const [, versionType, versionDate] = versionMatch;
  // If it's a preview version and doesn't already end with -preview, append it
  if (versionType === "preview" && !versionDate.endsWith("-preview")) {
    return `${versionDate}-preview`;
  }
  return versionDate;
}

const EXCLUDE_SPECS = [
  // The dereferencer has trouble with # formats like "$ref": "#/parameters/projectTask" for whatever reason
  "/azure-rest-api-specs/specification/cognitiveservices/data-plane/QnAMaker/stable/v4.0/QnAMaker.json",
  "/azure-rest-api-specs/specification/datamigration/resource-manager/Microsoft.DataMigration/DataMigration/stable/2025-06-30/datamigration.json",
  "/azure-rest-api-specs/specification/machinelearningservices/resource-manager/Microsoft.MachineLearningServices/stable/2025-09-01/machineLearningServices.json",
  "/azure-rest-api-specs/specification/managementgroups/resource-manager/Microsoft.Management/ManagementGroups/stable/2023-04-01/management.json",
  "/azure-rest-api-specs/specification/securityinsights/resource-manager/Microsoft.SecurityInsights/stable/2025-09-01/Metadata.json",
  "/azure-rest-api-specs/specification/securityinsights/resource-manager/Microsoft.SecurityInsights/stable/2025-09-01/ContentTemplates.json",
  "/azure-rest-api-specs/specification/securityinsights/resource-manager/Microsoft.SecurityInsights/stable/2025-09-01/ContentPackages.json",
  // Missing example file reference
  "/azure-rest-api-specs/specification/eventhub/resource-manager/Microsoft.EventHub/preview/2018-01-01-preview/operations-preview.json",
  // Unsupported int32 number format
  "/azure-rest-api-specs/specification/azurearcdata/resource-manager/Microsoft.AzureArcData/preview/2025-06-01-preview/azurearcdata.json",
  // Unsupported duration-constant format (all servicefabricmanagedclusters versions)
  "/azure-rest-api-specs/specification/servicefabricmanagedclusters",
  // Unsupported time format
  "/azure-rest-api-specs/specification/computeschedule/resource-manager/Microsoft.ComputeSchedule/preview/2025-04-15-preview/computeschedule.json",
  // "/azure-rest-api-specs/specification/workloads/resource-manager/Microsoft.Workloads/stable/2023-04-01/monitors.json",
];

interface SpecMetadata {
  provider: string;
  version: string;
  isStable: boolean;
}

/**
 * Find all the latest Azure OpenAPI spec files with the resource types to extract from each
 * Parses each spec to extract resource types, then deduplicates by keeping the latest version per resource type
 */
export async function* findLatestAzureOpenApiSpecFiles(specsRepo: string) {
  const specsRoot = path.join(specsRepo, "specification");

  // keep the latest version for each resource type
  const resourceTypeMap: Record<string, {
    path: string;
    metadata: SpecMetadata;
  }> = {};

  for await (const specDir of findAllAzureOpenApiSpecDirs(specsRoot)) {
    for await (const spec of Deno.readDir(specDir)) {
      if (spec.isFile && spec.name.endsWith(".json")) {
        const specPath = path.join(specDir, spec.name);

        if (
          EXCLUDE_SPECS.some((s) =>
            specPath.endsWith(s) || specPath.includes(s)
          )
        ) continue;

        const metadata = parseSpecPath(specPath);
        if (!metadata) continue;

        // extract resource types
        let resourceTypes: string[];
        try {
          resourceTypes = await extractResourceTypesFromSpec(specPath);
        } catch (e) {
          console.warn(`Failed to parse ${specPath}: ${e}`);
          continue;
        }

        // track the latest version
        for (const resourceType of resourceTypes) {
          const existing = resourceTypeMap[resourceType];

          if (!existing || shouldReplace(existing.metadata, metadata)) {
            resourceTypeMap[resourceType] = {
              path: specPath,
              metadata,
            };
          }
        }
      }
    }
  }

  // Group resource types by spec path
  const specPathToResourceTypes: Record<string, Set<string>> = {};
  for (
    const [resourceType, { path: specPath }] of Object.entries(resourceTypeMap)
  ) {
    if (!specPathToResourceTypes[specPath]) {
      specPathToResourceTypes[specPath] = new Set();
    }
    specPathToResourceTypes[specPath].add(resourceType);
  }

  // Yield each spec path with the resource types that should be extracted from it
  for (
    const [specPath, resourceTypes] of Object.entries(specPathToResourceTypes)
  ) {
    yield { specPath, resourceTypes };
  }
}

/**
 * Extract resource types from an OpenAPI spec by parsing its paths
 * Returns resource types in the format "Provider/resourceType" (e.g., "Microsoft.Insights/autoscalesettings")
 */
async function extractResourceTypesFromSpec(
  specPath: string,
): Promise<string[]> {
  const fileUrl = new URL(`file://${specPath}`);

  // Parse the spec (but don't dereference, just get the paths)
  const spec = await SwaggerParser.parse(fileUrl.href) as AzureOpenApiDocument;

  if (!spec.paths) return [];

  const resourceTypes = new Set<string>();

  for (const path of Object.keys(spec.paths)) {
    const pathInfo = parseEndpointPath(path);
    if (!pathInfo) continue;

    // only include Microsoft providers
    // TODO should not need to do this, we check it later
    if (pathInfo.resourceType.toLowerCase().startsWith("microsoft.")) {
      resourceTypes.add(pathInfo.resourceType);
    }
  }

  return Array.from(resourceTypes);
}

/**
 * Find all Azure OpenAPI spec version directories (both stable and preview)
 */
async function* findAllAzureOpenApiSpecDirs(
  dir: string,
): AsyncGenerator<string> {
  for await (const entry of Deno.readDir(dir)) {
    if (entry.isDirectory) {
      const entryPath = path.join(dir, entry.name);

      // If it's a version directory (stable or preview), yield all version subdirectories
      if (entry.name === "stable" || entry.name === "preview") {
        for await (const version of Deno.readDir(entryPath)) {
          if (version.isDirectory) {
            yield path.join(entryPath, version.name);
          }
        }
      } else {
        // Recursively search subdirectories
        yield* findAllAzureOpenApiSpecDirs(entryPath);
      }
    }
  }
}

/**
 * Parse the spec file path to extract provider, version, and stability
 * Expected patterns:
 *   - .../Microsoft.{Provider}/{stable|preview}/{version}/{filename}.json
 *   - .../Microsoft.{Provider}/{SubCategory}/{stable|preview}/{version}/{filename}.json
 */
function parseSpecPath(specPath: string): SpecMetadata | null {
  // Match both patterns: with and without subcategory
  const match = specPath.match(
    /\/(Microsoft\.[^/]+)\/(?:[^/]+\/)?(stable|preview)\/([^/]+)\//,
  );
  if (!match) return null;

  return {
    provider: match[1],
    version: match[3],
    isStable: match[2] === "stable",
  };
}

/**
 * Determine if candidate spec should replace the existing one
 * ALWAYS prefers stable over preview, regardless of version
 */
function shouldReplace(
  existing: SpecMetadata,
  candidate: SpecMetadata,
): boolean {
  // If existing is stable and candidate is preview, never replace
  if (existing.isStable && !candidate.isStable) {
    return false;
  }

  // If existing is preview and candidate is stable, always replace
  if (!existing.isStable && candidate.isStable) {
    return true;
  }

  // Both have same stability, compare versions
  return candidate.version > existing.version;
}
