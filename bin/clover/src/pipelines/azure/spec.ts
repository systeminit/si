import assert from "node:assert";
import logger from "../../logger.ts";
import { OnlyProperties } from "../../spec/props.ts";
import { CfProperty, CfHandler, CfHandlerKind } from "../types.ts";
import {
  AzureOperationData,
  AzureSchema,
  PropertySet,
  isAzureObjectProperty,
  AZURE_HTTP_METHODS,
  AzureOpenApiOperation,
  AzureProperty,
  AzureOpenApiSpec,
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
  if (!isAzureObjectProperty(schema))
    console.log(`schema: ${JSON.stringify(schema, null, 2)}`);
  assert(isAzureObjectProperty(schema), "Expected body schema to be an object");

  return {
    properties: schema.properties ?? {},
    required: schema.required ?? [],
  };
}

export function extractPropertiesFromResponseBody(
  operation: AzureOpenApiOperation | null,
) {
  const schema = operation?.responses["200"]?.schema;
  if (!schema) return { properties: {}, required: [] };

  return {
    properties: schema.properties ?? {},
    required: schema.required ?? [],
  };
}

export function normalizeAzureProperty(prop: JSONSchema): AzureProperty {
  if (prop === true || prop === false) return { type: "string" };

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
      const val = normalized[key] as JSONSchema | boolean;
      const baseKey = key === "exclusiveMinimum" ? "minimum" : "maximum";
      if (val === true && typeof normalized[baseKey] === "number") {
        normalized[key] = normalized[baseKey];
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

    // Recursively normalize nested schemas
    if (normalized.properties) {
      normalized.properties = Object.fromEntries(
        Object.entries(normalized.properties)
          .filter(([_k, v]) => typeof v === "object" && v !== null)
          .map(([k, v]) => [k, normalizeAzureProperty(v)]),
      );
    }

    // Normalize additionalProperties and items (convert boolean to schema)
    for (const key of ["additionalProperties", "items"] as const) {
      if (normalized[key] === true) {
        normalized[key] = { type: "string" };
      } else if (normalized[key] && typeof normalized[key] === "object") {
        normalized[key] = normalizeAzureProperty(normalized[key] as JSONSchema);
      }
    }

    return normalized as AzureProperty;
  }

  // Handle oneOf with primitives by merging in the first non-string type
  if (prop.oneOf) {
    const primitives = prop.oneOf.map((m) =>
      typeof m === "object" &&
      typeof m.type === "string" &&
      ["string", "number", "integer", "boolean"].includes(m.type)
        ? m.type
        : undefined,
    );

    if (primitives.every((t) => t)) {
      const type = primitives.find((t) => t !== "string") ?? "string";
      const smooshed = { ...prop, type, oneOf: undefined };
      return normalizeAzureProperty(smooshed);
    }
  }

  return prop as AzureProperty;
}

export function mergePropertyDefinitions(
  existing: JSONSchema | undefined,
  newProp: JSONSchema,
): JSONSchema {
  if (!existing) return newProp;
  assert(typeof existing !== "boolean");
  assert(typeof newProp !== "boolean");

  const merged = { ...existing };

  const existingEnum = existing.enum as unknown[] | undefined;
  const newPropEnum = newProp.enum as unknown[] | undefined;
  if (existingEnum && newPropEnum) {
    merged.enum = [...new Set([...existingEnum, ...newPropEnum])];
  } else if (newPropEnum) {
    merged.enum = newPropEnum;
  }

  if (newProp.description && !existing.description) {
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

    switch (methodLower) {
      case "get": {
        const isList = op.operationId?.includes("List");
        result.handlers[isList ? "list" : "read"] = defaultHandler;
        if (!isList || !result.getOperation) {
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
  ): string[] {
    const paths: string[] = [];

    for (const [key, prop] of Object.entries(properties)) {
      const propSchema = prop;
      const currentPath = pathPrefix ? `${pathPrefix}/${key}` : key;

      if (propSchema?.readOnly === true) {
        paths.push(currentPath);
      }

      if (propSchema?.properties && typeof propSchema.properties === "object") {
        paths.push(...collectReadOnlyPaths(propSchema.properties, currentPath));
      }
    }

    return paths;
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

  const normalizedProperties: Record<string, CfProperty> = {};
  for (const [key, prop] of Object.entries(mergedProperties)) {
    if (typeof prop !== "object" || prop === null) continue;
    normalizedProperties[key] = normalizeAzureProperty(prop) as CfProperty;
  }

  // Split properties into domain (writable) and resource_value (readable)
  const readOnlySet = new Set(onlyProperties.readOnly);
  const domainProperties: Record<string, CfProperty> = {};
  const resourceValueProperties: Record<string, CfProperty> = {};

  for (const [name, prop] of Object.entries(normalizedProperties)) {
    if (readOnlySet.has(name)) {
      resourceValueProperties[name] = prop;
    } else {
      domainProperties[name] = prop;
    }
  }

  const schema: AzureSchema = {
    typeName: resourceType,
    description,
    properties: normalizedProperties,
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

function isResourcePath(path: string): boolean {
  const match = path.match(
    /providers\/([^/]+)\/([^/{]+)(\/\{[^}]+\})?(\/(.+))?/,
  );
  if (!match) return false;
  return !match[4];
}

function extractResourceTypeFromPath(path: string): string | null {
  const match = path.match(/providers\/([^/]+)\/([^/{]+)/);
  if (match) {
    const providerNamespace = match[1];
    const serviceName = providerNamespace.split(".").pop() || providerNamespace;
    const resourceType = match[2];
    const capitalizedResource =
      resourceType.charAt(0).toUpperCase() + resourceType.slice(1);
    return `Azure::${serviceName}::${capitalizedResource}`;
  }
  return null;
}

export function parseAzureSpec(
  openApiSpec: AzureOpenApiSpec,
): ExpandedPkgSpec[] {
  const specs: ExpandedPkgSpec[] = [];

  if (!openApiSpec.paths) {
    console.warn("No paths found in Azure schema");
    return [];
  }

  const resourceOperations: Record<string, AzureOperationData[]> = {};

  Object.entries(openApiSpec.paths).forEach(([path, operations]) => {
    if (!isResourcePath(path)) return;

    AZURE_HTTP_METHODS.forEach((method) => {
      const openApiOperation = operations[method];
      if (openApiOperation) {
        const resourceType = extractResourceTypeFromPath(path);

        if (!resourceType) return;
        if (resourceType.includes("Reminder: Need renaming")) return; // lol
        // these are the supplemental actions endpoints. Skipping for now
        if (resourceType.endsWith("Operations")) return;

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
    });
  });

  Object.entries(resourceOperations).forEach(([resourceType, operations]) => {
    const spec = mergeAzureResourceOperations(
      resourceType,
      operations,
      openApiSpec.info.version,
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
