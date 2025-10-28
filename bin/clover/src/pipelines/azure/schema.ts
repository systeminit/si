import type {
  CfArrayProperty,
  CfObjectProperty,
  CfProperty,
  CommonCommandOptions,
  SuperSchema,
} from "../types.ts";
import { JSONSchema } from "../draft_07.ts";
import assert from "node:assert";
import SwaggerParser from "@apidevtools/swagger-parser";
import { OpenAPIV3_1 } from "openapi-types";
import path from "node:path";
import { Extend } from "../../extend.ts";

/// OpenAPI.Document without $ref, with some Azure-specific extensions
export type AzureOpenApiDocument =
  OpenAPIV3_1.Document<AzureOpenApiOperationExt>;
/// OpenAPI.Operation without $ref, with some Azure-specific extensions
export type AzureOpenApiOperation = Extend<
  OpenAPIV3_1.OperationObject,
  AzureOpenApiOperationExt
>;
export type AzureOpenApiResponse = Extend<
  OpenAPIV3_1.ResponseObject,
  HasAzureOpenApiSchema
>;
export type AzureOpenApiParameter = Extend<
  OpenAPIV3_1.ParameterObject,
  HasAzureOpenApiSchema
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
  }
>;

/// Adds "schema" to responses in OpenAPI.Operation
interface AzureOpenApiOperationExt {
  parameters?: AzureOpenApiParameter[];
  responses?: Record<string, AzureOpenApiResponse>;
  "x-ms-pageable"?: {
    nextLinkName: string;
  };
}

interface HasAzureOpenApiSchema {
  schema?: NormalizedAzureSchema;
}

export type PropertySet = Set<string>;

export interface AzureSchema extends SuperSchema {
  requiredProperties: Set<string>;
  apiVersion?: string;
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
  return versionType === "preview" ? `${versionDate}-preview` : versionDate;
}

const EXCLUDE_SPECS = [
  // The dereferencer has trouble with # formats like "$ref": "#/parameters/projectTask" for whatever reason
  "/azure-rest-api-specs/specification/cognitiveservices/data-plane/QnAMaker/stable/v4.0/QnAMaker.json",
  "/azure-rest-api-specs/specification/datamigration/resource-manager/Microsoft.DataMigration/DataMigration/stable/2025-06-30/datamigration.json",
  "/azure-rest-api-specs/specification/machinelearningservices/resource-manager/Microsoft.MachineLearningServices/stable/2025-09-01/machineLearningServices.json",
  "/azure-rest-api-specs/specification/managementgroups/resource-manager/Microsoft.Management/ManagementGroups/stable/2023-04-01/management.json",
  "/azure-rest-api-specs/specification/securityinsights/resource-manager/Microsoft.SecurityInsights/stable/2025-09-01/Metadata.json",
  "/azure-rest-api-specs/specification/workloads/resource-manager/Microsoft.Workloads/stable/2023-04-01/monitors.json",
  "/azure-rest-api-specs/specification/securityinsights/resource-manager/Microsoft.SecurityInsights/stable/2025-09-01/ContentTemplates.json",
  "/azure-rest-api-specs/specification/securityinsights/resource-manager/Microsoft.SecurityInsights/stable/2025-09-01/ContentPackages.json",
];

/**
 * Find all the latest Azure OpenAPI spec files
 */
export async function* findLatestAzureOpenApiSpecFiles(specsRepo: string) {
  const specsRoot = path.join(specsRepo, "specification");
  for await (const specDir of findLatestAzureOpenApiSpecDirs(specsRoot)) {
    // Read the spec
    let foundFiles = false;
    for await (const spec of Deno.readDir(specDir)) {
      if (spec.isFile && spec.name.endsWith(".json")) {
        foundFiles = true;
        const specPath = path.join(specDir, spec.name);
        if (!EXCLUDE_SPECS.some((s) => specPath.endsWith(s))) {
          yield specPath;
        }
      }
    }
    assert(foundFiles, `No spec files found in ${specDir}`);
  }
}

async function* findLatestAzureOpenApiSpecDirs(
  dir: string,
): AsyncGenerator<string> {
  // Now find the latest stable (or preview if no stable) version in each service directory
  let latest: { parent: "stable" | "preview"; version: string } | undefined;
  for await (const entry of Deno.readDir(dir)) {
    if (entry.isDirectory) {
      const entryPath = path.join(dir, entry.name);
      // If it's a "stable" or "preview" directory, look for the latest version and yield its specs
      if (entry.name === "stable" || entry.name === "preview") {
        // Pick the directory with the latest version
        for await (const version of Deno.readDir(entryPath)) {
          if (version.isDirectory) {
            if (
              !latest ||
              (latest.parent === entry.name && version.name > latest.version) ||
              (latest.parent === "preview" && entry.name === "stable")
            ) {
              latest = { parent: entry.name, version: version.name };
            }
          }
        }

        if (entry.name === "stable") {
          assert(latest, `No latest version in ${entryPath}`);
        }
      } else {
        yield* findLatestAzureOpenApiSpecDirs(entryPath);
      }
    }
  }
  if (latest) {
    yield path.join(dir, latest.parent, latest.version);
  }
}
