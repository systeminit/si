import _ from "lodash";
import logger from "../../logger.ts";
import { OnlyProperties } from "../../spec/props.ts";
import type { CfProperty, SuperSchema } from "../types.ts";
import { CfHandler, CfHandlerKind } from "../types.ts";
import {
  type AzureSchema,
  type JsonSchema,
  type AzureOperationData,
  type PropertySet,
} from "./schema.ts";

/**
 * Extracts properties from an Azure ARM request body schema
 */
export function extractPropertiesFromRequestBody(
  operation: JsonSchema | null,
): { properties: JsonSchema; required: string[] } {
  // Swagger 2.0 format: parameters with in:"body"
  const parameters = operation?.parameters as JsonSchema[] | undefined;
  const bodyParam = parameters?.find((p) => (p.in as string) === "body");
  let schema = bodyParam?.schema as JsonSchema | undefined;

  if (!schema) {
    return { properties: {}, required: [] };
  }

  // Flatten allOf composition recursively
  schema = flattenAllOf(schema);

  // Azure ARM often nests properties under a "properties" object
  let properties = (schema?.properties as JsonSchema) || {};

  // Azure ARM often has a "properties" property containing the actual resource properties
  // Flatten this structure by merging the nested properties into the top level
  if (properties.properties && typeof properties.properties === "object") {
    const nestedPropsSchema = properties.properties as JsonSchema;
    const nestedProps = nestedPropsSchema.properties as JsonSchema;
    if (nestedProps && typeof nestedProps === "object") {
      // Keep top-level properties and add nested properties
      properties = { ...properties, ...nestedProps };
    }
  }

  return {
    properties,
    required: (schema?.required as string[]) || [],
  };
}

/**
 * Recursively flattens allOf composition in a schema
 */
function flattenAllOf(schema: JsonSchema): JsonSchema {
  if (!schema.allOf) return schema;

  const merged: JsonSchema = { ...schema };
  delete merged.allOf;

  for (const part of schema.allOf as JsonSchema[]) {
    const flattened = flattenAllOf(part);
    Object.assign(merged, flattened);

    if (flattened.properties) {
      merged.properties = {
        ...merged.properties as JsonSchema,
        ...flattened.properties
      };
    }

    if (flattened.required) {
      merged.required = [
        ...(merged.required as string[] || []),
        ...(flattened.required as string[])
      ];
    }
  }

  return merged;
}

/**
 * Extracts properties from an Azure ARM response body schema
 */
export function extractPropertiesFromResponseBody(
  operation: JsonSchema | null,
): { properties: JsonSchema; required: string[] } {
  // Swagger 2.0 format: responses["200"].schema
  const response200 = (operation?.responses as any)?.["200"];
  let schema = response200?.schema as JsonSchema | undefined;

  if (!schema) {
    return { properties: {}, required: [] };
  }

  // Flatten allOf composition recursively
  schema = flattenAllOf(schema);

  let properties = (schema?.properties as JsonSchema) || {};

  // Azure ARM often has a "properties" property containing the actual resource properties
  // Flatten this structure by merging the nested properties into the top level
  if (properties.properties && typeof properties.properties === "object") {
    const nestedPropsSchema = properties.properties as JsonSchema;
    const nestedProps = nestedPropsSchema.properties as JsonSchema;
    if (nestedProps && typeof nestedProps === "object") {
      // Keep top-level properties (id, name, type, location, tags, etc.)
      // and add nested properties
      properties = { ...properties, ...nestedProps };
    }
  }

  return {
    properties,
    required: (schema?.required as string[]) || [],
  };
}

/**
 * Builds handlers from Azure ARM operations
 */
export function buildHandlersFromOperations(
  operations: AzureOperationData[],
): {
  handlers: Record<CfHandlerKind, CfHandler>;
  getOperation: JsonSchema | null;
  putOperation: JsonSchema | null;
  patchOperation: JsonSchema | null;
  deleteOperation: JsonSchema | null;
} {
  const handlers = {} as Record<CfHandlerKind, CfHandler>;
  let getOperation: JsonSchema | null = null;
  let putOperation: JsonSchema | null = null;
  let patchOperation: JsonSchema | null = null;
  let deleteOperation: JsonSchema | null = null;

  const defaultHandler = { permissions: [], timeoutInMinutes: 60 };

  operations.forEach(({ method, openApiOperation }) => {
    const op = openApiOperation;
    const methodLower = method.toLowerCase();

    switch (methodLower) {
      case "get": {
        getOperation = op;
        const opId = op.operationId as string;
        // Determine if this is a list or read operation
        handlers[opId?.includes("List") ? "list" : "read"] = defaultHandler;
        break;
      }
      case "put": {
        putOperation = op;
        // Azure ARM PUT is used for both create and update
        handlers["create"] = defaultHandler;
        handlers["update"] = defaultHandler;
        break;
      }
      case "patch": {
        patchOperation = op;
        handlers["update"] = defaultHandler;
        break;
      }
      case "delete": {
        deleteOperation = op;
        handlers["delete"] = defaultHandler;
        break;
      }
    }
  });

  return {
    handlers,
    getOperation,
    putOperation,
    patchOperation,
    deleteOperation,
  };
}

/**
 * Normalizes Azure ARM property schemas
 * Handles Azure-specific schema patterns and compositions
 */
export function normalizeAzureProperty(prop: JsonSchema): JsonSchema {
  // Handle allOf first
  if (prop.allOf) {
    prop = flattenAllOf(prop);
  }

  // If no type is specified but properties exist, it's an object
  if (!prop.type && prop.properties) {
    prop = { ...prop, type: "object" };
  }

  // If no type is specified but items exist, it's an array
  if (!prop.type && prop.items) {
    prop = { ...prop, type: "array" };
  }

  // If no type at all, default to string
  if (!prop.type && !prop.oneOf && !prop.anyOf) {
    prop = { ...prop, type: "string" };
  }

  if (prop.type) {
    // Normalize nested properties
    const normalized = { ...prop };

    // Strip Azure-specific formats that aren't supported
    if (normalized.format) {
      const format = normalized.format as string;
      // Map or remove Azure-specific formats
      if (format === "arm-id" || format === "uuid" || format === "email") {
        // These are strings
        delete normalized.format;
      } else if (format === "int32" || format === "int64") {
        // These are integers - remove format but keep type
        delete normalized.format;
      } else if (format === "decimal" || format === "double" || format === "float" || format === "unixtime") {
        // These are numbers - remove format but keep type
        delete normalized.format;
      } else if (format === "duration" || format === "date" || format === "date-time-rfc1123") {
        // Date/time formats - treat as string
        delete normalized.format;
      } else if (format === "byte" || format === "binary") {
        // Binary data - treat as string
        delete normalized.format;
      } else if (format === "password") {
        // Password field - treat as string
        delete normalized.format;
      }
    }

    // Clean up boolean values that should be removed
    const booleanKeys = ["x-nullable", "x-ms-client-flatten", "x-ms-azure-resource", "readOnly"];
    for (const key of booleanKeys) {
      if (typeof normalized[key] === "boolean" && normalized[key] !== true) {
        delete normalized[key];
      }
    }

    // Remove x-ms extensions that aren't part of standard JSON Schema
    for (const key in normalized) {
      if (key.startsWith("x-ms-") && key !== "x-ms-enum") {
        delete normalized[key];
      }
    }

    if (normalized.properties) {
      normalized.properties = Object.fromEntries(
        Object.entries(normalized.properties as Record<string, JsonSchema>).map(
          ([key, value]) => [key, normalizeAzureProperty(value)],
        ),
      );
    }

    if (
      normalized.additionalProperties &&
      typeof normalized.additionalProperties === "object"
    ) {
      normalized.additionalProperties = normalizeAzureProperty(
        normalized.additionalProperties as JsonSchema,
      );
    }

    if (normalized.items) {
      normalized.items = normalizeAzureProperty(
        normalized.items as JsonSchema,
      );
    }

    return normalized;
  }

  // Handle oneOf with primitive types
  if (prop.oneOf) {
    const allPrimitives = (prop.oneOf as JsonSchema[]).every((member) => {
      const type = member.type;
      return (
        type === "string" ||
        type === "number" ||
        type === "integer" ||
        type === "boolean"
      );
    });

    if (allPrimitives) {
      // Prefer non-string types
      const nonStringMember = (prop.oneOf as JsonSchema[]).find(
        (member) => member.type !== "string",
      );
      const smooshed = nonStringMember
        ? { ...prop, type: nonStringMember.type, oneOf: undefined }
        : { ...prop, type: "string", oneOf: undefined };

      return normalizeAzureProperty(smooshed);
    }
  }

  // Handle allOf - merge all schemas
  if (prop.allOf) {
    const merged = (prop.allOf as JsonSchema[]).reduce((acc, schema) => {
      return { ...acc, ...schema };
    }, {});
    return normalizeAzureProperty({ ...prop, ...merged, allOf: undefined });
  }

  return prop;
}

/**
 * Merges property definitions from multiple operations
 */
export function mergePropertyDefinitions(
  existing: JsonSchema | undefined,
  newProp: JsonSchema,
): JsonSchema {
  if (!existing) return newProp;

  const merged = { ...existing };

  // Merge enum values if both exist
  const existingEnum = existing.enum as unknown[] | undefined;
  const newPropEnum = newProp.enum as unknown[] | undefined;
  if (existingEnum && newPropEnum) {
    merged.enum = [...new Set([...existingEnum, ...newPropEnum])];
  } else if (newPropEnum) {
    merged.enum = newPropEnum;
  }

  // Merge descriptions
  if (newProp.description && !existing.description) {
    merged.description = newProp.description;
  }

  return merged;
}

/**
 * Merges Azure ARM resource operations into a single AzureSchema
 * Analyzes GET/PUT/PATCH/DELETE operations to infer property classifications
 */
export function mergeAzureResourceOperations(
  resourceType: string,
  operations: AzureOperationData[],
  apiVersion: string,
): { schema: AzureSchema; onlyProperties: OnlyProperties } | null {
  // Extract handlers and operations
  const {
    handlers,
    getOperation,
    putOperation,
    patchOperation,
    deleteOperation,
  } = buildHandlersFromOperations(operations);

  // Must have a GET operation to proceed
  if (!getOperation) {
    logger.debug(`No GET operation found for ${resourceType}`);
    return null;
  }

  // Get description from operation
  const description = (getOperation.description as string) ||
    (getOperation.summary as string) ||
    `Azure ${resourceType} resource`;

  // Extract properties from GET response
  const { properties: getProperties, required: getRequired } =
    extractPropertiesFromResponseBody(getOperation);

  if (!getProperties || Object.keys(getProperties).length === 0) {
    logger.debug(`No properties found in GET response for ${resourceType}`);
    return null;
  }

  const mergedProperties = { ...getProperties };
  const requiredProperties = new Set(getRequired);

  // Track which properties appear in different operations
  const getPropertySet: PropertySet = new Set(Object.keys(getProperties));
  const createUpdateProperties: PropertySet = new Set();
  const updateProperties: PropertySet = new Set();
  const deleteProperties: PropertySet = new Set();

  // Process PUT operation (create/update)
  if (putOperation) {
    const { properties: putProps, required: putRequired } =
      extractPropertiesFromRequestBody(putOperation);

    Object.keys(putProps).forEach((prop) => createUpdateProperties.add(prop));
    putRequired.forEach((prop) => requiredProperties.add(prop));

    Object.entries(putProps).forEach(([key, prop]) => {
      mergedProperties[key] = mergePropertyDefinitions(
        mergedProperties[key] as JsonSchema,
        prop as JsonSchema,
      );
    });
  }

  // Process PATCH operation (update)
  if (patchOperation) {
    const { properties: patchProps, required: patchRequired } =
      extractPropertiesFromRequestBody(patchOperation);

    Object.keys(patchProps).forEach((prop) => updateProperties.add(prop));
    patchRequired.forEach((prop) => requiredProperties.add(prop));

    Object.entries(patchProps).forEach(([key, prop]) => {
      mergedProperties[key] = mergePropertyDefinitions(
        mergedProperties[key] as JsonSchema,
        prop as JsonSchema,
      );
    });
  }

  // Build onlyProperties classification
  const onlyProperties: OnlyProperties = {
    createOnly: [],
    readOnly: [],
    writeOnly: [],
    primaryIdentifier: ["id"],
  };

  // readOnly: in GET but not in PUT/PATCH
  getPropertySet.forEach((prop) => {
    if (
      !createUpdateProperties.has(prop) &&
      !updateProperties.has(prop) &&
      !deleteProperties.has(prop)
    ) {
      onlyProperties.readOnly.push(`/${prop}`);
    }
  });

  // Normalize properties, filtering out any that are invalid (e.g., boolean primitives)
  const normalizedProperties: Record<string, CfProperty> = {};
  for (const [key, prop] of Object.entries(mergedProperties)) {
    // Skip properties that are primitives (boolean, string, etc.) rather than schema objects
    if (typeof prop !== 'object' || prop === null) {
      continue;
    }
    try {
      normalizedProperties[key] = normalizeAzureProperty(prop as JsonSchema) as CfProperty;
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      console.warn(`Failed to normalize property ${key} in ${resourceType}: ${errorMessage}`);
    }
  }

  // Extract resource path from first operation
  const resourcePath = operations[0]?.path || "";

  const schema: AzureSchema = {
    typeName: resourceType,
    description,
    properties: normalizedProperties as Record<string, CfProperty>,
    requiredProperties,
    primaryIdentifier: ["id"],
    handlers,
    apiVersion,
    resourcePath,
  };

  return { schema, onlyProperties };
}

/**
 * Converts Azure Swagger schema to SuperSchema array for spec generation
 */
export function parseAzureSchema(rawSchema: unknown): SuperSchema[] {
  const schema = rawSchema as JsonSchema;
  const schemas: SuperSchema[] = [];

  if (!schema.paths) {
    console.warn("No paths found in Azure schema");
    return [];
  }

  const resourceOperations: Record<string, AzureOperationData[]> = {};

  Object.entries(schema.paths as JsonSchema).forEach(
    ([path, pathItem]) => {
      const methods = ["get", "put", "patch", "delete", "post"];

      methods.forEach((method) => {
        if ((pathItem as JsonSchema)[method]) {
          const operation = (pathItem as JsonSchema)[method] as JsonSchema;
          const resourceType = extractResourceTypeFromPath(path);

          if (!resourceType) return;

          // Skip test/placeholder resources
          if (resourceType.includes("Reminder: Need renaming")) return;

          if (!resourceOperations[resourceType]) {
            resourceOperations[resourceType] = [];
          }

          resourceOperations[resourceType].push({
            method,
            path,
            openApiOperation: operation,
            apiVersion: extractApiVersionFromPath(path),
          });
        }
      });
    },
  );

  Object.entries(resourceOperations).forEach(([resourceType, operations]) => {
    const apiVersion = operations[0]?.apiVersion || "2023-01-01";
    const result = mergeAzureResourceOperations(
      resourceType,
      operations,
      apiVersion,
    );

    if (result) {
      (result.schema as any)._inferredOnlyProperties = result.onlyProperties;
      schemas.push(result.schema);
    }
  });

  console.log(
    `Generated ${schemas.length} schemas from ${
      Object.keys(resourceOperations).length
    } resource types`,
  );

  return schemas;
}

function extractResourceTypeFromPath(path: string): string | null {
  const match = path.match(/providers\/([^/]+)\/([^/{]+)/);
  if (match) {
    const providerNamespace = match[1];
    // Extract the service name after the last dot (e.g., Microsoft.Compute -> Compute)
    const serviceName = providerNamespace.split('.').pop() || providerNamespace;
    // Capitalize first letter of resource type
    const resourceType = match[2];
    const capitalizedResource = resourceType.charAt(0).toUpperCase() + resourceType.slice(1);
    return `Azure::${serviceName}::${capitalizedResource}`;
  }
  return null;
}

function extractApiVersionFromPath(path: string): string | undefined {
  return "2023-01-01";
}
