import type {
  CfHandler,
  CfHandlerKind,
  CfObjectProperty,
  CfProperty,
} from "../types.ts";
import { JSONSchema } from "../draft_07.ts";
import assert from "node:assert";
import $RefParser from "@apidevtools/json-schema-ref-parser";

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
