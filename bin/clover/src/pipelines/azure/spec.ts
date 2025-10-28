import assert from "node:assert";
import util from "node:util";
import logger from "../../logger.ts";
import { CfHandler, CfHandlerKind } from "../types.ts";
import {
  AzureSchema,
  AzureProperty,
  AzureOpenApiDocument,
  AzureOpenApiOperation,
  NormalizedAzureSchema,
} from "./schema.ts";
import { JSONSchema } from "../draft_07.ts";
import { ExpandedPkgSpec } from "../../spec/pkgs.ts";
import { makeModule } from "../generic/index.ts";
import { AZURE_PROVIDER_CONFIG } from "./provider.ts";
import { OnlyProperties } from "../../spec/props.ts";

/// Maximum resource depth (including the top level resource)
const MAX_RESOURCE_DEPTH = 3;

/// Add schemas (e.g. Azure::Portal::UserSettings) to ignore them until it's fixed
/// Make sure you also add a comment explaining why--all schemas should be supported!
const IGNORE_RESOURCE_TYPES = new Set<string>([
  // GET endpoint returns an array
  "Microsoft.PowerBI/privateLinkServicesForPowerBI"

]);

export function parseAzureSpec(
  openApiDoc: AzureOpenApiDocument,
): ExpandedPkgSpec[] {
  const specs: ExpandedPkgSpec[] = [];

  if (!openApiDoc.paths) {
    logger.warn("No paths found in Azure schema");
    return [];
  }

  const defaultHandler = { permissions: [], timeoutInMinutes: 60 };
  const resourceOperations: Record<
    string,
    {
      getOperation?: AzureOpenApiOperation;
      putOperation?: AzureOpenApiOperation;
      handlers: { [key in CfHandlerKind]?: CfHandler };
    }
  > = {};

  // Collect all operations for each resource type
  for (const [path, methods] of Object.entries(openApiDoc.paths)) {
    const pathInfo = parseEndpointPath(path);
    if (!pathInfo || !methods) continue;
    const resourceType = `${pathInfo.resourceProvider}/${pathInfo.resourceType}`;

    // Presently we only support Microsoft. providers
    if (!pathInfo.resourceProvider.toLowerCase().startsWith("microsoft.")) continue;
    // Ignore certain problematic resource types (temporarily until we fix them)
    if (IGNORE_RESOURCE_TYPES.has(resourceType)) continue;

    resourceOperations[resourceType] ??= { handlers: {} };
    const resource = resourceOperations[resourceType];

    if (pathInfo.resourceNameParam) {
      // If it has a /providers/<provider>/<resource-type>/{resourceName}, it's a CRUD op
      if (methods.get) {
        resource.getOperation = methods.get;
        resource.handlers.read = defaultHandler;
      }
      if (methods.put) {
        resource.putOperation = methods.put;
        resource.handlers.create = defaultHandler;
        resource.handlers.update = defaultHandler;
      }
      if (methods.delete) {
        resource.handlers.delete = defaultHandler;
      }
    } else {
      // It may be a list operation if you don't have to pass in the name
      if (methods.get && isListOperation(methods.get)) {
        resource.handlers.list = defaultHandler;
      }
    }
  }

  // Build specs from collected operations
  for (const [resourceType, resource] of Object.entries(resourceOperations)) {
    if (!resource.getOperation) {
      logger.debug(`No GET operation found for ${resourceType}`);
      continue;
    }
    if (!resource.putOperation) {
      // readonly schema! Skipping.
      logger.debug(`No PUT operation found for ${resourceType}`);
      continue;
    }

    const spec = buildDomainAndResourceValue(
      resourceType,
      resource.getOperation,
      resource.putOperation,
      resource.handlers,
      openApiDoc.info.version,
    );
    if (spec) {
      specs.push(spec);
    }
  }

  logger.debug(`Generated ${specs.length} schemas`);

  return specs;
}

/// Normalize a generic JSONSchema into an AzureProperty:
/// - "true" (any) is not supported
/// - "false" (never) yield undefined (no property)
/// - allOf/oneOf/anyOf are flattened/merged
/// - missing "type" is inferred from "format", "properties", or "items"
/// - Draft 4 exclusiveMinimum/Maximum bools converted to Draft 6+
/// - formats normalized to SI-supported ones
///
/// Recursively normalizes child properties as well.
export function normalizeAzureSchema(
  prop: JSONSchema | undefined,
  processing: JSONSchema[],
): NormalizedAzureSchema | undefined {
  if (prop === undefined) return undefined;
  if (processing.includes(prop)) return undefined;
  processing.push(prop);
  try {
    // Flatten the schema, merging allOf props and such, before we normalize type/format
    const normalized = flattenAzureSchema(prop, {}, processing);
    if (normalized === undefined) return undefined;

    // Infer type from format / properties / items if missing
    normalized.type ??= inferType(normalized);

    // Draft 4 -> Draft 6+ conversion (exclusiveMinimum/Maximum)
    for (const key of ["exclusiveMinimum", "exclusiveMaximum"] as const) {
      const baseKey = key === "exclusiveMinimum" ? "minimum" : "maximum";
      const val = normalized[key] as JSONSchema | boolean;
      const baseVal = normalized[baseKey];
      if (val === true && typeof baseVal === "number") {
        delete normalized[baseKey];
      } else if (val === false) {
        delete normalized[key];
      }
    }

    // Normalize formats to SI-supported ones
    if (normalized.format) {
      // TODO just support these or ignore later!
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

    return normalized;
  } finally {
    processing.pop();
  }
}

/// Infer the type for a schema from its other properties if "type" is missing
function inferType(prop: NormalizedAzureSchema) {
  if (prop.type) return prop.type;
  switch (prop.format) {
    case "int32":
    case "int64":
      return "integer";
    case "float":
    case "double":
    case "decimal":
      return "number";
  }
  if (prop.properties || prop.additionalProperties) return "object";
  if (prop.items) return "array";
}

/// Flattens JSONSchema recursively merging allOf, oneOf and anyOf into a single schema and
/// turning true ("any") into an empty schema, and throwing an exception for "false" (never) schemas.
function flattenAzureSchema(
  schemaProp: JSONSchema,
  flattened: NormalizedAzureSchema = {},
  normalizing: JSONSchema[],
): NormalizedAzureSchema | undefined {
  if (schemaProp === false) {
    throw new Error("Boolean schema 'false' (never) not supported");
  }
  // "any" becomes empty schema (which matches anything)
  if (schemaProp === true) {
    schemaProp = {};
  }

  // Pull off the stuff we're removing and children we're normalizing, so we can easily copy
  // the remaining values
  const {
    oneOf,
    anyOf,
    allOf,
    properties,
    patternProperties,
    items,
    additionalProperties,
    ...rest
  } = schemaProp;

  // Merge oneOf and anyOf by normalizing each alternative (so they have a type) and then
  // merging them into the flattened type (this means { a: string } | { b: string } becomes
  // { a?: string; b?: string }).
  // TODO handle required here?
  // TODO empty oneOf / anyOf (or oneOf: [ true ], which is effectively empty)
  if (oneOf) {
    for (const alternative of oneOf) {
      const child = normalizeAzureSchema(alternative, normalizing);
      if (!child) throw new Error("Cycle in oneOf or anyOf alternative");
      unionAzureSchema(child, flattened);
    }
  }
  if (anyOf) {
    for (const alternative of anyOf) {
      const child = normalizeAzureSchema(alternative, normalizing);
      if (!child) throw new Error("Cycle in oneOf or anyOf alternative");
      unionAzureSchema(child, flattened);
    }
  }

  // Merge allOf types into the flattened type (this means { a: string } & { b: string } becomes
  // { a: string; b: string }).
  //
  // We don't normalize here, because allOf children can be *partial* properties and we need
  // to merge them together so we have all the information we can have before normalizing. We
  // will normalize at the end after all properties have been flattened down.
  if (allOf) {
    for (const alternative of allOf) {
      flattenAzureSchema(alternative, flattened, normalizing);
    }
  }

  // Normalize child schemas (properties, items, etc.) so we can do a simple intersect after
  const prop: NormalizedAzureSchema = rest;
  if (properties) {
    prop.properties = {};
    if (Object.keys(properties).length > 0) {
      for (const [propName, childProp] of Object.entries(properties)) {
        const child = normalizeAzureSchema(childProp, normalizing);
        if (!child) continue; // If the prop is part of a cycle, don't include it
        // TODO find a better way! This fixes some Azure schemas with empty properties
        if (propName === "properties") {
          if (defaultAnyTypeTo(child, "object")) {
            child.properties ??= {};
          }
        } else {
          defaultAnyTypeTo(child, "string");
        }
        prop.properties[propName] = child;
      }
      // If all props were part of cycles, this prop is part of the cycle
      if (Object.keys(prop.properties).length == 0) return undefined;
    }
  }
  if (patternProperties) {
    prop.patternProperties = {};
    if (Object.keys(patternProperties).length > 0) {
      for (const [propName, childProp] of Object.entries(patternProperties)) {
        const child = normalizeAzureSchema(childProp, normalizing);
        if (!child) continue; // If the prop is part of a cycle, don't include it
        defaultAnyTypeTo(child, "string");
        prop.patternProperties[propName] = child;
      }
      // If all props were part of cycles, this prop is part of the cycle
      if (Object.keys(prop.patternProperties).length == 0) return undefined;
    }
  }
  if (items) {
    prop.items = normalizeAzureSchema(
      Array.isArray(items) ? { anyOf: items } : items,
      normalizing,
    );
    if (prop.items === undefined) return undefined;
    defaultAnyTypeTo(prop.items, "string");
  }
  if (additionalProperties) {
    prop.additionalProperties = normalizeAzureSchema(
      additionalProperties,
      normalizing,
    );
    if (prop.additionalProperties === undefined) return undefined;
    defaultAnyTypeTo(prop.additionalProperties, "string");
  }

  // Finally, intersect the props together
  intersectAzureSchema(prop, flattened);

  return flattened;
}

/// Intersects two flattened schemas, merging properties and such
function intersectAzureSchema(
  prop: NormalizedAzureSchema,
  intersected: NormalizedAzureSchema | undefined,
): NormalizedAzureSchema;
function intersectAzureSchema(
  prop: NormalizedAzureSchema | undefined,
  intersected: NormalizedAzureSchema,
): NormalizedAzureSchema;
function intersectAzureSchema(
  prop: NormalizedAzureSchema | undefined,
  intersected: NormalizedAzureSchema | undefined,
): NormalizedAzureSchema {
  if (!prop) return intersected!;
  if (!intersected) return prop;

  // *Now* merge in any remaining properties (outer properties override inner allOf properties).
  // Pay attention to make sure properties don't conflict.
  for (const key of Object.keys(prop) as (keyof NormalizedAzureSchema)[]) {
    if (prop[key] === undefined) continue;
    switch (key) {
      // These fields get merged
      case "properties":
      case "patternProperties": {
        intersected[key] ??= {};
        for (const [propName, childProp] of Object.entries(prop[key])) {
          intersected[key][propName] = intersectAzureSchema(
            childProp,
            intersected[key][propName],
          );
        }
        break;
      }
      case "additionalProperties":
      case "items": {
        // If items are part of a cycle, the array is also part of a cycle
        intersected[key] = intersectAzureSchema(prop[key], intersected[key]);
        break;
      }

      // These fields get merged as arrays without dups
      case "enum":
      case "required": {
        if (prop[key]) {
          intersected[key] = (intersected[key] ?? []).concat(
            prop[key].filter((v) => !intersected[key]?.includes(v)),
          );
        }
        break;
      }

      // We override these fields no matter what they were before
      case "title":
      case "description":
        intersected[key] = prop[key];
        continue;

      // Do not copy these, we handled them earlier
      case "allOf":
      case "anyOf":
      case "oneOf":
        break;

      default: {
        // Other fields must be identical if present in both
        if (
          intersected[key] !== undefined &&
          !util.isDeepStrictEqual(intersected[key], prop[key])
        ) {
          throw new Error(
            `Incompatible property ${key}: ${util.inspect(
              intersected[key],
            )} vs ${util.inspect(prop[key])}`,
          );
        }
        intersected[key] = prop[key];
        break;
      }
    }
  }

  return intersected;
}

const MORE_SPECIFIC_THAN_STRING = ["integer", "number", "boolean"];

function unionAzureSchema(
  prop: NormalizedAzureSchema,
  unioned: NormalizedAzureSchema | undefined,
): NormalizedAzureSchema;
function unionAzureSchema(
  prop: NormalizedAzureSchema | undefined,
  unioned: NormalizedAzureSchema,
): NormalizedAzureSchema;
function unionAzureSchema(
  prop: NormalizedAzureSchema | undefined,
  unioned: NormalizedAzureSchema | undefined,
) {
  if (!prop) return unioned;
  if (!unioned) return { ...prop };

  // If there are conflicting types [number | integer | boolean, string], we pick the specific one
  // TODO handle "format" / "minimum" / "maximum" conflicts too
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

  intersectAzureSchema(prop, unioned);
}

function defaultAnyTypeTo(prop: NormalizedAzureSchema, type: string) {
  if (prop.type !== undefined) return false;
  const isAnyType = Object.keys(prop).every(
    (k) => k === "type" || k === "description" || k === "readOnly",
  );
  if (isAnyType) prop.type = type;
  return isAnyType;
}

function stubResourceReferences(
  prop: NormalizedAzureSchema | undefined,
  /// The number of levels to keep the full property structure before stubbing
  resourceDepth: number,
) {
  if (!prop) return;

  // Increase resource depth if this is a resource, and possibly stub it
  if (
    prop.properties &&
    "id" in prop.properties &&
    "properties" in prop.properties
  ) {
    resourceDepth += 1;

    // Stub the resource if we're at max depth
    if (resourceDepth > MAX_RESOURCE_DEPTH) {
      prop.properties = { id: prop.properties.id };
      return;
    }
  }

  for (const childProp of Object.values(prop.properties ?? {})) {
    stubResourceReferences(childProp, resourceDepth);
  }
  for (const childProp of Object.values(prop.patternProperties ?? {})) {
    stubResourceReferences(childProp, resourceDepth);
  }
  // Don't count array/map items as depth; an array counts the same as a singleton for stubbing
  stubResourceReferences(prop.items, resourceDepth);
  stubResourceReferences(prop.additionalProperties, resourceDepth);
}

function buildDomainAndResourceValue(
  resourceType: string,
  getOperation: AzureOpenApiOperation,
  putOperation: AzureOpenApiOperation,
  handlers: { [key in CfHandlerKind]?: CfHandler },
  apiVersion: string,
): ExpandedPkgSpec | null {
  // Grab resourceValue properties from the GET response
  const { properties: resourceValueProperties } =
    extractPropertiesFromResponseBody(getOperation);
  if (Object.keys(resourceValueProperties).length === 0) {
    logger.debug(`No properties found in GET response for ${resourceType}`);
    return null;
  }
  for (const prop of Object.values(resourceValueProperties)) {
    stubResourceReferences(prop, 0);
  }
  if (!("id" in resourceValueProperties)) {
    throw new Error(
      `No id property in GET response: ${
        getOperation.operationId
      }\n\n${util.inspect(getOperation, { depth: 12 })}`,
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
  try {
    return makeModule(
      schema,
      description,
      onlyProperties,
      AZURE_PROVIDER_CONFIG,
      domainProperties as Record<string, AzureProperty>,
      resourceValueProperties as Record<string, AzureProperty>,
    );
  } catch (e) {
    logger.error(`Error creating spec for ${resourceType}`);
    throw e;
  }
}

function extractPropertiesFromRequestBody(
  operation: AzureOpenApiOperation | null,
) {
  const bodyParams = operation?.parameters?.filter(
    (p) => !("$ref" in p) && p.in === "body",
  );
  if (!operation || !bodyParams || bodyParams.length == 0) {
    return { properties: {}, required: [] };
  }
  assert(bodyParams.length <= 1, "Expected at most one body parameter");

  assert(!("$ref" in bodyParams[0]), "Body parameter is a $ref");
  const { schema } = bodyParams[0];
  if (!schema) return { properties: {}, required: [] };
  assert(!("$ref" in schema), "Body parameter is a $ref");
  const azureProp = normalizeAzureSchema(schema, []);
  assert(
    azureProp?.type === "object",
    `Response schema is not an object: ${operation.operationId}`,
  );
  return {
    properties: azureProp.properties ?? {},
    required: azureProp.required ?? [],
  };
}

function extractPropertiesFromResponseBody(
  operation: AzureOpenApiOperation | null,
) {
  const schema = operation?.responses?.["200"]?.schema;
  if (!schema) return { properties: {}, required: [] };
  const azureProp = normalizeAzureSchema(schema, []);
  assert(
    azureProp?.type === "object",
    `Response schema is not an object: ${operation.operationId}`,
  );

  const result = {
    properties: azureProp.properties ?? {},
    required: azureProp.required ?? [],
  };
  if (Object.keys(result.properties).length === 0) {
    logger.debug(`No properties found in GET response for ${operation}`);
    return result;
  }
  if (!("id" in result.properties)) {
    throw new Error(
      `No id property in GET response: ${
        operation.operationId
      }\n\n${util.inspect(operation, { depth: 12 })}\n\n${util.inspect(
        azureProp,
        { depth: 12 },
      )}`,
    );
  }
  return result;
}

/// Remove read-only properties from a record of name, property pairs
function removeReadOnlyProperties(
  properties: Record<string, NormalizedAzureSchema>,
  seen: Set<NormalizedAzureSchema>,
): Record<string, NormalizedAzureSchema> {
  return Object.fromEntries(
    Object.entries(properties)
      .map(([key, childProp]) => [key, removeReadOnlyProperty(childProp, seen)])
      .filter(([_, childProp]) => childProp !== undefined),
  );
}

/// Remove read-only properties from an AzureProperty
function removeReadOnlyProperty(
  property: NormalizedAzureSchema | undefined,
  seen: Set<NormalizedAzureSchema>,
): NormalizedAzureSchema | undefined {
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
      assert(!Array.isArray(property.items), "Tuple arrays not supported");
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
      throw new Error(
        `Expected basic JSON schema type, got ${util.inspect(property)}`,
      );
  }
  return property;
}

function parseEndpointPath(path: string) {
  // Form:         /subscriptions/{subscriptionId}/resourceGroups/{resourceGroupName}/providers/Microsoft.Network/applicationGateways/{applicationGatewayName}
  // Or list form: /subscriptions/{subscriptionId}/providers/Microsoft.Network/applicationGateways
  // TODO support non-{resourceGroupName} get/put/delete and {resourceGroupName} list paths
  const match = path.match(
    /\/subscriptions\/\{([^/}]+)\}(?:\/resource[Gg]roups\/\{([^/}]+)\})?\/providers\/([^/]+)\/([^/{]+)(?:\/\{([^/}]+)\})?$/,
  );
  if (!match) return undefined;
  const [
    _,
    subscriptionIdParam,
    resourceGroupParam,
    resourceProvider,
    resourceType,
    resourceNameParam,
  ] = match;

  // If the endpoint has {resourceName} it must also have {resourceGroup}. The list endpoint
  // (no {resourceName} must *not* have {resourceGroup} in it.
  if (!!resourceGroupParam !== !!resourceNameParam) return undefined;

  return {
    resourceProvider,
    resourceType,
    subscriptionIdParam,
    resourceGroupParam,
    resourceNameParam,
  };
}

// 1. pageables are always list ops
// 2. responses that are arrays are list ops
// 3. reponses that have a value object that is an array is a list op
function isListOperation(operation: AzureOpenApiOperation): boolean {
  if (operation["x-ms-pageable"] !== undefined) {
    return true;
  }

  const schema = operation.responses?.["200"]?.schema;
  if (!schema || typeof schema !== "object") return false;
  if ("type" in schema && schema.type === "array") {
    return true;
  }

  if (schema.properties?.value) {
    const valueSchema = schema.properties.value;
    if (
      typeof valueSchema === "object" &&
      "type" in valueSchema &&
      valueSchema.type === "array"
    ) {
      return true;
    }
  }

  return false;
}
