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
import logger from "../../logger.ts";
import { JSONSchema } from "../draft_07.ts";
import TypeValue = JSONSchema.TypeValue;

export function mergeResourceOperations(
  resourceName: string,
  endpoint: string,
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

  logger.debug(`Will parse ${resourceName}`);

  const createProps = extractPropertiesFromRequestBody(
    postOperation?.requestBody,
  );

  // Start with POST properties for domain (writable properties)
  const domainProperties = { ...createProps.properties };
  const requiredProperties = new Set(createProps.required || []);

  // Properties that appear in different operations - for onlyProperties classification
  const createProperties: PropertySet = new Set(
    Object.keys(createProps.properties ?? {}),
  );

  const getResponse = getOperation?.responses?.["200"];

  const getProps = extractPropertiesFromResponse(
    getResponse,
    isGetOperationList,
  );
  const getProperties: PropertySet = new Set(Object.keys(getProps.properties ?? {}));
  const updateProperties: PropertySet = new Set();

  // Merge PUT/PATCH into domain (POST + PUT/PATCH = writable properties)
  const updateOperation = patchOperation || putOperation;
  if (updateOperation) {
    const updateProps = extractPropertiesFromRequestBody(
      updateOperation.requestBody,
    );
    Object.keys(updateProps.properties ?? {}).forEach((prop) =>
      updateProperties.add(prop)
    );
    Object.entries(updateProps.properties ?? {}).forEach(([key, prop]) => {
      domainProperties[key] = mergePropertyDefinitions(
        domainProperties[key],
        prop,
      );
    });
    // Add required properties from UPDATE operation
    if(updateProps.required) {
      updateProps.required.forEach((prop) => requiredProperties.add(prop));
    }
  }

  const resourceValueProperties = { ...getProps.properties };

  const onlyProperties: OnlyProperties = {
    createOnly: [],
    readOnly: [],
    writeOnly: [],
    primaryIdentifier: ["id"], // TODO This is false for some DO assets
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

  // Extract the doc tag from the first operation
  const firstOp = getOperation || postOperation || putOperation || patchOperation;
  const docTag = firstOp?.tags?.[0];

  // Use provider-style naming: DigitalOcean Droplets (capitalized, preserving plurality)
  const schema: DigitalOceanSchema = {
    typeName: `DigitalOcean ${resourceName}`,
    description: schemaDescription,
    requiredProperties,
    handlers,
    endpoint,
    docTag,
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
): NormalizedDigitalOceanSchema {
  const schema = requestBody?.content?.["application/json"]?.schema;

  if (!schema) {
    return { properties: {}, required: [] };
  }

  // TODO deal with "oneOf" props
  // if (schema.oneOf && Array.isArray(schema.oneOf)) {
  //   // Try to find the "single" variant - typically has "name" not "names"
  //   let selectedSchema = schema.oneOf[0];
  //
  //   for (const alt of schema.oneOf) {
  //     const flattened = flattenSchemaProperties(alt);
  //     // Prefer schema with "name" property over "names" (single vs multi)
  //     if (flattened.properties.name && !flattened.properties.names) {
  //       selectedSchema = alt;
  //       break;
  //     }
  //   }
  //   return flattenSchemaProperties(selectedSchema);
  // }

  return flattenSchemaProperties(schema);
}

export function extractPropertiesFromResponse(
  response: DigitalOceanOpenApiResponse | undefined,
  isListOperation: boolean = false,
): NormalizedDigitalOceanSchema {
  let schema = response?.content?.["application/json"]?.schema;

  if (!schema) {
    return { properties: {}, required: [] };
  }

  // For LIST operations, extract from the collection items
  if (isListOperation) {
    // Flatten first to resolve allOf
    const flattened = flattenSchemaProperties(schema);

    // DigitalOcean typically uses a wrapper object with the resource name as key
    // e.g., { droplets: [...] } or { volumes: [...] }
    // Find the first property that is an array
    for (const [key, propSchema] of Object.entries(flattened.properties ?? {})) {
      if (propSchema.type === "array" && propSchema.items) {
        schema = propSchema.items;
        return flattenSchemaProperties(schema);
      }
    }
  } else {
    // For single resource GET operations, DigitalOcean wraps the response
    // e.g., { droplet: {...} } or { volume: {...} }
    // Unwrap by taking the first property value
    const flattened = flattenSchemaProperties(schema);
    if (flattened.properties && Object.keys(flattened.properties).length > 0) {
      const firstPropSchema = Object.values(flattened.properties)[0];
      if (firstPropSchema && firstPropSchema.properties) {
        return flattenSchemaProperties(firstPropSchema);
      }
    }
  }

  return flattenSchemaProperties(schema);
}

function flattenSchemaProperties(
  schema: NormalizedDigitalOceanSchema | undefined,
): NormalizedDigitalOceanSchema {
  if (!schema) {
    return { properties: {}, required: [] };
  }

  const clone = _.cloneDeep(schema);

  const rootProp = {
    allOf: clone.allOf,
    anyOf: clone.anyOf,
    oneOf: clone.oneOf,
    properties: clone.properties || {},
    required: clone.required || [],
    type: clone.type,
  } as NormalizedDigitalOceanSchema;

  const queue = [{ prop: rootProp, level: 0, path: "" }];
  while (queue.length > 0) {
    const entry = queue.shift();
    if (!entry) continue;
    const { prop, level, path } = entry;
    logger.verbose(`Parsing ${path} (lvl ${level})`)
    const {
      properties,
      items,
      required,
      type,
    } = flattenOfStatements(prop);

    prop.properties = properties;
    prop.required = required;
    prop.items = items;
    prop.type = type;

    prop.allOf = undefined;
    prop.anyOf = undefined;
    prop.oneOf = undefined;

    const subProps = Object.entries(prop.properties ?? {});
    if (prop.items) {
      subProps.push(["$items", prop.items]);
    }
    for (const [name, childProp] of subProps) {
      queue.push({ prop: childProp, level: level + 1, path: `${path}/${name}` });
    }
  }

  return { properties: rootProp.properties, required: rootProp.required };
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
          "ipv4", // TODO Joi can validate ips we should use this
          "ipv6", // TODO Joi can validate ips we should use this
          "cidr",
        ].includes(format)
      ) {
        delete normalized.format;
      } // Normalize integer formats to int64
      else if (
        normalized.type === "integer" &&
        ["int32", "int64", "uint32"].includes(format)
      ) {
        normalized.format = "int64";
      } // Normalize number formats to double
      else if (
        normalized.type === "number" &&
        ["float", "double", "decimal"].includes(format)
      ) {
        normalized.format = "double";
      } else if (
        normalized.type === "string" &&
        ["hostname", "url"].includes(format)
      ) {
        normalized.format = "uri";
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

  // Handle anyOf with primitive types - smoosh them like oneOf
  if (prop.anyOf) {
    if (!Array.isArray(prop.anyOf)) {
      throw new Error(
        `Invalid anyOf at path ${path || "(root)"}: expected array`,
      );
    }

    const allPrimitives = prop.anyOf.every((member) => {
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
      const nonStringMember = prop.anyOf.find(
        (member) => member.type !== "string",
      );
      const smooshed = nonStringMember
        ? { ...prop, type: nonStringMember.type, anyOf: undefined }
        : { ...prop, type: "string", anyOf: undefined };

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

function flattenOfStatements(schema: NormalizedDigitalOceanSchema): NormalizedDigitalOceanSchema {
  let properties: Record<string, NormalizedDigitalOceanSchema> | undefined = {};
  const requiredDuplicated: string[] | undefined = [];
  let items: NormalizedDigitalOceanSchema | undefined = undefined;

  let type = schema.type;

  if (schema.allOf) {
    for (const subSchema of schema.allOf) {
      if (!type) {
        type = subSchema.type;
      } else if (subSchema.type && subSchema.type !== type) {
        type = decideTypePrecedence(subSchema.type, type);
      }

      if (subSchema.required)
        requiredDuplicated.push(...subSchema.required);

      if (!items) {
        items = subSchema.items;
      }

      const flattened = flattenOfStatements(subSchema);

      if (flattened.properties) {
        Object.assign(properties, flattened.properties);
      }

      if (flattened.required) {
        requiredDuplicated.push(...flattened.required);
      }
    }
  } else if (schema.anyOf) {
    for (const subSchema of schema.anyOf) {
      if (!type) {
        type = subSchema.type;
      } else if (subSchema.type && subSchema.type !== type) {
        type = decideTypePrecedence(subSchema.type, type);
      }

      if (!items) {
        items = subSchema.items;
      }

      const flattened = flattenOfStatements(subSchema);
      if (flattened.properties) {
        Object.assign(properties, flattened.properties);
      }

      if (flattened.required) {
        requiredDuplicated.push(...flattened.required);

      }
    }
  } else if (schema.oneOf) {
    for (const subSchema of schema.oneOf) {
      if (!type) {
        type = subSchema.type;
      } else if (subSchema.type && subSchema.type !== type) {
        // Type precedence rules: string wins over numeric types
        type = decideTypePrecedence(subSchema.type, type);
      }

      if (!items) {
        items = subSchema.items;
      }

      const flattened = flattenOfStatements(subSchema);
      if (flattened.properties) {
        Object.assign(properties, flattened.properties);
      }

      if (flattened.required) {
        requiredDuplicated.push(...flattened.required);
      }
    }
  }

  // Apply schema's own properties on top to give them precedence
  Object.assign(properties, schema.properties);

  if (schema.required) {
    requiredDuplicated.push(...schema.required);
  }

  if (!items && schema.items) {
    items = schema.items;
  }

  // Deduplicate required properties
  const required =
    !type || type === "object"
      ? Array.from(new Set(requiredDuplicated))
      : undefined;

  if (type && type !== "object") {
    properties = undefined;
  }

  return {
    properties,
    required,
    items,
    type,
  }
}

/// Decide which type wins when openapi declares that a property can be more than one type
function decideTypePrecedence(t1: TypeValue, t2: TypeValue) {
  const precedenceKey = (t1: TypeValue, t2: TypeValue) => JSON.stringify([t1, t2].sort());

  const typePrecedence = new Map<string, string>([
    [precedenceKey('integer', 'string'), 'string'],
    [precedenceKey('number', 'string'), 'string'],
    [precedenceKey('integer', 'number'), 'number'],
  ]);

  const sortedKey = precedenceKey(t1, t2);
  const winningType = typePrecedence.get(sortedKey);
  if (!winningType) {
    throw new Error(`Could not find type precedence rule for types: ${t1} and ${t2}`);
  }

  return winningType;
}
