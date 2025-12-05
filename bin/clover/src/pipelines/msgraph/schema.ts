import { JSONSchema } from "../draft_07.ts";
import { SuperSchema } from "../types.ts";
import { OpenAPIV3_1 } from "openapi-types";
import { Extend } from "../../extend.ts";
import SwaggerParser from "@apidevtools/swagger-parser";

export type JsonSchema = JSONSchema;

// Schema object type (without boolean) - used for normalized schemas
export type JsonSchemaObjectOnly =
  & Omit<
    Exclude<JSONSchema, boolean | null>,
    | "properties"
    | "additionalProperties"
    | "items"
    | "allOf"
    | "oneOf"
    | "anyOf"
  >
  & {
    properties?: Record<string, JsonSchemaObject>;
    additionalProperties?: JsonSchemaObject;
    items?: JsonSchemaObject;
    allOf?: readonly JsonSchemaObject[];
    oneOf?: readonly JsonSchemaObject[];
    anyOf?: readonly JsonSchemaObject[];
    discriminator?: {
      propertyName: string;
      mapping?: Record<string, string>;
    };
  };

// After dereferencing, schemas can be booleans (true="any", false="never") or objects
// Make it recursive so nested properties are also JsonSchemaObject
export type JsonSchemaObject = boolean | JsonSchemaObjectOnly;

export interface EntraSchema extends SuperSchema {
  typeName: string;
  description: string;
  requiredProperties: Set<string>;
  handlers: Record<string, { permissions: string[]; timeoutInMinutes: number }>;
  endpoint: string;
  discriminators?: Record<string, Record<string, string>>;
}

export type EntraOpenApiDocument = OpenAPIV3_1.Document<
  EntraOpenApiOperationExt
>;

export type EntraOpenApiOperation = Extend<
  OpenAPIV3_1.OperationObject,
  EntraOpenApiOperationExt
>;

interface EntraOpenApiOperationExt {
  parameters?: EntraOpenApiParameter[];
  requestBody?: EntraOpenApiRequestBody;
  responses?: Record<string, EntraOpenApiResponse>;
}

export type EntraOpenApiRequestBody = Extend<
  OpenAPIV3_1.RequestBodyObject,
  {
    content?: Record<string, { schema?: NormalizedEntraSchema }>;
  }
>;

export type EntraOpenApiResponse = Extend<
  OpenAPIV3_1.ResponseObject,
  {
    content?: Record<string, { schema?: NormalizedEntraSchema }>;
  }
>;

export type EntraOpenApiParameter = Extend<
  OpenAPIV3_1.ParameterObject,
  {
    schema?: NormalizedEntraSchema;
  }
>;

export interface OperationData {
  endpoint: string;
  openApiDescription: {
    get?: EntraOpenApiOperation;
    post?: EntraOpenApiOperation;
    patch?: EntraOpenApiOperation;
    delete?: EntraOpenApiOperation;
  };
}

export type PropertySet = Set<string>;

export type NormalizedEntraSchema = Extend<
  JsonSchemaObjectOnly,
  {
    allOf?: readonly NormalizedEntraSchema[];
    oneOf?: readonly NormalizedEntraSchema[];
    anyOf?: readonly NormalizedEntraSchema[];
    items?: NormalizedEntraSchema;
    properties?: Record<string, NormalizedEntraSchema>;
    additionalProperties?: NormalizedEntraSchema;
    discriminators?: Record<string, Record<string, string>>;
  }
>;

export async function readEntraOpenApiSpec(
  schemaPath: string,
): Promise<EntraOpenApiDocument> {
  const fileUrl = new URL(`file://${schemaPath}`);

  const spec = (await SwaggerParser.dereference(
    fileUrl.href,
  )) as EntraOpenApiDocument;

  return spec;
}
