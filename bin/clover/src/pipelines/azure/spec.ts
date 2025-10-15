import logger from "../../logger.ts";
import { OnlyProperties } from "../../spec/props.ts";
import type { CfProperty } from "../types.ts";
import { CfHandler, CfHandlerKind } from "../types.ts";
import { ExpandedPkgSpec } from "../../spec/pkgs.ts";
import { makeModule } from "../generic/index.ts";
import { azureProviderConfig } from "./provider.ts";
import {
  type AzureOperationData,
  type AzureSchema,
  type JsonSchema,
  type PropertySet,
} from "./schema.ts";

export function extractPropertiesFromRequestBody(
  operation: JsonSchema | null,
): { properties: JsonSchema; required: string[] } {
  const parameters = operation?.parameters as JsonSchema[] | undefined;
  const bodyParam = parameters?.find((p) => (p.in as string) === "body");
  const schema = bodyParam?.schema as JsonSchema | undefined;

  if (!schema) return { properties: {}, required: [] };

  const properties = (schema?.properties as JsonSchema) || {};

  return {
    properties,
    required: (schema?.required as string[]) || [],
  };
}

export function extractPropertiesFromResponseBody(
  operation: JsonSchema | null,
): { properties: JsonSchema; required: string[] } {
  const response200 = (operation?.responses as any)?.["200"];
  const schema = response200?.schema as JsonSchema | undefined;

  if (!schema) return { properties: {}, required: [] };

  const properties = (schema?.properties as JsonSchema) || {};

  return {
    properties,
    required: (schema?.required as string[]) || [],
  };
}

export function normalizeAzureProperty(prop: JsonSchema | boolean): JsonSchema {
  if (prop === true || prop === false) return { type: "string" };

  if (!prop.type) {
    // Infer type from format if present
    if (prop.format === "int32" || prop.format === "int64") {
      prop = { ...prop, type: "integer" };
    } else if (
      prop.format === "float" || prop.format === "double" ||
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
      if (normalized[key] === true && typeof normalized[baseKey] === "number") {
        normalized[key] = normalized[baseKey];
        delete normalized[baseKey];
      } else if (normalized[key] === false) {
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
        Object.entries(normalized.properties as Record<string, JsonSchema>)
          .filter(([_k, v]) => typeof v === "object" && v !== null)
          .map(([k, v]) => [k, normalizeAzureProperty(v)]),
      );
    }

    // Normalize additionalProperties and items (convert boolean to schema)
    for (const key of ["additionalProperties", "items"] as const) {
      if (normalized[key] === true) {
        normalized[key] = { type: "string" };
      } else if (normalized[key] && typeof normalized[key] === "object") {
        normalized[key] = normalizeAzureProperty(normalized[key] as JsonSchema);
      }
    }

    return normalized;
  }

  // Handle oneOf with primitives
  if (prop.oneOf) {
    const allPrimitives = (prop.oneOf as JsonSchema[]).every((m) => {
      return ["string", "number", "integer", "boolean"].includes(
        m.type as string,
      );
    });

    if (allPrimitives) {
      const nonString = (prop.oneOf as JsonSchema[]).find((m) =>
        m.type !== "string"
      );
      const smooshed = nonString
        ? { ...prop, type: nonString.type, oneOf: undefined }
        : { ...prop, type: "string", oneOf: undefined };
      return normalizeAzureProperty(smooshed);
    }
  }

  return prop;
}

export function mergePropertyDefinitions(
  existing: JsonSchema | undefined,
  newProp: JsonSchema,
): JsonSchema {
  if (!existing) return newProp;

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
        const opId = op.operationId as string;
        const isList = opId?.includes("List");
        handlers[isList ? "list" : "read"] = defaultHandler;
        if (!isList || !getOperation) {
          getOperation = op;
        }
        break;
      }
      case "put": {
        putOperation = op;
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

  const description = (getOperation.description as string) ||
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
          mergedProperties[key] as JsonSchema,
          prop as JsonSchema,
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
    properties: JsonSchema,
    pathPrefix: string = "",
  ): string[] {
    const paths: string[] = [];

    for (const [key, prop] of Object.entries(properties)) {
      const propSchema = prop as JsonSchema;
      const currentPath = pathPrefix ? `${pathPrefix}/${key}` : key;

      if (propSchema?.readOnly === true) {
        paths.push(currentPath);
      }

      if (propSchema?.properties && typeof propSchema.properties === "object") {
        paths.push(
          ...collectReadOnlyPaths(
            propSchema.properties as JsonSchema,
            currentPath,
          ),
        );
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
    const notInWrites = !createUpdateProperties.has(prop) &&
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
      writeProps.filter((prop) => !getPropertySet.has(prop)).map((prop) =>
        `/${prop}`
      ),
    ),
  ];

  const normalizedProperties: Record<string, CfProperty> = {};
  for (const [key, prop] of Object.entries(mergedProperties)) {
    if (typeof prop !== "object" || prop === null) continue;
    try {
      normalizedProperties[key] = normalizeAzureProperty(
        prop as JsonSchema,
      ) as CfProperty;
    } catch (error) {
      const errorMessage = error instanceof Error
        ? error.message
        : String(error);
      console.warn(
        `Failed to normalize property ${key} in ${resourceType}: ${errorMessage}`,
      );
    }
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
    properties: normalizedProperties as Record<string, CfProperty>,
    requiredProperties,
    primaryIdentifier: ["id"],
    handlers,
    apiVersion,
  };

  return makeModule(
    schema,
    description,
    onlyProperties,
    azureProviderConfig,
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
    const capitalizedResource = resourceType.charAt(0).toUpperCase() +
      resourceType.slice(1);
    return `Azure::${serviceName}::${capitalizedResource}`;
  }
  return null;
}

export function parseAzureSchema(rawSchema: unknown): ExpandedPkgSpec[] {
  const schema = rawSchema as JsonSchema;
  const specs: ExpandedPkgSpec[] = [];

  if (!schema.paths) {
    console.warn("No paths found in Azure schema");
    return [];
  }

  const resourceOperations: Record<string, AzureOperationData[]> = {};

  Object.entries(schema.paths as JsonSchema).forEach(([path, pathItem]) => {
    if (!isResourcePath(path)) return;

    const methods = ["get", "put", "patch", "delete", "post"];

    methods.forEach((method) => {
      if ((pathItem as JsonSchema)[method]) {
        const operation = (pathItem as JsonSchema)[method] as JsonSchema;
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
          openApiOperation: operation,
          apiVersion: undefined,
        });
      }
    });
  });

  const schemaInfo = schema.info as JsonSchema | undefined;
  const apiVersion = schemaInfo?.version as string | undefined || "2023-01-01";

  Object.entries(resourceOperations).forEach(([resourceType, operations]) => {
    const spec = mergeAzureResourceOperations(
      resourceType,
      operations,
      apiVersion,
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
