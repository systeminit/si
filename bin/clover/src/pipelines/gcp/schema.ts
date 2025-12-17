import { SuperSchema } from "../types.ts";
import { JSONSchema } from "../draft_07.ts";
import { Extend } from "../../extend.ts";

export type JsonSchema = JSONSchema;

// After dereferencing, all schemas are objects (no boolean/null)
// Make it recursive so nested properties are also JsonSchemaObject
export type JsonSchemaObject =
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
  };

export type NormalizedGcpSchema = Extend<
  JsonSchemaObject,
  {
    allOf?: readonly NormalizedGcpSchema[];
    oneOf?: readonly NormalizedGcpSchema[];
    anyOf?: readonly NormalizedGcpSchema[];
    items?: NormalizedGcpSchema;
    properties?: Record<string, NormalizedGcpSchema>;
    additionalProperties?: NormalizedGcpSchema;
  }
>;

export interface GcpDiscoveryDocument {
  kind: "discovery#restDescription";
  name: string;
  version: string;
  title: string;
  description?: string;
  documentationLink?: string;
  baseUrl: string;
  basePath: string;
  rootUrl: string;
  servicePath: string;
  parameters?: Record<string, GcpParameter>;
  auth?: GcpAuth;
  schemas: Record<string, NormalizedGcpSchema>;
  resources?: Record<string, GcpResource>;
  methods?: Record<string, GcpMethod>;
}

export interface GcpResource {
  methods?: Record<string, GcpMethod>;
  resources?: Record<string, GcpResource>;
}

export interface GcpMethod {
  id: string;
  path: string;
  httpMethod: string;
  description?: string;
  parameters?: Record<string, GcpParameter>;
  parameterOrder?: string[];
  request?: NormalizedGcpSchema;
  response?: NormalizedGcpSchema;
  scopes?: string[];
  supportsMediaDownload?: boolean;
  supportsMediaUpload?: boolean;
}

export interface GcpParameter {
  type: string;
  description?: string;
  required?: boolean;
  location: "path" | "query";
  pattern?: string;
  minimum?: string;
  maximum?: string;
  default?: string;
}

export interface GcpAuth {
  oauth2?: {
    scopes: Record<string, { description: string }>;
  };
}

export interface GcpSchema extends SuperSchema {
  requiredProperties: Set<string>;
  service: string;
  title: string;
  version: string;
  resourcePath: string[];
  baseUrl: string;
  documentationLink?: string;
  // API methods for CRUD operations
  methods: {
    get?: GcpMethod;
    insert?: GcpMethod;
    update?: GcpMethod;
    patch?: GcpMethod;
    delete?: GcpMethod;
    list?: GcpMethod;
  };
}

// Raw (un-dereferenced) types from GCP Discovery documents
// In GCP Discovery format, request/response are ALWAYS { $ref: string }, never inline
interface RawGcpMethod {
  id: string;
  path: string;
  httpMethod: string;
  description?: string;
  parameters?: Record<string, GcpParameter>;
  parameterOrder?: string[];
  request?: { $ref: string };
  response?: { $ref: string };
  scopes?: string[];
  supportsMediaDownload?: boolean;
  supportsMediaUpload?: boolean;
}

interface RawGcpResource {
  methods?: Record<string, RawGcpMethod>;
  resources?: Record<string, RawGcpResource>;
}

interface RawGcpDiscoveryDocument {
  kind: "discovery#restDescription";
  name: string;
  version: string;
  title: string;
  description?: string;
  documentationLink?: string;
  baseUrl: string;
  basePath: string;
  rootUrl: string;
  servicePath: string;
  parameters?: Record<string, GcpParameter>;
  auth?: GcpAuth;
  schemas: Record<string, NormalizedGcpSchema>;
  resources?: Record<string, RawGcpResource>;
  methods?: Record<string, RawGcpMethod>;
}

export async function readGcpDiscoveryDocument(
  filePath: string,
): Promise<GcpDiscoveryDocument> {
  const content = await Deno.readTextFile(filePath);
  const doc = JSON.parse(content) as RawGcpDiscoveryDocument;

  // Dereference $ref fields in the GCP Discovery document
  // Note that this is a different $ref than OpenApi specs, but simpler in that
  // it just references within the same file by name
  return dereferenceGcpDiscoveryDocument(doc);
}

/**
 * Dereferences $ref fields in a GCP Discovery document.
 *
 * GCP Discovery documents use a simpler $ref system than OpenAPI - they reference
 * schemas by name within the same document's schemas object. This function
 * dereferences:
 * 1. Method-level refs (method.request.$ref, method.response.$ref)
 * 2. Schema-level refs (nested within properties, items, additionalProperties)
 *
 * Example transformation:
 * Before: method.request = { $ref: "Instance" }
 * After:  method.request = { type: "object", properties: { name: {...}, ... } }
 *
 * Circular references are handled by replacing them with a placeholder object.
 *
 * @param doc - Raw GCP Discovery document with $ref references
 * @returns Fully dereferenced document with resolved schema objects
 */
function dereferenceGcpDiscoveryDocument(
  doc: RawGcpDiscoveryDocument,
): GcpDiscoveryDocument {
  const schemas = doc.schemas || {};

  const dereferencedSchemas = dereferenceSchemas(schemas);

  // Create a dereferenced copy of the document
  const dereferenced: GcpDiscoveryDocument = {
    ...doc,
    schemas: dereferencedSchemas,
    resources: doc.resources
      ? dereferenceResources(doc.resources, dereferencedSchemas)
      : undefined,
    methods: doc.methods
      ? dereferenceMethods(doc.methods, dereferencedSchemas)
      : undefined,
  };

  return dereferenced;
}

function dereferenceSchemas(
  schemas: Record<string, NormalizedGcpSchema>,
): Record<string, NormalizedGcpSchema> {
  const dereferenced: Record<string, NormalizedGcpSchema> = {};

  for (const [name, schema] of Object.entries(schemas)) {
    dereferenced[name] = dereferenceSchema(schema, schemas, new Set());
  }

  return dereferenced;
}

function dereferenceSchema(
  schema: NormalizedGcpSchema,
  allSchemas: Record<string, NormalizedGcpSchema>,
  visited: Set<string>,
): NormalizedGcpSchema {
  // Handle $ref - dereference by looking up in allSchemas
  if (hasRef(schema)) {
    if (visited.has(schema.$ref)) {
      return {
        type: "object",
        description: `Circular reference to ${schema.$ref}`,
      };
    }
    visited.add(schema.$ref);
    const referencedSchema = allSchemas[schema.$ref];
    if (referencedSchema) {
      return dereferenceSchema(referencedSchema, allSchemas, visited);
    }
  }

  const dereferenced: NormalizedGcpSchema = { ...schema };

  if (schema.properties) {
    dereferenced.properties = {};
    for (const [key, value] of Object.entries(schema.properties)) {
      dereferenced.properties[key] = dereferenceSchema(
        value,
        allSchemas,
        new Set(visited),
      );
    }
  }

  if (schema.items) {
    dereferenced.items = dereferenceSchema(
      schema.items,
      allSchemas,
      new Set(visited),
    );
  }

  if (schema.additionalProperties) {
    dereferenced.additionalProperties = dereferenceSchema(
      schema.additionalProperties,
      allSchemas,
      new Set(visited),
    );
  }

  if (schema.allOf) {
    dereferenced.allOf = schema.allOf.map((s) =>
      dereferenceSchema(s, allSchemas, new Set(visited))
    );
  }
  if (schema.oneOf) {
    dereferenced.oneOf = schema.oneOf.map((s) =>
      dereferenceSchema(s, allSchemas, new Set(visited))
    );
  }
  if (schema.anyOf) {
    dereferenced.anyOf = schema.anyOf.map((s) =>
      dereferenceSchema(s, allSchemas, new Set(visited))
    );
  }

  return dereferenced;
}

function hasRef(
  schema: NormalizedGcpSchema,
): schema is NormalizedGcpSchema & { $ref: string } {
  return typeof schema === "object" &&
    schema !== null &&
    "$ref" in schema &&
    typeof schema.$ref === "string";
}

function dereferenceMethods(
  methods: Record<string, RawGcpMethod>,
  schemas: Record<string, NormalizedGcpSchema>,
): Record<string, GcpMethod> {
  const dereferenced: Record<string, GcpMethod> = {};

  for (const [name, method] of Object.entries(methods)) {
    dereferenced[name] = {
      ...method,
      request: method.request ? schemas[method.request.$ref] : undefined,
      response: method.response ? schemas[method.response.$ref] : undefined,
    };
  }

  return dereferenced;
}

function dereferenceResources(
  resources: Record<string, RawGcpResource>,
  schemas: Record<string, NormalizedGcpSchema>,
): Record<string, GcpResource> {
  const dereferenced: Record<string, GcpResource> = {};

  for (const [name, resource] of Object.entries(resources)) {
    dereferenced[name] = {
      methods: resource.methods
        ? dereferenceMethods(resource.methods, schemas)
        : undefined,
      resources: resource.resources
        ? dereferenceResources(resource.resources, schemas)
        : undefined,
    };
  }

  return dereferenced;
}
