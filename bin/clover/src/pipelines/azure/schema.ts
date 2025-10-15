import type {
  CfHandler,
  CfHandlerKind,
  CfObjectProperty,
  CfProperty,
  CommonCommandOptions,
} from "../types.ts";
import { JSONSchema } from "../draft_07.ts";
import assert from "node:assert";
import $RefParser from "@apidevtools/json-schema-ref-parser";
import path from "node:path";

type JSONPointer = string;

export type PropertySet = Set<string>;

export interface AzureOperationData {
  method: string;
  path: string;
  openApiOperation: AzureOpenApiOperation;
  apiVersion?: string;
}

export interface AzureSchema {
  typeName: string;
  description: string;
  sourceUrl?: string;
  documentationUrl?: string;
  properties: Record<string, CfProperty>;
  requiredProperties: Set<string>;
  primaryIdentifier: JSONPointer[];
  handlers?: { [key in CfHandlerKind]?: CfHandler };
  apiVersion?: string;
}

export interface AzureOpenApiSpec {
  info: {
    title?: string;
    version: string;
    description?: string;
  };
  host: string;
  schemes: string[];
  paths: Record<string, Record<AzureHttpMethod, AzureOpenApiOperation>>;
}

export const AZURE_HTTP_METHODS = [
  "get",
  "put",
  "patch",
  "delete",
  "post",
  "head",
] as const;
export type AzureHttpMethod = (typeof AZURE_HTTP_METHODS)[number];

export interface AzureOpenApiOperation {
  operationId: string;
  description?: string;
  summary?: string;
  parameters?: AzureOpenApiParameter[];
  responses: Record<string, AzureOpenApiResponse>;
}

type AzureOpenApiParameter = {
  name: string;
  description?: string;
} & (
  | {
      in: "query" | "header" | "path" | "cookie";
    }
  | {
      name: string;
      in: "body";
      schema: JSONSchema.Object;
    }
);

export interface AzureOpenApiResponse {
  description?: string;
  schema?: JSONSchema.Object;
}

export function assertAzureOpenApiSpec(
  o: unknown,
): asserts o is AzureOpenApiSpec {
  assert(
    typeof o === "object" && o !== null,
    `Expected spec to be object: ${JSON.stringify(o, null, 2)}`,
  );
  assert(
    "info" in o && typeof o.info === "object" && o.info !== null,
    `Expected info in spec: ${JSON.stringify(o, null, 2)}`,
  );
  assert(
    "paths" in o && typeof o.paths === "object" && o.paths !== null,
    `Expected paths in spec: ${JSON.stringify(o, null, 2)}`,
  );
  for (const methods of Object.values(o.paths)) {
    assert(typeof methods === "object" && methods !== null);
    for (const [method, operation] of Object.entries(methods)) {
      // TODO DataLakeStorage does this and it's unclear if it's an error
      if (method === "parameters") continue;
      assert(
        AZURE_HTTP_METHODS.includes(method as AzureHttpMethod),
        `Unexpected method ${method} in spec: ${JSON.stringify(o, null, 2)}`,
      );
      assertAzureOpenApiOperation(operation);
    }
  }
}

export function assertAzureOpenApiOperation(
  o: unknown,
): asserts o is AzureOpenApiOperation {
  assert(
    typeof o === "object" && o !== null,
    `Expected operation to be object: ${JSON.stringify(o, null, 2)}`,
  );
  assert(
    "operationId" in o,
    `Expected operationId in operation: ${JSON.stringify(o, null, 2)}`,
  );
  if ("parameters" in o) {
    assert(
      Array.isArray(o.parameters),
      `Expected parameters to be array: ${JSON.stringify(o, null, 2)}`,
    );
    for (const parameter of o.parameters) {
      assertAzureOpenApiParameter(parameter);
    }
  }
  assert(
    "responses" in o && typeof o.responses === "object" && o.responses !== null,
    `Expected responses in operation: ${JSON.stringify(o, null, 2)}`,
  );
  for (const response of Object.values(o.responses)) {
    assertAzureOpenApiResponse(response);
  }
}

export function assertAzureOpenApiParameter(
  o: unknown,
): asserts o is AzureOpenApiResponse {
  assert(
    typeof o === "object" && o !== null,
    `Expected parameter to be object: ${JSON.stringify(o, null, 2)}`,
  );
  assert(
    "name" in o && typeof o.name === "string",
    `Expected name in parameter: ${JSON.stringify(o, null, 2)}`,
  );
  assert(
    "in" in o && typeof o.in === "string",
    `Expected 'in' in parameter: ${JSON.stringify(o, null, 2)}`,
  );
  if (o.in === "body") {
    assert(
      "schema" in o,
      `Expected schema in body parameter: ${JSON.stringify(o, null, 2)}`,
    );
  }
}

export function assertAzureOpenApiResponse(
  o: unknown,
): asserts o is AzureOpenApiResponse {
  assert(
    typeof o === "object" && o !== null,
    `Expected response to be object: ${JSON.stringify(o, null, 2)}`,
  );
}

export type AzureProperty = CfProperty;
export type AzureObjectProperty = CfObjectProperty;

export function isAzureObjectProperty(o: unknown): o is AzureObjectProperty {
  if (!(typeof o === "object" && o !== null)) return false;
  return !("type" in o && o.type !== "object");
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

  const swagger = await $RefParser.dereference(fileUrl.href, {
    dereference: {
      circular: "ignore",
      onDereference: (_path: string, value: JSONSchema) => {
        if (!(typeof value === "boolean")) {
          Object.assign(value, flattenAllOfProperties(value));
        }
      },
    },
  });

  assertAzureOpenApiSpec(swagger);

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

function flattenAllOfProperties({
  allOf,
  ...schema
}: Exclude<JSONSchema, boolean>): Exclude<JSONSchema, boolean> {
  if (Array.isArray(allOf)) {
    const merged: Exclude<JSONSchema, boolean>["properties"] = {};
    for (const part of allOf) {
      if (typeof part !== "object" || part === null)
        throw new Error("Schema object does not contain child object");
      const flattened = flattenAllOfProperties(part);
      if (flattened.properties) {
        Object.assign(merged, flattened.properties);
      }
    }
    if (schema.properties) {
      Object.assign(merged, schema.properties);
    }
    return { ...schema, properties: merged };
  }

  return schema;
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
