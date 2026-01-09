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
import {
  buildGcpTypeName,
  detectCreateOnlyProperties,
  titleCaseResourcePath,
} from "./utils.ts";

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

// Resource patterns to skip during spec generation
// Operations resources are internal GCP async tracking resources, not user-facing
const SKIP_RESOURCE_PATTERNS = [
  /[Oo]perations?$/, // operations, Operations, operation, Operation
];

function shouldSkipResource(resourceName: string): boolean {
  return SKIP_RESOURCE_PATTERNS.some((pattern) => pattern.test(resourceName));
}

interface ResourceSpec extends GcpResourceMethods {
  resourceName: string;
  resourcePath: string[];
  handlers: { [key in CfHandlerKind]?: CfHandler };
  availableScopes?: string[];
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

  // Deduplicate resources by their path after stripping scope prefix
  // Group by stripped path and track all available scopes
  const deduplicatedSpecs = deduplicateScopedResources(resourceSpecs);

  // Process each resource
  for (const resourceSpec of deduplicatedSpecs) {
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

/**
 * Deduplicate resources that exist at multiple scopes (projects, organizations, folders, billingAccounts).
 * Groups resources by their path after stripping the scope prefix, keeps one representative,
 * and records all available scopes. Note that all scopes share an API surface,
 * so it is safe to only keep one.
 */
function deduplicateScopedResources(
  resourceSpecs: ResourceSpec[],
): ResourceSpec[] {
  const byStrippedPath = new Map<
    string,
    { spec: ResourceSpec; scopes: string[] }
  >();

  for (const spec of resourceSpecs) {
    const scope = getScopePrefix(spec.resourcePath);
    const strippedPath = stripScopePrefix(spec.resourcePath);
    const key = strippedPath.join(".");

    if (byStrippedPath.has(key)) {
      const existing = byStrippedPath.get(key)!;
      if (scope && !existing.scopes.includes(scope)) {
        existing.scopes.push(scope);
      }
    } else {
      // First occurrence - use stripped path and record scope
      byStrippedPath.set(key, {
        spec: {
          ...spec,
          resourcePath: strippedPath,
          resourceName: spec.resourceName,
        },
        scopes: scope ? [scope] : [],
      });
    }
  }

  // Convert back to array, include availableScopes for any scoped resource
  return Array.from(byStrippedPath.values()).map(({ spec, scopes }) => ({
    ...spec,
    availableScopes: scopes.length > 0 ? scopes : undefined,
  }));
}

// Scope prefixes that indicate a multi-scope resource
// These will be deduplicated into a single asset with a scope selector
const SCOPE_PREFIXES = [
  "projects",
  "organizations",
  "folders",
  "billingAccounts",
];

// Path segments to strip from resource paths (they don't add meaningful context)
// "locations", "zones", and "regions" are stripped because regional vs global is handled by the location parameter
const STRIP_PATH_SEGMENTS = ["locations", "zones", "regions"];

// Patterns in method descriptions that indicate the resource only supports global location
const GLOBAL_ONLY_PATTERNS = [
  /only supported value for location is `global`/i,
  /Only global location is supported/i,
];

// Check if any method description indicates this is a global-only resource
function isGlobalOnlyResource(methods: {
  get?: GcpMethod;
  insert?: GcpMethod;
  update?: GcpMethod;
  patch?: GcpMethod;
  delete?: GcpMethod;
  list?: GcpMethod;
}): boolean {
  const allMethods = [
    methods.get,
    methods.insert,
    methods.update,
    methods.patch,
    methods.delete,
    methods.list,
  ].filter(Boolean) as GcpMethod[];

  for (const method of allMethods) {
    if (method.description) {
      for (const pattern of GLOBAL_ONLY_PATTERNS) {
        if (pattern.test(method.description)) {
          return true;
        }
      }
    }
  }
  return false;
}

function getScopePrefix(resourcePath: string[]): string | null {
  if (resourcePath.length === 0) return null;
  const firstSegment = resourcePath[0].toLowerCase();
  const match = SCOPE_PREFIXES.find((prefix) =>
    firstSegment === prefix.toLowerCase()
  );
  return match || null;
}

function stripScopePrefix(resourcePath: string[]): string[] {
  let path = resourcePath;

  // Strip scope prefix (projects, organizations, etc.)
  // BUT only if there's something left after stripping - if the resource IS
  // the scope type itself (like "folders" in Resource Manager), keep it
  const scope = getScopePrefix(path);
  if (scope && path.length > 1) {
    path = path.slice(1);
  }

  // Strip other non-meaningful segments (like "locations"), but keep the last
  // segment if stripping would result in an empty path (e.g., keep "locations"
  // for location discovery endpoints)
  const filtered = path.filter((segment) =>
    !STRIP_PATH_SEGMENTS.some((strip) =>
      segment.toLowerCase() === strip.toLowerCase()
    )
  );

  // If filtering removed everything, keep the last segment from the original path
  if (filtered.length === 0 && path.length > 0) {
    return [path[path.length - 1]];
  }

  return filtered;
}
/// Recursively collect all resources and their methods
function collectResources(
  resources: Record<string, GcpResource>,
  parentPath: string[],
  collected: ResourceSpec[],
) {
  for (const [resourceName, resource] of Object.entries(resources)) {
    if (shouldSkipResource(resourceName)) {
      logger.debug(`Skipping resource: ${resourceName}`);
      continue;
    }

    const resourcePath = [...parentPath, resourceName];

    if (resource.methods) {
      const methods = extractMethodsFromResource(resource.methods);

      // Create a spec if there's a get, list, or insert method
      // - insert: resources we can create
      // - get/list only: read-only resources useful for prop-to-prop subscriptions
      if (methods.get || methods.list || methods.insert) {
        const handlers: { [key in CfHandlerKind]?: CfHandler } = {};
        const defaultHandler = { permissions: [], timeoutInMinutes: 60 };

        // read handler works with get, or falls back to filtering list results
        if (methods.get || methods.list) handlers.read = defaultHandler;
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
  const {
    resourcePath,
    get,
    insert,
    update,
    patch,
    delete: deleteMethod,
    list,
    handlers,
    availableScopes,
  } = resourceSpec;

  // We need at least a get or list method to build a spec
  // List-only resources can still work - we filter list results for refresh
  if (!get && !list) {
    logger.debug(
      `No GET or LIST method found for resource ${resourcePath.join(".")}`,
    );
    return null;
  }

  // Get the response schema for resource_value properties
  // Prefer get.response, but fall back to extracting item schema from list.response
  let resourceSchema: NormalizedGcpSchema | undefined;

  if (get?.response) {
    resourceSchema = get.response;
  } else if (list?.response) {
    // List responses typically have an array property containing resources
    // Common patterns: "items", or the plural resource name (e.g., "connections")
    const listResponse = list.response;
    if (listResponse.properties) {
      // Find the array property that contains the resource items
      for (const [propName, propDef] of Object.entries(listResponse.properties)) {
        if (propDef.type === "array" && propDef.items) {
          resourceSchema = propDef.items;
          break;
        }
      }
    }
  }

  if (!resourceSchema) {
    logger.debug(
      `No response schema found for resource ${resourcePath.join(".")}`,
    );
    return null;
  }

  const resourceValueProperties = normalizeGcpSchemaProperties(resourceSchema);

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

  // Add path parameters (region, zone, etc.) to domain - these are needed to build API URLs
  // Exclude "project" as it comes from the credential
  // Only override if the property is readOnly or doesn't exist - some resources (like Subnetwork)
  // have writable region/zone fields that should be sent in the body too
  // Track which ones we add as path-only so we can mark them required later
  const pathParamsToAdd = ["region", "zone", "location"];
  const addedPathOnlyParams: string[] = [];
  for (const paramName of pathParamsToAdd) {
    if (insert?.parameters?.[paramName]) {
      const existingProp = domainProperties[paramName];
      const isReadOnly = existingProp && typeof existingProp === "object" &&
        existingProp.readOnly;

      if (!existingProp || isReadOnly) {
        const param = insert.parameters[paramName];
        domainProperties[paramName] = {
          type: param.type || "string",
          description: param.description,
        };
        // Track this as path-only if we're overriding a readOnly prop or adding new
        if (param.required) {
          addedPathOnlyParams.push(paramName);
        }
      }
    }
  }

  // Remove read-only properties from domain
  const writableDomainProperties = Object.fromEntries(
    Object.entries(domainProperties).filter(([_, prop]) =>
      typeof prop === "object" && prop !== null && !prop.readOnly
    ),
  );

  // Add 'parent' property if required by API and not "project" only
  // Project-only resources get parent auto-constructed from projectId + location
  // Multi-scope or non-project resources require explicit parent from user
  const insertParams = insert?.parameterOrder || [];
  const listParams = list?.parameterOrder || [];
  const apiNeedsParent =
    (insertParams.includes("parent") || listParams.includes("parent")) &&
    !writableDomainProperties["parent"];
  const isProjectOnly = availableScopes?.length === 1 &&
    availableScopes[0] === "projects";
  const needsExplicitParent = apiNeedsParent && !isProjectOnly;

  if (needsExplicitParent) {
    writableDomainProperties["parent"] = {
      type: "string",
      description:
        "The parent resource name (e.g., projects/my-project/locations/us-central1, organizations/123, folders/456)",
    };
  }

  // For project-only resources that are global-only, add a location prop with default "global"
  // This allows the parent to be auto-constructed as projects/{projectId}/locations/global
  // The prop will be marked hidden in addDefaultProps since the value is always "global"
  const isGlobalOnly = isGlobalOnlyResource({
    get,
    insert,
    update,
    patch,
    delete: deleteMethod,
    list,
  });
  if (isProjectOnly && isGlobalOnly && !writableDomainProperties["location"]) {
    writableDomainProperties["location"] = {
      type: "string",
      description:
        "The location for this resource (this resource only supports 'global')",
      default: "global",
    };
  }

  // Determine primary identifier from path parameters
  // Use get method if available, otherwise fall back to list or insert
  const primaryIdentifier = determinePrimaryIdentifier(get || list || insert);

  const typeName = buildGcpTypeName(doc.title || doc.name, resourcePath);

  // Normalize the asset description
  const description = normalizeDescription(
    resourceSchema.description ||
      get?.description ||
      list?.description ||
      `GCP ${doc.name} ${titleCaseResourcePath(resourcePath)} resource`,
  )!;

  // Detect required properties for creation
  // GCP uses annotations.required on each method that require it
  const requiredProperties = new Set<string>();

  // First check schema-level required array (rarely populated in GCP)
  for (
    const prop of resourceSchema.required || insert?.request?.required || []
  ) {
    requiredProperties.add(prop);
  }

  // Then check property-level annotations.required for the insert method
  if (insert?.request?.properties) {
    const insertMethodId = insert.id; // e.g., "compute.instances.insert"
    for (
      const [propName, propDef] of Object.entries(insert.request.properties)
    ) {
      if (propDef.annotations?.required?.includes(insertMethodId)) {
        requiredProperties.add(propName);
      }
    }
  }

  // Mark path-only parameters as required (e.g., region for Address, zone for Instance)
  for (const paramName of addedPathOnlyParams) {
    requiredProperties.add(paramName);
  }

  const schema: GcpSchema = {
    typeName,
    description,
    requiredProperties,
    handlers,
    service: doc.name,
    title: doc.title || doc.name,
    version: doc.version,
    resourcePath,
    baseUrl: doc.baseUrl,
    documentationLink: doc.documentationLink,
    availableScopes,
    isGlobalOnly,
    methods: {
      get,
      insert,
      update,
      patch,
      delete: deleteMethod,
      list,
    },
  };

  const onlyProperties: OnlyProperties = {
    createOnly: [],
    readOnly: [],
    writeOnly: [],
    primaryIdentifier,
  };

  // Classify properties
  const writeableProps = new Set(Object.keys(writableDomainProperties));
  const getProps = new Set(Object.keys(resourceValueProperties));

  // createOnly: Detect properties marked as immutable or creation-time-only
  onlyProperties.createOnly = detectCreateOnlyProperties(insert?.request);

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

/**
 * Normalizes description text by replacing newlines with spaces
 * and collapsing multiple spaces. GCP descriptions often have
 * embedded newlines that cause awkward formatting.
 */
function normalizeDescription(desc: string | undefined): string | undefined {
  if (!desc) return desc;
  return desc
    .replace(/\n/g, " ") // Replace newlines with spaces
    .replace(/\s+/g, " ") // Collapse multiple spaces
    .trim();
}

export function normalizeGcpProperty(
  prop: RawGcpProperty,
): NormalizedGcpSchema {
  const normalized: NormalizedGcpSchema = { ...prop };

  // Normalize the prop descriptions
  if (normalized.description) {
    normalized.description = normalizeDescription(normalized.description);
  }

  // Transform "any" type to "string" (most permissive type that's supported)
  if (normalized.type === "any") {
    normalized.type = "string";
  }

  // Note: GCP uses string type with int32/int64/uint32/uint64 format to avoid JS precision issues
  // We keep these as strings since the API returns them as strings, not actual integers
  // Converting to integer would cause type mismatches when SI tries to populate values

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
        "int32",
        "int64",
        "uint32",
        "uint64",
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
      default:
        // Handle non-standard delete method names (deleteConnection, remove, etc.)
        // Only map if we haven't already found a standard delete method
        if (
          !result.delete &&
          (lowerName.startsWith("delete") || lowerName.startsWith("remove"))
        ) {
          result.delete = method;
        }
        break;
    }
  }

  return result;
}

function determinePrimaryIdentifier(method: GcpMethod | undefined): string[] {
  // GCP resources typically have a 'name' or 'id' field
  // Look at the last parameter in parameterOrder
  if (method?.parameterOrder && method.parameterOrder.length > 0) {
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
