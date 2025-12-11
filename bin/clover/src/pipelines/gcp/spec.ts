import { ExpandedPkgSpec } from "../../spec/pkgs.ts";
import {
  GcpDiscoveryDocument,
  GcpMethod,
  GcpResource,
  GcpSchema,
  NormalizedGcpSchema,
} from "./schema.ts";
import { JSONSchema } from "../draft_07.ts";
import { makeModule } from "../generic/index.ts";
import { gcpProviderConfig } from "./provider.ts";
import { OnlyProperties } from "../../spec/props.ts";
import { CfHandler, CfHandlerKind, CfProperty } from "../types.ts";
import logger from "../../logger.ts";

export type GcpResourceMethods = {
  get?: GcpMethod;
  list?: GcpMethod;
  insert?: GcpMethod;
  update?: GcpMethod;
  patch?: GcpMethod;
  delete?: GcpMethod;
};

export type GcpMethodMap = Record<string, GcpMethod>;

// These shapes are the same, but this make it obvious what we are passing
// around and when
type GcpSchemaDefinition = NormalizedGcpSchema;
type RawGcpProperty = NormalizedGcpSchema;

interface ResourceSpec extends GcpResourceMethods {
  resourceName: string;
  resourcePath: string[];
  handlers: { [key in CfHandlerKind]?: CfHandler };
}

export function parseGcpDiscoveryDocument(
  doc: GcpDiscoveryDocument,
): ExpandedPkgSpec[] {
  const specs: ExpandedPkgSpec[] = [];

  if (!doc.resources && !doc.methods) {
    logger.debug(
      `No resources or methods found in ${doc.name} v${doc.version}`,
    );
    return specs;
  }

  // Collect all resources (including nested ones)
  const resourceSpecs: ResourceSpec[] = [];
  if (doc.resources) {
    collectResources(doc.resources, [], resourceSpecs);
  }

  // Process each resource
  for (const resourceSpec of resourceSpecs) {
    try {
      const spec = buildGcpResourceSpec(resourceSpec, doc);
      if (spec) {
        specs.push(spec);
      }
    } catch (e) {
      logger.error(
        `Failed to process resource ${resourceSpec.resourcePath.join(".")}: ${
          e instanceof Error ? e.message : String(e)
        }`,
      );
      // Continue processing other resources
    }
  }

  logger.debug(
    `Generated ${specs.length} specs from ${doc.name} v${doc.version}`,
  );

  return specs;
}

/// Recursively collect all resources and their methods
function collectResources(
  resources: Record<string, GcpResource>,
  parentPath: string[],
  collected: ResourceSpec[],
) {
  for (const [resourceName, resource] of Object.entries(resources)) {
    const resourcePath = [...parentPath, resourceName];

    if (resource.methods) {
      const methods = extractMethodsFromResource(resource.methods);

      // Only create a spec if there's at least a get or insert method
      if (methods.get || methods.insert) {
        const handlers: { [key in CfHandlerKind]?: CfHandler } = {};
        const defaultHandler = { permissions: [], timeoutInMinutes: 60 };

        if (methods.get) handlers.read = defaultHandler;
        if (methods.list) handlers.list = defaultHandler;
        if (methods.insert) {
          handlers.create = defaultHandler;
        }
        if (methods.update || methods.patch) {
          handlers.update = defaultHandler;
        }
        if (methods.delete) handlers.delete = defaultHandler;

        collected.push({
          resourceName,
          resourcePath,
          ...methods,
          handlers,
        });
      }
    }

    // Recursively process nested resources
    if (resource.resources) {
      collectResources(resource.resources, resourcePath, collected);
    }
  }
}

function buildGcpResourceSpec(
  resourceSpec: ResourceSpec,
  doc: GcpDiscoveryDocument,
): ExpandedPkgSpec | null {
  const { resourcePath, get, insert, update, patch, handlers } = resourceSpec;

  // We need at least a get method to build a spec
  if (!get) {
    logger.debug(
      `No GET method found for resource ${resourcePath.join(".")}`,
    );
    return null;
  }

  // Get the response schema for resource_value properties
  const getResponseSchema = get.response;

  if (!getResponseSchema) {
    logger.debug(
      `No response schema found for GET method of ${resourcePath.join(".")}`,
    );
    return null;
  }

  const resourceValueProperties = normalizeGcpSchemaProperties(
    getResponseSchema,
  );

  // Get the request schema for domain properties
  const domainProperties = insert?.request
    ? normalizeGcpSchemaProperties(insert.request)
    : {};

  // Merge update/patch properties into domain
  if (update?.request) {
    const updateProps = normalizeGcpSchemaProperties(update.request);
    Object.assign(domainProperties, updateProps);
  }
  if (patch?.request) {
    const patchProps = normalizeGcpSchemaProperties(patch.request);
    Object.assign(domainProperties, patchProps);
  }

  // Remove read-only properties from domain
  const writableDomainProperties = Object.fromEntries(
    Object.entries(domainProperties).filter(([_, prop]) =>
      typeof prop === "object" && prop !== null && !prop.readOnly
    ),
  );

  // Determine primary identifier from path parameters
  const primaryIdentifier = determinePrimaryIdentifier(get);

  // Build GCP schema
  const fullResourceName = resourcePath.join(".");
  const typeName = `GCP ${doc.title || doc.name} ${fullResourceName}`;
  const description = getResponseSchema.description ||
    get.description ||
    `GCP ${doc.name} ${fullResourceName} resource`;

  const requiredProperties = new Set(
    getResponseSchema.required || insert?.request?.required || [],
  );

  const schema: GcpSchema = {
    typeName,
    description,
    requiredProperties,
    handlers,
    service: doc.name,
    version: doc.version,
    resourcePath,
    baseUrl: doc.baseUrl,
    documentationLink: doc.documentationLink,
  };

  const onlyProperties: OnlyProperties = {
    createOnly: [],
    readOnly: [],
    writeOnly: [],
    primaryIdentifier,
  };

  // Classify properties
  const writeableProps = new Set(Object.keys(domainProperties));
  const getProps = new Set(Object.keys(resourceValueProperties));

  // readOnly: in GET but not writable (INSERT/UPDATE/PATCH)
  for (const prop of getProps) {
    if (!writeableProps.has(prop)) {
      onlyProperties.readOnly.push(prop);
    }
  }

  // writeOnly: writable (INSERT/UPDATE/PATCH) but not in GET
  for (const prop of writeableProps) {
    if (!getProps.has(prop)) {
      onlyProperties.writeOnly.push(prop);
    }
  }

  return makeModule(
    schema,
    description,
    onlyProperties,
    gcpProviderConfig,
    writableDomainProperties as Record<string, CfProperty>,
    resourceValueProperties as Record<string, CfProperty>,
  );
}

function normalizeGcpSchemaProperties(
  schema: GcpSchemaDefinition,
): Record<string, JSONSchema> {
  if (!schema.properties) {
    return {};
  }

  const normalized: Record<string, JSONSchema> = {};

  for (const [propName, propDef] of Object.entries(schema.properties)) {
    normalized[propName] = normalizeGcpProperty(propDef);
  }

  return normalized;
}

export function normalizeGcpProperty(
  prop: RawGcpProperty,
): NormalizedGcpSchema {
  const normalized: NormalizedGcpSchema = { ...prop };

  // Transform "any" type to "string" (most permissive type that's supported)
  if (normalized.type === "any") {
    normalized.type = "string";
  }

  // GCP uses string+int64 to avoid JS precision issues, normalize to integer
  if (
    normalized.type === "string" &&
    normalized.format &&
    (
      normalized.format === "int32" ||
      normalized.format === "int64" ||
      normalized.format === "uint32" ||
      normalized.format === "uint64"
    )
  ) {
    normalized.type = "integer";
  }

  // Normalize formats to SI-supported ones
  if (normalized.format) {
    const format = normalized.format;
    if (
      normalized.type === "integer" &&
      (
        format === "int32" ||
        format === "int64" ||
        format === "uint32" ||
        format === "uint64"
      )
    ) {
      delete normalized.format;
    } else if (
      normalized.type === "number" &&
      (
        format === "float" ||
        format === "double" ||
        format === "decimal"
      )
    ) {
      normalized.format = "double";
    } else if (
      normalized.type === "string" &&
      (
        format === "date-time" ||
        format === "date-time-rfc1123" ||
        format === "google-datetime"
      )
    ) {
      normalized.format = "date-time";
    } else if (
      normalized.type === "string" &&
      (format === "uri" || format === "url")
    ) {
      normalized.format = "uri";
    } else if (
      [
        "uuid",
        "email",
        "duration",
        "google-duration",
        "google-fieldmask",
        "date",
        "time",
        "byte",
        "binary",
        "password",
      ].includes(format)
    ) {
      delete normalized.format;
    }
  }

  // Parse minimum/maximum from strings to numbers
  if (
    normalized.minimum !== undefined && typeof normalized.minimum === "string"
  ) {
    normalized.minimum = parseFloat(normalized.minimum);
  }
  if (
    normalized.maximum !== undefined && typeof normalized.maximum === "string"
  ) {
    normalized.maximum = parseFloat(normalized.maximum);
  }

  // Recursively normalize nested structures
  if (prop.properties) {
    normalized.properties = {};
    for (const [key, value] of Object.entries(prop.properties)) {
      normalized.properties[key] = normalizeGcpProperty(value);
    }
  }

  if (prop.items) {
    normalized.items = normalizeGcpProperty(prop.items);
  }

  if (prop.additionalProperties) {
    normalized.additionalProperties = normalizeGcpProperty(
      prop.additionalProperties,
    );
  }

  return normalized;
}

export function extractMethodsFromResource(
  methods: GcpMethodMap,
): GcpResourceMethods {
  const result: GcpResourceMethods = {};

  for (const [methodName, method] of Object.entries(methods)) {
    const lowerName = methodName.toLowerCase();

    // Map method names to CRUD operations
    switch (lowerName) {
      case "get":
        result.get = method;
        break;
      case "list":
      case "aggregatedlist":
      case "listall":
        result.list = method;
        break;
      case "insert":
      case "create":
        result.insert = method;
        break;
      case "update":
        result.update = method;
        break;
      case "patch":
        result.patch = method;
        break;
      case "delete":
        result.delete = method;
        break;
    }
  }

  return result;
}

function determinePrimaryIdentifier(method: GcpMethod): string[] {
  // GCP resources typically have a 'name' or 'id' field
  // Look at the last parameter in parameterOrder
  if (method.parameterOrder && method.parameterOrder.length > 0) {
    const lastParam = method.parameterOrder[method.parameterOrder.length - 1];

    // Common GCP identifier patterns
    const identifierMap: Record<string, string> = {
      "name": "name",
      "resourceId": "id",
      "id": "id",
      "instanceId": "id",
      "diskId": "id",
      "networkId": "id",
    };

    return [identifierMap[lastParam] || "name"];
  }

  // Default to 'name' (most GCP resources use name)
  return ["name"];
}
