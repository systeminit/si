import assert from "node:assert";
import logger from "../../logger.ts";
import { OnlyProperties } from "../../spec/props.ts";
import { CfProperty, CfHandler, CfHandlerKind } from "../types.ts";
import {
  AzureOperationData,
  AzureSchema,
  PropertySet,
  isAzureObjectProperty,
  AzureProperty,
  AzureOpenApiDocument,
  AzureOpenApiOperation,
  AZURE_HTTP_METHODS,
} from "./schema.ts";
import { JSONSchema } from "../draft_07.ts";
import { ExpandedPkgSpec } from "../../spec/pkgs.ts";
import { makeModule } from "../generic/index.ts";
import { AZURE_PROVIDER_CONFIG } from "./provider.ts";

export function extractPropertiesFromRequestBody(
  operation: AzureOpenApiOperation | null,
) {
  const bodyParams = operation?.parameters?.filter((p) => p.in === "body");
  if (!bodyParams || bodyParams.length == 0)
    return { properties: {}, required: [] };
  assert(bodyParams.length <= 1, "Expected at most one body parameter");

  const { schema } = bodyParams[0];
  assert(isAzureObjectProperty(schema), "Body schema is not an object");

  return {
    properties: schema.properties ?? {},
    required: schema.required ?? [],
  };
}

export function extractPropertiesFromResponseBody(
  operation: AzureOpenApiOperation | null,
) {
  const schema = operation?.responses?.["200"]?.schema;
  if (!schema) return { properties: {}, required: [] };
  assert(
    isAzureObjectProperty(schema),
    `Response schema is not an object: ${operation.operationId}`,
  );

  return {
    properties: schema.properties ?? {},
    required: schema.required ?? [],
  };
}

/// Normalize a generic JSONSchema into an AzureProperty, *recursively*

/// Normalize a generic JSONSchema into an AzureProperty:
/// - "true" and "false" become { type: "string" }
/// - missing "type" is inferred from "format", "properties", or "items"
/// - Draft 4 exclusiveMinimum/Maximum bools converted to Draft 6+
/// - formats normalized to SI-supported ones
export function normalizeAzureProperty(prop: JSONSchema): AzureProperty {
  if (prop === true || prop === false) {
    prop = { type: "string" };
  }

  if (!prop.type) {
    // Infer type from format if present
    if (prop.format === "int32" || prop.format === "int64") {
      prop = { ...prop, type: "integer" };
    } else if (
      prop.format === "float" ||
      prop.format === "double" ||
      prop.format === "decimal"
    ) {
      prop = { ...prop, type: "number" };
    } else if (prop.properties) {
      prop = { ...prop, type: "object" };
    } else if (prop.items) {
      prop = { ...prop, type: "array" };
    } else if (!prop.oneOf && !prop.anyOf) {
      prop = { ...prop, type: "string" };
    }
  }

  if (prop.type) {
    const normalized = { ...prop };

    // Draft 4 -> Draft 6+ conversion (exclusiveMinimum/Maximum)
    for (const key of ["exclusiveMinimum", "exclusiveMaximum"] as const) {
      const baseKey = key === "exclusiveMinimum" ? "minimum" : "maximum";
      const val = normalized[key] as JSONSchema | boolean;
      const baseVal = prop[baseKey];
      if (val === true && typeof baseVal === "number") {
        delete normalized[baseKey];
      } else if (val === false) {
        delete normalized[key];
      }
    }

    // Normalize formats to SI-supported ones
    if (normalized.format) {
      const format = normalized.format as string;
      if (
        normalized.type === "integer" &&
        (format === "int32" || format === "int64")
      ) {
        normalized.format = "int64";
      } else if (
        normalized.type === "number" &&
        (format === "float" || format === "double" || format === "decimal")
      ) {
        normalized.format = "double";
      } else if (
        [
          "arm-id",
          "uuid",
          "email",
          "unixtime",
          "duration",
          "date",
          "date-time-rfc1123",
          "byte",
          "binary",
          "password",
        ].includes(format)
      ) {
        delete normalized.format;
      }
    }

    prop = normalized;
  }

  // Handle oneOf with primitives by merging in the first non-string type
  if (prop.oneOf) {
    assert(!prop.type);
    let type: "string" | "number" | "integer" | "boolean" | undefined;
    for (const oneOf of prop.oneOf.map(normalizeAzureProperty)) {
      switch (oneOf.type) {
        // use string only if nothing else is available
        case "string":
          type ??= "string";
          break;
        // number/integer/boolean overrides string
        case "number":
        case "integer":
        case "boolean":
          if (!type || type === "string") type = oneOf.type;
          break;
        // We can't smoosh object/array together with string (or don't know how yet)
        default:
          if (type) throw new Error();
          break;
      }
    }
    if (type) {
      prop = { ...prop, type };
      delete prop.oneOf;
    }
  }

  return prop as AzureProperty;
}

/// Normalize, with a seen list to check for cycles
function normalizeAzurePropertyRecursively(
  prop: JSONSchema,
  seen: Set<JSONSchema>,
): AzureProperty {
  // If there's a circular reference, we know we're already normalizing this prop, so we don't
  // do it twice.
  if (seen.has(prop)) return prop as AzureProperty;
  seen.add(prop);

  // First normalize the base types
  prop = normalizeAzureProperty(prop);

  // Recursively normalize properties, additionalProperties, and items
  if (prop.properties) {
    prop = {
      ...prop,
      properties: Object.fromEntries(
        Object.entries(prop.properties)
          .filter(([_k, v]) => typeof v === "object" && v !== null)
          .map(([k, v]) => [k, normalizeAzurePropertyRecursively(v, seen)]),
      ),
    };
  }

  // Normalize additionalProperties
  if (prop.additionalProperties) {
    prop = {
      ...prop,
      additionalProperties: normalizeAzurePropertyRecursively(
        prop.additionalProperties,
        seen,
      ),
    };
  }

  // Normalize items
  if (prop.items) {
    if (Array.isArray(prop.items)) {
      prop.items = prop.items.map((item) =>
        normalizeAzurePropertyRecursively(item, seen),
      ) as JSONSchema[];
    } else {
      prop.items = normalizeAzurePropertyRecursively(prop.items, seen);
    }
  }

  return prop as AzureProperty;
}

export function mergePropertyDefinitions(
  existing: AzureProperty | undefined,
  newProp: AzureProperty,
): AzureProperty {
  if (!existing) return newProp;
  assert(typeof existing !== "boolean");
  assert(typeof newProp !== "boolean");

  const merged = { ...existing };

  if ("enum" in merged) {
    assert("enum" in newProp, "Cannot merge enum with non-enum");
    const existingEnum = merged.enum as unknown[] | undefined;
    const newPropEnum = newProp.enum as unknown[] | undefined;
    if (existingEnum && newPropEnum) {
      merged.enum = [...new Set([...existingEnum, ...newPropEnum])];
    } else if (newPropEnum) {
      merged.enum = newPropEnum;
    }
  }

  if (newProp.description && !merged.description) {
    merged.description = newProp.description;
  }

  return merged;
}

export function buildHandlersFromOperations(operations: AzureOperationData[]) {
  const result: {
    handlers: { [key in CfHandlerKind]?: CfHandler };
    getOperation?: AzureOpenApiOperation;
    putOperation?: AzureOpenApiOperation;
    patchOperation?: AzureOpenApiOperation;
    deleteOperation?: AzureOpenApiOperation;
  } = { handlers: {} };

  const defaultHandler = { permissions: [], timeoutInMinutes: 60 };

  operations.forEach(({ method, openApiOperation }) => {
    const op = openApiOperation;
    const methodLower = method.toLowerCase();

    // If there's no schema or it's not an object, we can't merge its properties
    switch (methodLower) {
      case "get": {
        const isList = op.operationId?.includes("List");
        result.handlers[isList ? "list" : "read"] = defaultHandler;
        if (!isList) {
          result.getOperation = op;
        }
        break;
      }
      case "put": {
        result.putOperation = op;
        result.handlers["create"] = defaultHandler;
        result.handlers["update"] = defaultHandler;
        break;
      }
      case "patch": {
        result.patchOperation = op;
        result.handlers["update"] = defaultHandler;
        break;
      }
      case "delete": {
        result.deleteOperation = op;
        result.handlers["delete"] = defaultHandler;
        break;
      }
    }
  });

  return result;
}

export function mergeAzureResourceOperations(
  resourceType: string,
  operations: AzureOperationData[],
  apiVersion: string,
): ExpandedPkgSpec | null {
  const {
    handlers,
    getOperation,
    putOperation,
    patchOperation,
    deleteOperation,
  } = buildHandlersFromOperations(operations);

  if (!getOperation) {
    logger.debug(`No GET operation found for ${resourceType}`);
    return null;
  }

  if (!putOperation && !patchOperation && !deleteOperation) {
    // readonly schema! Skipping.
    logger.debug(
      `No PUT, PATCH or DELETE operations found for ${resourceType}`,
    );
    return null;
  }

  const description =
    getOperation.description ||
    (getOperation.summary as string) ||
    `Azure ${resourceType} resource`;

  const { properties: getProperties, required: getRequired } =
    extractPropertiesFromResponseBody(getOperation);

  if (!getProperties || Object.keys(getProperties).length === 0) {
    logger.debug(`No properties found in GET response for ${resourceType}`);
    return null;
  }

  const mergedProperties = { ...getProperties };
  const requiredProperties = new Set(getRequired);

  const getPropertySet: PropertySet = new Set(Object.keys(getProperties));
  const createUpdateProperties: PropertySet = new Set();
  const updateProperties: PropertySet = new Set();
  const deleteProperties: PropertySet = new Set();

  // Process write operations
  [
    {
      operation: putOperation,
      propertySet: createUpdateProperties,
      name: "put",
    },
    { operation: patchOperation, propertySet: updateProperties, name: "patch" },
    {
      operation: deleteOperation,
      propertySet: deleteProperties,
      name: "delete",
    },
  ].forEach(({ operation, propertySet }) => {
    if (!operation) return;

    const { properties: operationProps, required: operationRequired } =
      extractPropertiesFromRequestBody(operation);

    Object.keys(operationProps).forEach((prop) => propertySet.add(prop));
    operationRequired.forEach((prop) => requiredProperties.add(prop));

    Object.entries(operationProps).forEach(([key, prop]) => {
      if (key === "properties") {
        mergedProperties[key] = prop;
      } else {
        mergedProperties[key] = mergePropertyDefinitions(
          mergedProperties[key],
          prop,
        );
      }
    });
  });

  // Build onlyProperties
  const onlyProperties: OnlyProperties = {
    createOnly: [],
    readOnly: [],
    writeOnly: [],
    primaryIdentifier: ["id"],
  };

  // Recursively collect readOnly property paths from the schema
  function collectReadOnlyPaths(
    properties: JSONSchema,
    pathPrefix: string = "",
    seen: JSONSchema[] = [],
  ): string[] {
    if (seen.includes(properties)) return [];
    seen.push(properties);
    try {
      const paths: string[] = [];

      for (const [key, prop] of Object.entries(properties)) {
        const propSchema = prop;
        const currentPath = pathPrefix ? `${pathPrefix}/${key}` : key;

        if (propSchema?.readOnly === true) {
          paths.push(currentPath);
        }

        if (
          propSchema?.properties &&
          typeof propSchema.properties === "object"
        ) {
          paths.push(
            ...collectReadOnlyPaths(propSchema.properties, currentPath, seen),
          );
        }
      }

      return paths;
    } finally {
      seen.pop();
    }
  }

  // Collect all readOnly paths from the merged properties
  const readOnlyPaths = collectReadOnlyPaths(mergedProperties);
  onlyProperties.readOnly.push(...readOnlyPaths);

  // Also mark properties that are in GET but not in any write operations
  getPropertySet.forEach((prop) => {
    const isPrimaryIdentifier = onlyProperties.primaryIdentifier.includes(prop);
    const notInWrites =
      !createUpdateProperties.has(prop) &&
      !updateProperties.has(prop) &&
      !deleteProperties.has(prop);

    if (isPrimaryIdentifier || notInWrites) {
      if (!onlyProperties.readOnly.includes(prop)) {
        onlyProperties.readOnly.push(prop);
      }
    }
  });

  // createOnly: in PUT (create/update) but not in PATCH (update-only)
  if (patchOperation && updateProperties.size > 0) {
    createUpdateProperties.forEach((prop) => {
      if (!updateProperties.has(prop)) {
        onlyProperties.createOnly.push(`/${prop}`);
      }
    });
  }

  // writeOnly: in PUT/PATCH/DELETE but not in GET
  const writeProps = [
    ...createUpdateProperties,
    ...updateProperties,
    ...deleteProperties,
  ];
  onlyProperties.writeOnly = [
    ...new Set(
      writeProps
        .filter((prop) => !getPropertySet.has(prop))
        .map((prop) => `/${prop}`),
    ),
  ];

  // Split properties into domain (writable) and resource_value (readable)
  const readOnlySet = new Set(onlyProperties.readOnly);
  const domainProperties: Record<string, CfProperty> = {};
  const resourceValueProperties: Record<string, CfProperty> = {};

  for (const [name, prop] of Object.entries(mergedProperties)) {
    if (readOnlySet.has(name)) {
      resourceValueProperties[name] = prop;
    } else {
      domainProperties[name] = prop;
    }
  }

  const schema: AzureSchema = {
    typeName: resourceType,
    description,
    properties: mergedProperties,
    requiredProperties,
    primaryIdentifier: ["id"],
    handlers,
    apiVersion,
  };

  return makeModule(
    schema,
    description,
    onlyProperties,
    AZURE_PROVIDER_CONFIG,
    domainProperties,
    resourceValueProperties,
  );
}

function extractResourceTypeFromPath(path: string): string | null {
  // Form: /subscriptions/{subscriptionId}/resourceGroups/{resourceGroupName}/providers/Microsoft.Network/applicationGateways/{applicationGatewayName}
  const match = path.match(/\/providers\/([^/]+)\/([^/{]+)\/\{[^/}]+\}$/);
  if (!match) return null;
  const [_, providerNamespace, resourceType] = match;
  const serviceName = providerNamespace.split(".").pop() || providerNamespace;
  const capitalizedResource =
    resourceType.charAt(0).toUpperCase() + resourceType.slice(1);
  return `Azure::${serviceName}::${capitalizedResource}`;
}

export function parseAzureSpec(
  openApiDoc: AzureOpenApiDocument,
): ExpandedPkgSpec[] {
  const specs: ExpandedPkgSpec[] = [];

  if (!openApiDoc.paths) {
    console.warn("No paths found in Azure schema");
    return [];
  }

  const resourceOperations: Record<string, AzureOperationData[]> = {};

  for (const [path, methods] of Object.entries(openApiDoc.paths)) {
    const resourceType = extractResourceTypeFromPath(path);

    if (!resourceType) continue;
    if (resourceType.includes("Reminder: Need renaming")) continue; // lol
    // these are the supplemental actions endpoints. Skipping for now
    if (resourceType.endsWith("Operations")) continue;

    for (const method of AZURE_HTTP_METHODS) {
      const openApiOperation = methods?.[method];
      if (!openApiOperation) continue;

      if (!resourceOperations[resourceType]) {
        resourceOperations[resourceType] = [];
      }

      resourceOperations[resourceType].push({
        method,
        path,
        openApiOperation,
        apiVersion: undefined,
      });
    }
  }

  Object.entries(resourceOperations).forEach(([resourceType, operations]) => {
    const spec = mergeAzureResourceOperations(
      resourceType,
      operations,
      openApiDoc.info.version,
    );
    if (spec) {
      specs.push(spec);
    }
  });

  logger.debug(
    `Generated ${specs.length} schemas from ${
      Object.keys(resourceOperations).length
    } resource types`,
  );

  return specs;
}
