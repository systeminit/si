import _ from "lodash";
import { OnlyProperties } from "../../spec/props.ts";
import type { CfProperty } from "../types.ts";
import { CfHandler, CfHandlerKind } from "../types.ts";
import {
  type EntraOpenApiDocument,
  type EntraOpenApiOperation,
  type EntraOpenApiRequestBody,
  type EntraOpenApiResponse,
  type EntraSchema,
  type JsonSchemaObject,
  type JsonSchemaObjectOnly,
  type NormalizedEntraSchema,
  type OperationData,
  type PropertySet,
} from "./schema.ts";

export function mergeResourceOperations(
  resourceName: string,
  operations: OperationData[],
  description: string | undefined,
  allSchemas: EntraOpenApiDocument,
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

  // Filter out OData metadata properties - these are server-generated and not user-controllable
  const filteredDomainProperties = Object.fromEntries(
    Object.entries(writableDomainProperties).filter(([key, _]) =>
      !isODataMetadataProperty(key)
    ),
  );

  // Also filter OData metadata from resource_value properties
  const filteredResourceValueProperties = Object.fromEntries(
    Object.entries(resourceValueProperties).filter(([key, _]) =>
      !isODataMetadataProperty(key)
    ),
  );

  // Extract schemas for discriminator expansion
  const schemas = allSchemas.components?.schemas as
    | Record<string, JsonSchemaObject>
    | undefined;
  const normalizer = new EntraNormalizer(schemas);

  // Normalize domain properties (POST + PATCH = writable)
  const normalizedDomainProperties = Object.fromEntries(
    Object.entries(filteredDomainProperties)
      .map(([key, prop]) => {
        const normalized = normalizer.normalize(prop);
        // Skip properties that are part of cycles
        if (!normalized) return null;
        return [key, normalized];
      })
      .filter((entry): entry is [string, NormalizedEntraSchema] =>
        entry !== null
      ),
  );

  // Normalize resource_value properties (GET response = readable)
  const normalizedResourceValueProperties = Object.fromEntries(
    Object.entries(filteredResourceValueProperties)
      .map(([key, prop]) => {
        const normalized = normalizer.normalize(prop);
        // Skip properties that are part of cycles
        if (!normalized) return null;
        return [key, normalized];
      })
      .filter((entry): entry is [string, NormalizedEntraSchema] =>
        entry !== null
      ),
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

// Helper: Expand discriminators by finding matching subtypes and adding them as properties
function expandEntraDiscriminators(
  discriminator: JsonSchemaObjectOnly["discriminator"],
  properties: JsonSchemaObjectOnly["properties"],
  schemas: Record<string, JsonSchemaObject>,
): {
  expandedProperties: Record<string, JsonSchemaObject>;
  discriminators: Record<string, Record<string, string>>;
} | undefined {
  if (!discriminator || typeof discriminator !== "object") return undefined;
  if (!properties) return undefined;

  const discriminatorProp = discriminator.propertyName;
  const mapping = discriminator.mapping;

  if (!discriminatorProp || !mapping) return undefined;

  // Skip @odata.type discriminators - they have 1055+ subtypes (all entity types)
  // and are not useful for property type narrowing
  if (discriminatorProp === "@odata.type") {
    return undefined;
  }

  const expandedProperties: Record<string, JsonSchemaObject> = {
    ...properties,
  };

  // Replace the discriminator field with an object containing the subtypes
  const discriminatorObject: JsonSchemaObject = {
    type: "object",
    properties: {},
  };

  const discriminatorMap: Record<string, string> = {};

  // Format: "#microsoft.graph.application": "#/components/schemas/microsoft.graph.application"
  for (const [discriminatorValue, schemaRef] of Object.entries(mapping)) {
    // Extract schema name from ref like "#/components/schemas/microsoft.graph.application"
    const schemaName = String(schemaRef).replace(
      "#/components/schemas/",
      "",
    );

    const subtypeSchema = schemas[schemaName];
    if (!subtypeSchema) continue;

    // Handle boolean schemas
    if (typeof subtypeSchema === "boolean") continue;

    // Flatten allOf to get all properties from parent and child schemas
    const flattened = flattenSchemaProperties(
      subtypeSchema as NormalizedEntraSchema,
    );

    // Store the subtype's properties (flattened from allOf)
    const subtypeProps: JsonSchemaObject = {
      type: "object",
      description: subtypeSchema.description,
      properties: flattened.properties,
      required: flattened.required,
    };

    // Use schema name as the key (cleaner than the full discriminator value)
    discriminatorObject.properties![schemaName] = subtypeProps;
    discriminatorMap[schemaName] = discriminatorValue;
  }

  expandedProperties[discriminatorProp] = discriminatorObject;

  const discriminators = {
    [discriminatorProp]: discriminatorMap,
  };

  return {
    expandedProperties,
    discriminators,
  };
}

// Helper: Merge two schemas by intersecting their properties
function intersectEntraSchema(
  prop: NormalizedEntraSchema,
  intersected: NormalizedEntraSchema,
): NormalizedEntraSchema {
  if (!prop) return intersected;
  if (!intersected) return prop;

  // Merge properties from both schemas
  for (const key of Object.keys(prop) as (keyof NormalizedEntraSchema)[]) {
    if (prop[key] === undefined) continue;

    switch (key) {
      // Merge nested properties recursively
      case "properties": {
        if (!prop.properties) break;
        intersected.properties ??= {};
        for (const [propName, childProp] of Object.entries(prop.properties)) {
          intersected.properties[propName] = intersectEntraSchema(
            childProp as NormalizedEntraSchema,
            intersected.properties[propName] as NormalizedEntraSchema,
          );
        }
        break;
      }

      // Merge array items and additionalProperties
      case "items":
      case "additionalProperties": {
        intersected[key] = intersectEntraSchema(
          prop[key] as NormalizedEntraSchema,
          intersected[key] as NormalizedEntraSchema,
        );
        break;
      }

      // Merge arrays without duplicates
      case "enum":
      case "required": {
        if (prop[key]) {
          intersected[key] = (intersected[key] ?? []).concat(
            prop[key].filter((v) => !intersected[key]?.includes(v)),
          );
        }
        break;
      }

      // Override with new values
      case "title":
      case "description":
        intersected[key] = prop[key];
        break;

      // Skip these - already handled
      case "allOf":
      case "anyOf":
      case "oneOf":
        break;

      // Prefer false (writable) over true (readonly)
      case "readOnly": {
        if (intersected.readOnly !== undefined) {
          intersected.readOnly = intersected.readOnly && prop.readOnly;
        } else {
          intersected.readOnly = prop.readOnly;
        }
        break;
      }

      // For other fields, just copy if not already set
      default: {
        if (intersected[key] === undefined) {
          intersected[key] = prop[key];
        }
        break;
      }
    }
  }

  return intersected;
}

// Helper: Union two schemas (used for anyOf/oneOf)
const MORE_SPECIFIC_THAN_STRING = ["number", "integer", "boolean"];

function unionEntraSchema(
  prop: NormalizedEntraSchema | undefined,
  unioned: NormalizedEntraSchema | undefined,
): NormalizedEntraSchema | undefined {
  if (!prop) return unioned;
  if (!unioned) return { ...prop };

  // Prefer more specific types over string
  if (
    unioned.type === "string" &&
    MORE_SPECIFIC_THAN_STRING.includes(prop.type as string)
  ) {
    unioned.type = prop.type;
  } else if (
    prop.type === "string" &&
    MORE_SPECIFIC_THAN_STRING.includes(unioned.type as string)
  ) {
    prop.type = unioned.type;
  }

  return intersectEntraSchema(prop, unioned);
}

// Maximum depth of full resource property expansion (below this, resources are assumed
// to be references and will just have "id")
const MAX_EXPANDED_RESOURCE_DEPTH = 1;
const MAX_NORMALIZATION_DEPTH = 10;

export class EntraNormalizer {
  constructor(schemas?: Record<string, JsonSchemaObject>) {
    this.schemas = schemas;
  }

  normalizing: JsonSchemaObject[] = [];
  schemas?: Record<string, JsonSchemaObject>;
  discriminatorCollector: Record<string, Record<string, string>> = {};

  /// Normalize a general JSONSchema from Entra into simpler format without nesting
  normalize(prop: JsonSchemaObject): NormalizedEntraSchema;
  normalize(
    prop: JsonSchemaObject | undefined,
  ): NormalizedEntraSchema | undefined;
  normalize(
    prop: JsonSchemaObject | undefined,
  ): NormalizedEntraSchema | undefined {
    // This is only meant to be called at the top level, non-recursively
    if (prop === undefined) return undefined;
    if (this.normalizing.length !== 0) {
      throw new Error("normalize() should only be called at top level");
    }
    const result = this.normalizeOrCycle(prop, "", 0);
    if (!result) return undefined;

    // Stub deeply nested resources
    this.stubResourceReferences(result, 0);

    return result;
  }

  private normalizeOrCycle(
    prop: JsonSchemaObject,
    path = "",
    depth = 0,
  ): NormalizedEntraSchema | undefined {
    // Check for excessive depth
    if (depth > MAX_NORMALIZATION_DEPTH) {
      return undefined;
    }

    // Check for cycles BEFORE any object copying
    if (this.normalizing.includes(prop)) {
      return undefined; // Cycle detected
    }

    this.normalizing.push(prop); // Track ORIGINAL object

    try {
      // Flatten the schema, merging allOf props and such, before we normalize type/format
      const normalized = this.flatten(prop, {}, path, depth);
      if (normalized === undefined) return undefined;

      // Infer type from properties / items if missing
      normalized.type ??= this.inferType(normalized);

      // Normalize formats to SI-supported ones
      if (normalized.format) {
        const format = normalized.format as string;
        if (
          [
            "duration",
            "uuid",
            "email",
            "date",
            "time",
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
          (format === "int16" || format === "int32" || format === "int64" ||
            format === "uint8")
        ) {
          normalized.format = "int64";
        } // Normalize number formats to double
        else if (
          normalized.type === "number" &&
          (format === "float" || format === "double" || format === "decimal" ||
            format === "int16" || format === "int32" || format === "uint8")
        ) {
          normalized.format = "double";
        }
      }

      // At the root level, attach collected discriminators
      if (
        this.discriminatorCollector &&
        Object.keys(this.discriminatorCollector).length > 0
      ) {
        normalized.discriminators = this.discriminatorCollector;
      }

      return normalized;
    } finally {
      this.normalizing.pop(); // Always clean up
    }
  }

  private inferType(prop: NormalizedEntraSchema): string | undefined {
    if (prop.type) {
      // Handle array of types (e.g., ["string", "null"])
      if (Array.isArray(prop.type)) {
        // Return the first non-null type, or the first type if all are null
        const nonNullType = prop.type.find((t) => t !== "null");
        return (nonNullType || prop.type[0]) as string;
      }
      return prop.type as string;
    }
    if (prop.properties || prop.additionalProperties) return "object";
    if (prop.items) return "array";
    return undefined;
  }

  private flatten(
    schemaProp: JsonSchemaObject,
    flattened: NormalizedEntraSchema = {},
    path = "",
    depth = 0,
  ): NormalizedEntraSchema | undefined {
    // Handle boolean schemas up front (JSON Schema allows true/"any" and false/"never")
    if (schemaProp === false) {
      throw new Error(
        `Boolean schema 'false' (never) not supported at path: ${
          path || "(root)"
        }`,
      );
    }
    // "any" becomes empty schema (which matches anything)
    if (schemaProp === true) {
      schemaProp = {};
    }

    // Pull off the stuff we're removing and children we're normalizing
    const {
      oneOf,
      anyOf,
      allOf,
      properties,
      items,
      additionalProperties,
      discriminator,
      ...rest
    } = schemaProp;

    let expandedProperties = properties;

    // Merge oneOf and anyOf by normalizing each alternative and then merging them
    if (oneOf) {
      for (const alternative of oneOf) {
        const child = this.normalizeOrCycle(alternative, path, depth + 1);
        if (!child) continue; // Skip cycles
        unionEntraSchema(child, flattened);
      }
    }
    if (anyOf) {
      for (const alternative of anyOf) {
        const child = this.normalizeOrCycle(alternative, path, depth + 1);
        if (!child) continue; // Skip cycles
        unionEntraSchema(child, flattened);
      }
    }

    // Merge allOf types into the flattened type
    // We don't normalize here, because allOf children can be *partial* properties
    if (allOf) {
      for (const alternative of allOf) {
        this.flatten(alternative, flattened, path, depth);
      }
    }

    // Expand discriminators AFTER allOf processing
    if (this.schemas) {
      const expansion = expandEntraDiscriminators(
        discriminator,
        properties,
        this.schemas,
      );
      if (expansion) {
        expandedProperties = expansion.expandedProperties;

        // Add discriminators to collector
        if (this.discriminatorCollector) {
          for (
            const [key, subtypeMap] of Object.entries(expansion.discriminators)
          ) {
            this.discriminatorCollector[key] = subtypeMap;
          }
        }
      }
    }

    // Normalize child schemas (properties, items, etc.)
    const prop: NormalizedEntraSchema = rest as NormalizedEntraSchema;
    if (expandedProperties) {
      prop.properties = {};
      if (Object.keys(expandedProperties).length > 0) {
        for (
          const [propName, childProp] of Object.entries(expandedProperties)
        ) {
          const child = this.normalizeOrCycle(
            childProp,
            path ? `${path}.${propName}` : propName,
            depth + 1,
          );
          if (!child) continue; // If the prop is part of a cycle, don't include it
          prop.properties[propName] = child;
        }
        // If all props were part of cycles, this prop is part of the cycle
        if (Object.keys(prop.properties).length == 0) return undefined;
      }
    }

    if (items) {
      prop.items = this.normalizeOrCycle(
        items,
        path ? `${path}[items]` : "[items]",
        depth + 1,
      );
      if (prop.items === undefined) return undefined;
    }

    if (additionalProperties) {
      prop.additionalProperties = this.normalizeOrCycle(
        additionalProperties,
        path ? `${path}[additionalProperties]` : "[additionalProperties]",
        depth + 1,
      );
      if (prop.additionalProperties === undefined) return undefined;
    }

    // Finally, intersect the props together
    intersectEntraSchema(prop, flattened);

    return flattened;
  }

  private stubResourceReferences(
    prop: NormalizedEntraSchema | undefined,
    resourceDepth: number,
  ): void {
    if (!prop) return;

    // Detect Microsoft Graph resources/entities
    // Graph entities have an 'id' property and typically multiple other properties
    if (
      prop.properties &&
      "id" in prop.properties &&
      Object.keys(prop.properties).length >= 3
    ) {
      resourceDepth += 1;

      // Stub the resource if we're at max depth - keep only the ID!
      if (resourceDepth > MAX_EXPANDED_RESOURCE_DEPTH) {
        prop.properties = { id: prop.properties.id };
        return;
      }
    }

    // Recursively check nested properties
    if (prop.properties) {
      for (const childProp of Object.values(prop.properties)) {
        this.stubResourceReferences(
          childProp as NormalizedEntraSchema,
          resourceDepth,
        );
      }
    }

    // Arrays/maps don't increment depth counter
    if (prop.items) {
      this.stubResourceReferences(
        prop.items as NormalizedEntraSchema,
        resourceDepth,
      );
    }
    if (prop.additionalProperties) {
      this.stubResourceReferences(
        prop.additionalProperties as NormalizedEntraSchema,
        resourceDepth,
      );
    }
  }
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
    if (valueSchema.type === "array" || valueSchema.items !== undefined) {
      return true;
    }
  }

  return false;
}

function isODataMetadataProperty(propName: string): boolean {
  return propName.startsWith("@odata.");
}
