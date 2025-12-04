import { JSONSchema } from "../draft_07.ts";
import { SuperSchema } from "../types.ts";
import { OpenAPIV3_1 } from "openapi-types";
import { Extend } from "../../extend.ts";
import SwaggerParser from "@apidevtools/swagger-parser";

export type JsonSchema = JSONSchema;

// After dereferencing, all schemas are objects (no boolean/null)
// Make it recursive so nested properties are also JsonSchemaObject
export type JsonSchemaObject = Omit<
  Exclude<JSONSchema, boolean | null>,
  "properties" | "additionalProperties" | "items" | "allOf" | "oneOf" | "anyOf"
> & {
  properties?: Record<string, JsonSchemaObject>;
  additionalProperties?: JsonSchemaObject;
  items?: JsonSchemaObject;
  allOf?: readonly JsonSchemaObject[];
  oneOf?: readonly JsonSchemaObject[];
  anyOf?: readonly JsonSchemaObject[];
};

export interface DigitalOceanSchema extends SuperSchema {
  typeName: string;
  description: string;
  requiredProperties: Set<string>;
  handlers: Record<string, { permissions: string[]; timeoutInMinutes: number }>;
  endpoint: string;
  docTag?: string;
  identifierField?: string; // The field name to use as resource identifier (e.g., "id", "name", "username")
  updateMethod?: "PUT" | "PATCH"; // The HTTP method to use for updates (defaults to PUT)
  requiredQueryParams?: string[]; // Query parameters required for GET/PUT/PATCH/DELETE operations (e.g., ["region"] for NFS)
}

export type DigitalOceanOpenApiDocument = OpenAPIV3_1.Document<
  DigitalOceanOpenApiOperationExt
>;

export type DigitalOceanOpenApiOperation = Extend<
  OpenAPIV3_1.OperationObject,
  DigitalOceanOpenApiOperationExt
>;

interface DigitalOceanOpenApiOperationExt {
  parameters?: DigitalOceanOpenApiParameter[];
  requestBody?: DigitalOceanOpenApiRequestBody;
  responses?: Record<string, DigitalOceanOpenApiResponse>;
}

export type DigitalOceanOpenApiRequestBody = Extend<
  OpenAPIV3_1.RequestBodyObject,
  {
    content?: Record<string, { schema?: NormalizedDigitalOceanSchema }>;
  }
>;

export type DigitalOceanOpenApiResponse = Extend<
  OpenAPIV3_1.ResponseObject,
  {
    content?: Record<string, { schema?: NormalizedDigitalOceanSchema }>;
  }
>;

export type DigitalOceanOpenApiParameter = Extend<
  OpenAPIV3_1.ParameterObject,
  {
    schema?: NormalizedDigitalOceanSchema;
  }
>;

export interface OperationData {
  endpoint: string;
  endpointHasId: boolean;
  openApiDescription: {
    get?: DigitalOceanOpenApiOperation;
    post?: DigitalOceanOpenApiOperation;
    put?: DigitalOceanOpenApiOperation;
    patch?: DigitalOceanOpenApiOperation;
    delete?: DigitalOceanOpenApiOperation;
  };
}

export type PropertySet = Set<string>;

export type NormalizedDigitalOceanSchema = Extend<
  JsonSchemaObject,
  {
    allOf?: readonly NormalizedDigitalOceanSchema[];
    oneOf?: readonly NormalizedDigitalOceanSchema[];
    anyOf?: readonly NormalizedDigitalOceanSchema[];
    items?: NormalizedDigitalOceanSchema;
    properties?: Record<string, NormalizedDigitalOceanSchema>;
    patternProperties?: Record<string, NormalizedDigitalOceanSchema>;
    additionalProperties?: NormalizedDigitalOceanSchema;
  }
>;

export async function readDigitalOceanOpenApiSpec(
  schemaPath: string,
): Promise<DigitalOceanOpenApiDocument> {
  const fileUrl = new URL(`file://${schemaPath}`);

  const spec = (await SwaggerParser.dereference(
    fileUrl.href,
  )) as DigitalOceanOpenApiDocument;

  return spec;
}
