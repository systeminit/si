import _ from "lodash";
import { OnlyProperties } from "../../spec/props.ts";
import type { CfProperty } from "../types.ts";
import { CfHandler, CfHandlerKind } from "../types.ts";
import {
  type EntraOpenApiOperation,
  type EntraOpenApiRequestBody,
  type EntraOpenApiResponse,
  type EntraSchema,
  type JsonSchemaObject,
  type NormalizedEntraSchema,
  type OperationData,
  type PropertySet,
} from "./schema.ts";

export function mergeResourceOperations(
  resourceName: string,
  operations: OperationData[],
  description?: string,
): {
  schema: EntraSchema;
  onlyProperties: OnlyProperties;
  domainProperties: Record<string, CfProperty>;
  resourceValueProperties: Record<string, CfProperty>;
} | null {
  const {
    handlers,
    getOperation,
    postOperation,
    patchOperation,
    isGetOperationList,
  } = buildHandlersFromOperations(operations);

  // Use provided description or fall back to default
  const schemaDescription = description ??
    `Microsoft Entra ${resourceName} resource`;

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

  // Microsoft Graph uses "2XX" for success responses instead of "200"
  const getResponse = getOperation?.responses?.["200"] ||
    getOperation?.responses?.["2XX"];

  const getProps = extractPropertiesFromResponse(
    getResponse,
    isGetOperationList,
  );
  const getProperties: PropertySet = new Set(Object.keys(getProps.properties));
  const updateProperties: PropertySet = new Set();

  // Merge PATCH into domain (POST + PATCH = writable properties)
  if (patchOperation) {
    const updateProps = extractPropertiesFromRequestBody(
      patchOperation.requestBody,
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

  // createOnly: only in POST, not in PATCH
  createProperties.forEach((prop) => {
    if (!updateProperties.has(prop)) {
      onlyProperties.createOnly.push(prop);
    }
  });

  // readOnly: in GET but not in POST or PATCH
  getProperties.forEach((prop) => {
    if (!createProperties.has(prop) && !updateProperties.has(prop)) {
      onlyProperties.readOnly.push(prop);
    }
  });

  // writeOnly: in POST/PATCH but not in GET
  const writeProps = [...createProperties, ...updateProperties];
  onlyProperties.writeOnly = [
    ...new Set(writeProps.filter((prop) => !getProperties.has(prop))),
  ];

  // Remove readOnly properties from domain
  const writableDomainProperties = Object.fromEntries(
    Object.entries(domainProperties).filter(([_, prop]) => !prop.readOnly),
  );

  // Normalize domain properties (POST + PATCH = writable)
  const normalizedDomainProperties = Object.fromEntries(
    Object.entries(writableDomainProperties).map(([key, prop]) => [
      key,
      normalizeEntraProperty(prop),
    ]),
  );

  // Normalize resource_value properties (GET response = readable)
  const normalizedResourceValueProperties = Object.fromEntries(
    Object.entries(resourceValueProperties).map(([key, prop]) => [
      key,
      normalizeEntraProperty(prop),
    ]),
  );

  // Use Azure-style naming: Microsoft.Graph/users
  const schema: EntraSchema = {
    typeName: `Microsoft.Graph/${resourceName}`,
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
  getOperation: EntraOpenApiOperation | null;
  postOperation: EntraOpenApiOperation | null;
  patchOperation: EntraOpenApiOperation | null;
  isGetOperationList: boolean;
} {
  const handlers = {} as Record<CfHandlerKind, CfHandler>;
  let getOperation: EntraOpenApiOperation | null = null;
  let postOperation: EntraOpenApiOperation | null = null;
  let patchOperation: EntraOpenApiOperation | null = null;
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
    patchOperation,
    isGetOperationList,
  };
}

export function extractPropertiesFromRequestBody(
  requestBody: EntraOpenApiRequestBody | undefined,
): { properties: Record<string, NormalizedEntraSchema>; required: string[] } {
  const schema = requestBody?.content?.["application/json"]?.schema;

  return flattenSchemaProperties(schema);
}

export function extractPropertiesFromResponse(
  response: EntraOpenApiResponse | undefined,
  isListOperation: boolean = false,
): { properties: Record<string, NormalizedEntraSchema>; required: string[] } {
  let schema = response?.content?.["application/json"]?.schema;

  // For LIST operations, first flatten the schema, then extract from the collection items
  if (isListOperation && schema) {
    // Flatten first to resolve allOf
    const flattened = flattenSchemaProperties(schema);

    // Now check for the 'value' array property
    const valueSchema = flattened.properties.value;
    if (valueSchema) {
      // The value property is an array, so get the items schema
      const items = valueSchema.items;
      if (items) {
        schema = items;
        return flattenSchemaProperties(schema);
      }
    }
  }

  return flattenSchemaProperties(schema);
}

function flattenSchemaProperties(
  schema: NormalizedEntraSchema | undefined,
): { properties: Record<string, NormalizedEntraSchema>; required: string[] } {
  if (!schema) {
    return { properties: {}, required: [] };
  }

  const properties: Record<string, NormalizedEntraSchema> = {};
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

export function normalizeEntraProperty(
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
      // Note: "date-time" is kept as it's widely used in Microsoft Graph (792 occurrences)
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
                normalizeEntraProperty(
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
        normalized.additionalProperties = normalizeEntraProperty(
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
        normalized.items = normalizeEntraProperty(
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

      return normalizeEntraProperty(smooshed, path, newVisited);
    }
  }

  return prop;
}

export function mergePropertyDefinitions(
  existing: NormalizedEntraSchema | undefined,
  newProp: NormalizedEntraSchema,
): NormalizedEntraSchema {
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

function isListOperation(operation: EntraOpenApiOperation): boolean {
  const response = operation.responses?.["200"] ||
    operation.responses?.["2XX"];

  const schema = response?.content?.["application/json"]?.schema;
  if (!schema) {
    return false;
  }

  if (schema.type === "array") {
    return true;
  }

  // Check if response has a 'value' property that is an array (collection endpoint pattern)
  // Need to check both direct properties and allOf
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

  if (properties?.value) {
    const valueSchema = properties.value;
    if (
      typeof valueSchema === "object" &&
      (valueSchema.type === "array" || valueSchema.items !== undefined)
    ) {
      return true;
    }
  }

  return false;
}
