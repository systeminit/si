import assert from "node:assert";
import logger from "../../logger.ts";
import { CfHandler, CfHandlerKind } from "../types.ts";
import {
  AzureOperationData,
  AzureSchema,
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
import { OnlyProperties } from "../../spec/props.ts";
import { assertUnreachable } from "../../assertUnreachable.ts";

const IGNORE_RESOURCE_TYPES = new Set<string>([
  // These don't have an id property in the GET response; TODO look into this
  "Azure::Portal::Consoles",
  "Azure::Portal::UserSettings",
]);

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
    if (IGNORE_RESOURCE_TYPES.has(resourceType)) continue;

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

function buildHandlersFromOperations(operations: AzureOperationData[]) {
  const result: {
    handlers: { [key in CfHandlerKind]?: CfHandler };
    getOperation?: AzureOpenApiOperation;
    putOperation?: AzureOpenApiOperation;
    // patchOperation?: AzureOpenApiOperation;
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
      // case "patch": {
      //   result.patchOperation = op;
      //   result.handlers["update"] = defaultHandler;
      //   break;
      // }
      case "delete": {
        result.deleteOperation = op;
        result.handlers["delete"] = defaultHandler;
        break;
      }
    }
  });

  return result;
}

function mergeAzureResourceOperations(
  resourceType: string,
  operations: AzureOperationData[],
  apiVersion: string,
): ExpandedPkgSpec | null {
  const { handlers, getOperation, putOperation } =
    buildHandlersFromOperations(operations);

  if (!getOperation) {
    logger.debug(`No GET operation found for ${resourceType}`);
    return null;
  }
  if (!putOperation) {
    // readonly schema! Skipping.
    logger.debug(`No PUT operation found for ${resourceType}`);
    return null;
  }

  // Grab resourceValue properties from the GET response
  const { properties: resourceValueProperties } =
    extractPropertiesFromResponseBody(getOperation);
  if (Object.keys(resourceValueProperties).length === 0) {
    logger.debug(`No properties found in GET response for ${resourceType}`);
    return null;
  }
  if (!("id" in resourceValueProperties)) {
    throw new Error(
      `No id property in GET response: ${getOperation.operationId}`,
    );
  }

  // Grab domain properties from the PUT request
  let { properties: domainProperties, required: requiredProperties } =
    extractPropertiesFromRequestBody(putOperation);
  // Remove readonly properties from the domain
  domainProperties = removeReadOnlyProperties(domainProperties, new Set());
  if (Object.keys(domainProperties).length === 0) {
    logger.debug(`No properties found in GET response for ${resourceType}`);
    return null;
  }

  const description =
    getOperation.description ||
    (getOperation.summary as string) ||
    `Azure ${resourceType} resource`;

  const primaryIdentifier = ["id"];
  const schema: AzureSchema = {
    typeName: resourceType,
    description,
    requiredProperties: new Set(requiredProperties),
    handlers,
    apiVersion,
  };
  // TODO figure out readOnly and writeOnly properties based on which tree they appear in
  const onlyProperties: OnlyProperties = {
    createOnly: [],
    readOnly: [],
    writeOnly: [],
    primaryIdentifier,
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

function extractPropertiesFromRequestBody(
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

function extractPropertiesFromResponseBody(
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

function removeReadOnlyProperties(
  properties: Record<string, AzureProperty>,
  seen: Set<AzureProperty>,
): Record<string, AzureProperty> {
  return Object.fromEntries(
    Object.entries(properties)
      .map(([key, childProp]) => [key, removeReadOnlyProperty(childProp, seen)])
      .filter(([_, childProp]) => childProp !== undefined),
  );
}

function removeReadOnlyProperty(
  property: AzureProperty | undefined,
  seen: Set<AzureProperty>,
): AzureProperty | undefined {
  if (!property) return undefined;
  if (seen.has(property)) return undefined;
  seen.add(property);

  // If *this* property is readOnly, it should be removed.
  if (property.readOnly) {
    return undefined;
  }
  switch (property.type) {
    case "object":
    case undefined: {
      if (property.properties) {
        const properties = removeReadOnlyProperties(property.properties, seen);
        if (Object.keys(properties).length === 0) return undefined;
        property = { ...property, properties };
      }
      break;
    }
    case "array": {
      // If the items are readOnly, the array is readOnly
      const items = removeReadOnlyProperty(property.items, seen);
      if (!items) return undefined;
      property.items = items;
      break;
    }
    case "boolean":
    case "integer":
    case "number":
    case "string":
    case "json":
      break;
    default:
      // Probably *shouldn't* have this by now? But if it does, one side or the other is always primitive
      if (!Array.isArray(property.type)) {
        assertUnreachable(property.type);
      }
      break;
  }
  return property;
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
