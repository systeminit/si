import _ from "lodash";
import { OnlyProperties } from "../../spec/props.ts";
import type { CfProperty } from "../types.ts";
import { CfHandler, CfHandlerKind } from "../types.ts";
import {
  type DigitalOceanOpenApiOperation,
  type DigitalOceanOpenApiRequestBody,
  type DigitalOceanOpenApiResponse,
  type DigitalOceanSchema,
  type JsonSchemaObject,
  type NormalizedDigitalOceanSchema,
  type OperationData,
  type PropertySet,
} from "./schema.ts";

export function mergeResourceOperations(
  resourceName: string,
  operations: OperationData[],
  description?: string,
): {
  schema: DigitalOceanSchema;
  onlyProperties: OnlyProperties;
  domainProperties: Record<string, CfProperty>;
  resourceValueProperties: Record<string, CfProperty>;
} | null {
  const {
    handlers,
    getOperation,
    postOperation,
    putOperation,
    patchOperation,
    isGetOperationList,
  } = buildHandlersFromOperations(operations);

  // Use provided description or fall back to default
  const schemaDescription = description ??
    `DigitalOcean ${resourceName} resource`;

  const createProps = extractPropertiesFromRequestBody(
    postOperation?.requestBody,
  );

  // Start with POST properties for domain (writable properties)
  const domainProperties = { ...createProps.properties };
  const requiredProperties = new Set(createProps.required || []);

  // Properties that appear in different operations - for onlyProperties classification
  const createProperties: PropertySet = new Set(
    Object.keys(createProps.properties),
  );

  const getResponse = getOperation?.responses?.["200"];

  const getProps = extractPropertiesFromResponse(
    getResponse,
    isGetOperationList,
  );
  const getProperties: PropertySet = new Set(Object.keys(getProps.properties));
  const updateProperties: PropertySet = new Set();

  // Merge PUT/PATCH into domain (POST + PUT/PATCH = writable properties)
  const updateOperation = patchOperation || putOperation;
  if (updateOperation) {
    const updateProps = extractPropertiesFromRequestBody(
      updateOperation.requestBody,
    );
    Object.keys(updateProps.properties).forEach((prop) =>
      updateProperties.add(prop)
    );
    Object.entries(updateProps.properties).forEach(([key, prop]) => {
      domainProperties[key] = mergePropertyDefinitions(
        domainProperties[key],
        prop,
      );
    });
    // Add required properties from UPDATE operation
    updateProps.required.forEach((prop) => requiredProperties.add(prop));
  }

  const resourceValueProperties = { ...getProps.properties };

  const onlyProperties: OnlyProperties = {
    createOnly: [],
    readOnly: [],
    writeOnly: [],
    primaryIdentifier: ["id"],
  };

  // createOnly: only in POST, not in PUT/PATCH
  createProperties.forEach((prop) => {
    if (!updateProperties.has(prop)) {
      onlyProperties.createOnly.push(prop);
    }
  });

  // readOnly: in GET but not in POST or PUT/PATCH
  getProperties.forEach((prop) => {
    if (!createProperties.has(prop) && !updateProperties.has(prop)) {
      onlyProperties.readOnly.push(prop);
    }
  });

  // writeOnly: in POST/PUT/PATCH but not in GET
  const writeProps = [...createProperties, ...updateProperties];
  onlyProperties.writeOnly = [
    ...new Set(writeProps.filter((prop) => !getProperties.has(prop))),
  ];

  // Remove readOnly properties from domain
  const writableDomainProperties = Object.fromEntries(
    Object.entries(domainProperties).filter(([_, prop]) => !prop.readOnly),
  );

  // Normalize domain properties (POST + PUT/PATCH = writable)
  const normalizedDomainProperties = Object.fromEntries(
    Object.entries(writableDomainProperties).map(([key, prop]) => [
      key,
      normalizeDigitalOceanProperty(prop),
    ]),
  );

  // Normalize resource_value properties (GET response = readable)
  const normalizedResourceValueProperties = Object.fromEntries(
    Object.entries(resourceValueProperties).map(([key, prop]) => [
      key,
      normalizeDigitalOceanProperty(prop),
    ]),
  );

  // Use provider-style naming: DigitalOcean/droplets
  const schema: DigitalOceanSchema = {
    typeName: `DigitalOcean/${resourceName}`,
    description: schemaDescription,
    requiredProperties,
    handlers,
    endpoint: resourceName,
  };

  return {
    schema,
    onlyProperties,
    domainProperties: normalizedDomainProperties as Record<string, CfProperty>,
    resourceValueProperties: normalizedResourceValueProperties as Record<
      string,
      CfProperty
    >,
  };
}

export function buildHandlersFromOperations(operations: OperationData[]): {
  handlers: Record<CfHandlerKind, CfHandler>;
  getOperation: DigitalOceanOpenApiOperation | null;
  postOperation: DigitalOceanOpenApiOperation | null;
  putOperation: DigitalOceanOpenApiOperation | null;
  patchOperation: DigitalOceanOpenApiOperation | null;
  isGetOperationList: boolean;
} {
  const handlers = {} as Record<CfHandlerKind, CfHandler>;
  let getOperation: DigitalOceanOpenApiOperation | null = null;
  let postOperation: DigitalOceanOpenApiOperation | null = null;
  let putOperation: DigitalOceanOpenApiOperation | null = null;
  let patchOperation: DigitalOceanOpenApiOperation | null = null;
  let isGetOperationList = false;

  operations.forEach(({ openApiDescription }) => {
    const defaultHandler = { permissions: [], timeoutInMinutes: 60 };

    if (openApiDescription.get) {
      const op = openApiDescription.get;
      const isListOp = isListOperation(op);

      // Prefer READ over LIST for extracting resource properties
      if (!getOperation || (!isListOp && isGetOperationList)) {
        getOperation = op;
        isGetOperationList = isListOp;
      }

      handlers[isListOp ? "list" : "read"] = defaultHandler;
    }
    if (openApiDescription.patch) {
      patchOperation = openApiDescription.patch;
      handlers["update"] = defaultHandler;
    }
    if (openApiDescription.put) {
      putOperation = openApiDescription.put;
      // Only set update handler if PATCH doesn't already exist
      if (!handlers["update"]) {
        handlers["update"] = defaultHandler;
      }
    }
    if (openApiDescription.post) {
      postOperation = openApiDescription.post;
      handlers["create"] = defaultHandler;
    }
    if (openApiDescription.delete) {
      handlers["delete"] = defaultHandler;
    }
  });

  return {
    handlers,
    getOperation,
    postOperation,
    putOperation,
    patchOperation,
    isGetOperationList,
  };
}

export function extractPropertiesFromRequestBody(
  requestBody: DigitalOceanOpenApiRequestBody | undefined,
): { properties: Record<string, NormalizedDigitalOceanSchema>; required: string[] } {
  const schema = requestBody?.content?.["application/json"]?.schema;

  return flattenSchemaProperties(schema);
}

export function extractPropertiesFromResponse(
  response: DigitalOceanOpenApiResponse | undefined,
  isListOperation: boolean = false,
): { properties: Record<string, NormalizedDigitalOceanSchema>; required: string[] } {
  let schema = response?.content?.["application/json"]?.schema;

  // For LIST operations, extract from the collection items
  if (isListOperation && schema) {
    // Flatten first to resolve allOf
    const flattened = flattenSchemaProperties(schema);

    // DigitalOcean typically uses a wrapper object with the resource name as key
    // e.g., { droplets: [...] } or { volumes: [...] }
    // Find the first property that is an array
    for (const [key, propSchema] of Object.entries(flattened.properties)) {
      if (propSchema.type === "array" && propSchema.items) {
        schema = propSchema.items;
        return flattenSchemaProperties(schema);
      }
    }
  }

  return flattenSchemaProperties(schema);
}

function flattenSchemaProperties(
  schema: NormalizedDigitalOceanSchema | undefined,
): { properties: Record<string, NormalizedDigitalOceanSchema>; required: string[] } {
  if (!schema) {
    return { properties: {}, required: [] };
  }

  const properties: Record<string, NormalizedDigitalOceanSchema> = {};
  const required: string[] = [];

  // Recursively merge properties from allOf members first
  if (schema.allOf && Array.isArray(schema.allOf)) {
    for (const member of schema.allOf) {
      const flattened = flattenSchemaProperties(member);
      // Merge properties from this allOf member
      Object.assign(properties, flattened.properties);
      // Merge required arrays
      required.push(...flattened.required);
    }
  }

  // Then add properties from the main schema (these take precedence)
  if (schema.properties) {
    Object.assign(properties, schema.properties);
  }

  // Add required from main schema
  if (schema.required) {
    required.push(...schema.required);
  }

  // Remove duplicates from required array
  const uniqueRequired = [...new Set(required)];

  return { properties, required: uniqueRequired };
}

export function normalizeDigitalOceanProperty(
  prop: JsonSchemaObject,
  path = "",
  visited = new Set<JsonSchemaObject>(),
): JsonSchemaObject {
  if (visited.has(prop)) {
    throw new Error(
      `Cycle detected in schema at path: ${path || "(root)"}`,
    );
  }

  // Validate input
  if (!prop || typeof prop !== "object") {
    throw new Error(
      `Invalid schema at path ${
        path || "(root)"
      }: expected object, got ${typeof prop}`,
    );
  }

  // Track this object to detect cycles
  const newVisited = new Set(visited);
  newVisited.add(prop);

  if (prop.type) {
    // normalize nested properties
    const normalized = { ...prop };

    // Remove or normalize unsupported formats
    const format = normalized.format;
    if (format && typeof format === "string") {
      // Remove unsupported formats
      if (
        [
          "duration",
          "uuid",
          "email",
          "date",
          "date-time-rfc1123",
          "byte",
          "binary",
          "password",
          "uri",
        ].includes(format)
      ) {
        delete normalized.format;
      } // Normalize integer formats to int64
      else if (
        normalized.type === "integer" &&
        (format === "int32" || format === "int64")
      ) {
        normalized.format = "int64";
      } // Normalize number formats to double
      else if (
        normalized.type === "number" &&
        (format === "float" || format === "double" || format === "decimal" ||
          format === "int32")
      ) {
        normalized.format = "double";
      }
    }

    // Recursively normalize nested properties
    if (normalized.properties) {
      try {
        normalized.properties = Object.fromEntries(
          Object.entries(normalized.properties).map(
            ([key, value]) => {
              if (!value || typeof value !== "object") {
                throw new Error(
                  `Invalid property "${key}" at path ${path}: expected object`,
                );
              }
              return [
                key,
                normalizeDigitalOceanProperty(
                  value,
                  path ? `${path}.${key}` : key,
                  newVisited,
                ),
              ];
            },
          ),
        );
      } catch (error) {
        throw new Error(
          `Error normalizing properties at path ${path || "(root)"}: ${
            error instanceof Error ? error.message : String(error)
          }`,
        );
      }
    }

    // Recursively normalize additionalProperties
    if (
      normalized.additionalProperties &&
      typeof normalized.additionalProperties === "object"
    ) {
      try {
        normalized.additionalProperties = normalizeDigitalOceanProperty(
          normalized.additionalProperties,
          path ? `${path}[additionalProperties]` : "[additionalProperties]",
          newVisited,
        );
      } catch (error) {
        throw new Error(
          `Error normalizing additionalProperties at path ${
            path || "(root)"
          }: ${error instanceof Error ? error.message : String(error)}`,
        );
      }
    }

    // Recursively normalize array items
    if (normalized.items) {
      if (typeof normalized.items !== "object") {
        throw new Error(
          `Invalid items definition at path ${
            path || "(root)"
          }: expected object`,
        );
      }
      try {
        normalized.items = normalizeDigitalOceanProperty(
          normalized.items,
          path ? `${path}[items]` : "[items]",
          newVisited,
        );
      } catch (error) {
        throw new Error(
          `Error normalizing items at path ${path || "(root)"}: ${
            error instanceof Error ? error.message : String(error)
          }`,
        );
      }
    }

    return normalized;
  }

  // Handle oneOf with primitive types - smoosh them like cfDb does for array types
  if (prop.oneOf) {
    if (!Array.isArray(prop.oneOf)) {
      throw new Error(
        `Invalid oneOf at path ${path || "(root)"}: expected array`,
      );
    }

    const allPrimitives = prop.oneOf.every((member) => {
      if (!member || typeof member !== "object") {
        return false;
      }
      const type = member.type;
      return (
        type === "string" ||
        type === "number" ||
        type === "integer" ||
        type === "boolean"
      );
    });

    if (allPrimitives) {
      // Pick the non-string type (prefer number, integer, boolean over string)
      const nonStringMember = prop.oneOf.find(
        (member) => member.type !== "string",
      );
      const smooshed = nonStringMember
        ? { ...prop, type: nonStringMember.type, oneOf: undefined }
        : { ...prop, type: "string", oneOf: undefined };

      return normalizeDigitalOceanProperty(smooshed, path, newVisited);
    }
  }

  return prop;
}

export function mergePropertyDefinitions(
  existing: NormalizedDigitalOceanSchema | undefined,
  newProp: NormalizedDigitalOceanSchema,
): NormalizedDigitalOceanSchema {
  if (!existing) return newProp;

  if (existing.type !== newProp.type && newProp.type) {
    return { ...newProp };
  }

  const merged = { ...existing };

  // Merge enum values if both exist
  if (existing.enum && newProp.enum) {
    merged.enum = [...new Set([...existing.enum, ...newProp.enum])];
  } else if (newProp.enum) {
    merged.enum = newProp.enum;
  }

  return merged;
}

function isListOperation(operation: DigitalOceanOpenApiOperation): boolean {
  const response = operation.responses?.["200"];

  const schema = response?.content?.["application/json"]?.schema;
  if (!schema) {
    return false;
  }

  if (schema.type === "array") {
    return true;
  }

  // Check if response has properties that are arrays (DigitalOcean pattern)
  // e.g., { droplets: [...] }
  let properties = schema.properties;

  // If there's an allOf, merge properties from all schemas
  if (schema.allOf) {
    properties = {};
    for (const subSchema of schema.allOf) {
      if (subSchema.properties) {
        Object.assign(properties, subSchema.properties);
      }
    }
  }

  if (properties) {
    // Check if any property is an array
    for (const propSchema of Object.values(properties)) {
      if (
        typeof propSchema === "object" &&
        (propSchema.type === "array" || propSchema.items !== undefined)
      ) {
        return true;
      }
    }
  }

  return false;
}
