import assert from "node:assert";
import util from "node:util";
import logger from "../../logger.ts";
import { CfHandler, CfHandlerKind } from "../types.ts";
import {
  AzureDefinitions,
  AzureOpenApiDocument,
  AzureOpenApiOperation,
  AzureProperty,
  AzureSchema,
  AzureSchemaDefinition,
  NormalizedAzureSchema,
} from "./schema.ts";
import { JSONSchema } from "../draft_07.ts";
import { ExpandedPkgSpec } from "../../spec/pkgs.ts";
import { makeModule } from "../generic/index.ts";
import { AZURE_PROVIDER_CONFIG } from "./provider.ts";
import { OnlyProperties } from "../../spec/props.ts";
import { htmlToMarkdown } from "../../util.ts";

/// Maximum depth of full resource property expansion (below this, resources will are assumed
/// to be references and will just have "id")
const MAX_EXPANDED_RESOURCE_DEPTH = 3;

/// resource types (e.g. Microsoft.ServiceFabricMesh/applications) to ignore them until it's fixed
/// Make sure you also add a comment explaining why--all resource types should either be supported,
/// or ignored for a specific reason detected by heuristic!
const IGNORE_RESOURCE_TYPES = new Set<string>([
  // GET endpoint returns an array
  "Microsoft.PowerBI/privateLinkServicesForPowerBI",

  // Discriminator subtypes have properties with no type field (fluentdConfigUrl has description but no type)
  "Microsoft.ServiceFabricMesh/applications",

  // Discriminator subtypes have properties with no type field (jobStageDetails has no type and no allOf)
  "Microsoft.DataBox/jobs",

  // Read-only resource with circular schema references causing infinite recursion
  "Microsoft.DataProtection/operationStatus",

  // Crashes the JS interpreter
  "Microsoft.Media/videoAnalyzers/pipelineTopologies",
  "Microsoft.DataProtection/backupVaults/backupInstances",
  "Microsoft.DataProtection/backupVaults/backupPolicies",

  // Discriminator with no definitions
  "Microsoft.DataMigration/services/serviceTasks",
  "Microsoft.DataMigration/services/projects",

  // Unsupported format: url
  "Microsoft.MachineLearningServices/workspaces/connections",
  "Microsoft.MachineLearningServices/workspaces/endpoints",

  // Incompatible property x-ms-client-name: 'JobBaseProperties' vs 'LabelingJobProperties'
  "Microsoft.MachineLearningServices/workspaces/labelingJobs",

  // No id property in get response
  "Microsoft.Insights/components/exportconfiguration",
  "Microsoft.Insights/components/favorites",
  "Microsoft.Insights/components/ProactiveDetectionConfigs",
  "Microsoft.HDInsight/clusters/extensions",
]);

interface ResourceSpec {
  resourceType: string;
  get?: { path: string; operation: AzureOpenApiOperation };
  put?: { path: string; operation: AzureOpenApiOperation };
  handlers: { [key in CfHandlerKind]?: CfHandler };
}

export function parseAzureSpec(
  openApiDoc: AzureOpenApiDocument,
  resourceTypesFilter?: Set<string>,
): ExpandedPkgSpec[] {
  const specs: ExpandedPkgSpec[] = [];

  if (!openApiDoc.paths) {
    throw new Error("No paths found in Azure schema");
  }

  const defaultHandler = { permissions: [], timeoutInMinutes: 60 };
  const resourceOperations: Record<string, ResourceSpec> = {};

  // Collect all operations for each resource type
  for (const [path, methods] of Object.entries(openApiDoc.paths)) {
    if (!methods) continue;

    const pathInfo = parseEndpointPath(path);
    if (!pathInfo) continue;
    resourceOperations[pathInfo.resourceType] ??= {
      resourceType: pathInfo.resourceType,
      handlers: {},
    };
    const resource = resourceOperations[pathInfo.resourceType];

    // CRUD operation: /subscriptions/{subscriptionId}/resourceGroups/{resourceGroupName}/providers/Microsoft.Compute/virtualMachines/{vmName}[/extensions/{vmExtensionName}]...
    if (pathInfo.isCrudPath) {
      if (methods.get) {
        resource.get = { path, operation: methods.get };
        resource.handlers.read = defaultHandler;
      }
      if (methods.put) {
        resource.put = { path, operation: methods.put };
        resource.handlers.create = defaultHandler;
        resource.handlers.update = defaultHandler;
      }
      if (methods.delete) {
        resource.handlers.delete = defaultHandler;
      }

      // List operation: /subscriptions/{subscriptionId}/providers/Microsoft.Compute/virtualMachines
    } else if (methods.get && isListOperation(methods.get)) {
      resource.handlers.list = defaultHandler;
    }
  }

  // Build specs from collected operations
  for (const resource of Object.values(resourceOperations)) {
    // Presently we only support Microsoft. providers
    if (!resource.resourceType.toLowerCase().startsWith("microsoft.")) continue;
    // Ignore certain problematic resource types (temporarily until we fix them)
    if (IGNORE_RESOURCE_TYPES.has(resource.resourceType)) continue;
    // Skip resource types not in the filter (if filter is provided)
    if (
      resourceTypesFilter && !resourceTypesFilter.has(resource.resourceType)
    ) continue;
    // Skip subresources > 2 levels deep for now
    // TODO pick which of these to support
    const resourceDepth = resource.resourceType.split("/").length - 1;
    if (resourceDepth > 2) continue;
    // We only support readonly resources if they are top-level
    if (!resource.put && resourceDepth > 1) continue;

    try {
      const spec = buildDomainAndResourceValue(resource, openApiDoc);
      if (spec) {
        specs.push(spec);
      }
    } catch (e) {
      logger.error(`Failed to process resource type ${resource.resourceType}`);
      throw e;
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
export class AzureNormalizer {
  constructor(definitions: AzureDefinitions | undefined) {
    this.definitions = definitions;
  }
  normalizing: AzureSchemaDefinition[] = [];
  definitions: AzureDefinitions | undefined;
  discriminatorCollector: Record<string, Record<string, string>> = {};

  /// Normalize a general JSONSchema from Azure into simpler format
  /// without nesting.
  normalize(prop: AzureSchemaDefinition): NormalizedAzureSchema;
  normalize(
    prop: AzureSchemaDefinition | undefined,
  ): NormalizedAzureSchema | undefined;
  normalize(
    prop: AzureSchemaDefinition | undefined,
  ): NormalizedAzureSchema | undefined {
    // This is only meant to be called at the top level, non-recursively
    if (prop === undefined) return undefined;
    assert(this.normalizing.length === 0);
    const result = this.normalizeOrCycle(prop);
    assert(result, "Top-level schema somehow part of a cycle");
    if (!result) return undefined;
    return result;
  }

  /// Normalize a JSONSchema and merge its properties into an existing normalized schema. If
  /// the schemas are incompatible, throws an exception.
  intersect(
    prop: AzureSchemaDefinition | undefined,
    intersected: NormalizedAzureSchema,
  ) {
    return intersectAzureSchema(this.normalize(prop), intersected);
  }

  /// Normalize a JSONSchema and union its properties with an existing normalized schema.
  ///
  /// This will try to find a normalized shape that *can* accomodate both schemas but may not
  /// be optimal:
  ///
  ///   union({ foo: string; bar: string }, { foo: string; baz: string })
  ///     === { foo: string; bar?: string; baz?: string }
  ///
  ///   intersect({ foo: int }, { foo: float })
  ///     === { foo: float }
  ///
  /// @throws if we cannot find a common normalized shape (for example, string union object).
  union(
    prop: AzureSchemaDefinition | undefined,
    unioned: NormalizedAzureSchema,
  ) {
    return unionAzureSchema(this.normalize(prop), unioned);
  }

  private normalizeOrCycle(
    prop: AzureSchemaDefinition | undefined,
  ): NormalizedAzureSchema | undefined {
    if (prop === undefined) return undefined;
    if (this.normalizing.includes(prop)) return undefined;
    this.normalizing.push(prop);
    try {
      // Flatten the schema, merging allOf props and such, before we normalize type/format
      const normalized = this.flatten(prop, {});
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

      // At the root level, attach collected discriminators
      if (
        this.discriminatorCollector &&
        Object.keys(this.discriminatorCollector).length > 0
      ) {
        normalized.discriminators = this.discriminatorCollector;
      }

      return normalized;
    } finally {
      this.normalizing.pop();
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
  }

  /// Expands discriminators by finding matching subtypes and adding them as properties.
  /// Returns expanded properties and discriminator metadata, or undefined if no discriminators found.
  private expandDiscriminators(
    discriminator: AzureSchemaDefinition["discriminator"],
    properties: AzureSchemaDefinition["properties"],
  ): {
    expandedProperties: Record<string, JSONSchema>;
    discriminators: Record<string, Record<string, string>>;
  } | undefined {
    if (!discriminator) return undefined;
    if (!this.definitions) {
      throw new Error(
        `Schema has discriminator but no definitions: ${discriminator}`,
      );
    }
    if (!properties) return undefined;

    const discriminatorField = properties[discriminator];
    const enumValues = discriminatorField?.enum;

    // Find matching subtypes and their enum values
    // Maps definition name -> [schema, enum value]
    const subtypes: Map<string, [AzureSchemaDefinition, string]> = new Map();

    if (!enumValues) {
      // No enum - filter by checking if definition has the discriminator property
      for (const [defName, defSchema] of Object.entries(this.definitions)) {
        if (defHasProperty(defSchema, discriminator)) {
          // Without enum values, use definition name as the discriminator value
          subtypes.set(defName, [defSchema, defName]);
        }
      }
    } else {
      // Has enum - for each enum value, find matching definition
      for (const enumValue of enumValues) {
        const enumStr = String(enumValue);

        // Check for x-ms-discriminator-value match (direct or in allOf)
        let found = false;
        for (const [defName, defSchema] of Object.entries(this.definitions)) {
          // Boolean schemas don't have discriminator values or allOf
          // TODO: we should not need this check. It implies that we need to be doing
          // normalization for these sooner
          if (typeof defSchema === "boolean") continue;

          const hasDirectMatch =
            defSchema["x-ms-discriminator-value"] === enumStr;
          const hasAllOfMatch = defSchema.allOf?.some((
            s: AzureSchemaDefinition,
          ) => s["x-ms-discriminator-value"] === enumStr);

          if (hasDirectMatch || hasAllOfMatch) {
            subtypes.set(defName, [defSchema, enumStr]);
            found = true;
            break;
          }
        }

        // Check for exact definition name match
        // Also verify it has the discriminator property (direct or in allOf)
        if (!found && this.definitions[enumStr]) {
          const defSchema = this.definitions[enumStr];
          if (defHasProperty(defSchema, discriminator)) {
            subtypes.set(enumStr, [defSchema, enumStr]);
            found = true;
          }
        }

        // If still not found, that's okay - enum might have sentinel values like "Unknown"
      }
    }

    if (subtypes.size === 0) {
      return undefined;
    }

    // Create an object for the discriminator field and add subtypes as properties
    // within that object.
    const expandedProperties: Record<string, JSONSchema> = { ...properties };

    // Replace the discriminator field with an object containing the subtypes
    const discriminatorObject: JSONSchema = {
      type: "object",
      properties: {},
    };

    const discriminatorMap: Record<string, string> = {};

    for (const [defName, [subtype, enumValue]] of subtypes.entries()) {
      // TODO: we should not need this check. It implies that we need to be doing
      // normalization for these sooner
      if (typeof subtype === "boolean") continue;

      // TODO: We really should walk through these recursive cases as some of
      // these allOfs are likely desirable
      // Store the subtype's properties directly, but remove allOf to prevent circular references
      const subtypeSchema: JSONSchema = {
        type: "object",
        description: subtype.description,
        properties: subtype.properties,
        required: subtype.required,
      };

      discriminatorObject.properties![defName] = subtypeSchema;
      discriminatorMap[defName] = enumValue;
    }

    expandedProperties[discriminator] = discriminatorObject;

    const discriminators = {
      [discriminator]: discriminatorMap,
    };

    return {
      expandedProperties,
      discriminators,
    };

    /// Helper to check if a definition has a property (direct or in allOf)
    function defHasProperty(
      defSchema: AzureSchemaDefinition,
      propName: string,
    ): boolean {
      // Boolean schemas (true/false) don't have properties or allOf
      // TODO: we should not need this check. It implies that we need to be doing
      // normalization for these sooner
      if (typeof defSchema === "boolean") return false;

      const hasDirectProp = !!defSchema.properties?.[propName];
      const hasAllOfProp =
        defSchema.allOf?.some((s: AzureSchemaDefinition) =>
          !!s.properties?.[propName]
        ) ?? false;
      return hasDirectProp || hasAllOfProp;
    }
  }

  /// Flattens JSONSchema recursively merging allOf, oneOf and anyOf into a single schema and
  /// turning true ("any") into an empty schema, and throwing an exception for "false" (never) schemas.
  private flatten(
    schemaProp: AzureSchemaDefinition,
    flattened: NormalizedAzureSchema = {},
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
      discriminator,
      ...rest
    } = schemaProp;

    let expandedProperties = properties;

    // Merge oneOf and anyOf by normalizing each alternative (so they have a type) and then
    // merging them into the flattened type (this means { a: string } | { b: string } becomes
    // { a?: string; b?: string }).
    // TODO handle required here?
    // TODO empty oneOf / anyOf (or oneOf: [ true ], which is effectively empty)
    if (oneOf) {
      for (const alternative of oneOf) {
        const child = this.normalizeOrCycle(alternative);
        if (!child) throw new Error("Cycle in oneOf or anyOf alternative");
        unionAzureSchema(child, flattened);
      }
    }
    if (anyOf) {
      for (const alternative of anyOf) {
        const child = this.normalizeOrCycle(alternative);
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
        this.flatten(alternative, flattened);
      }
    }

    // Expand discriminators AFTER allOf processing to prevent allOf from overwriting expanded properties
    const expansion = this.expandDiscriminators(discriminator, properties);
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

    // Normalize child schemas (properties, items, etc.) so we can do a simple intersect after
    const prop: NormalizedAzureSchema = rest;
    if (expandedProperties) {
      prop.properties = {};
      if (Object.keys(expandedProperties).length > 0) {
        for (
          const [propName, childProp] of Object.entries(expandedProperties)
        ) {
          const child = this.normalizeOrCycle(childProp);
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
          const child = this.normalizeOrCycle(childProp);
          if (!child) continue; // If the prop is part of a cycle, don't include it
          defaultAnyTypeTo(child, "string");
          prop.patternProperties[propName] = child;
        }
        // If all props were part of cycles, this prop is part of the cycle
        if (Object.keys(prop.patternProperties).length == 0) return undefined;
      }
    }
    if (items) {
      prop.items = this.normalizeOrCycle(
        Array.isArray(items) ? { anyOf: items } : items,
      );
      if (prop.items === undefined) return undefined;
      defaultAnyTypeTo(prop.items, "string");
    }
    if (additionalProperties) {
      prop.additionalProperties = this.normalizeOrCycle(additionalProperties);
      if (prop.additionalProperties === undefined) return undefined;
      defaultAnyTypeTo(prop.additionalProperties, "string");
    }

    // Finally, intersect the props together
    intersectAzureSchema(prop, flattened);

    return flattened;
  }
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

      // Prefer false (writeable) over true (readonly)
      case "readOnly": {
        if (intersected[key] !== undefined) {
          intersected[key] = intersected[key] && prop[key];
        } else {
          intersected[key] = prop[key];
        }
        break;
      }

      default: {
        // Other fields must be identical if present in both
        if (
          intersected[key] !== undefined &&
          !util.isDeepStrictEqual(intersected[key], prop[key])
        ) {
          throw new Error(
            `Incompatible property ${key}: ${util.inspect(
              intersected[key],
            )
            } vs ${util.inspect(prop[key])}`,
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
    if (resourceDepth > MAX_EXPANDED_RESOURCE_DEPTH) {
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
  { resourceType, get, put, handlers }: ResourceSpec,
  openApiDoc: AzureOpenApiDocument,
): ExpandedPkgSpec | null {
  if (!get) {
    logger.debug(`No GET operation found for ${resourceType}`);
    return null;
  }

  // Create a shared normalizer
  const normalizer = new AzureNormalizer(openApiDoc.definitions);

  // Grab resourceValue properties from the GET response
  const resourceValue = responseSchema(get.operation, normalizer);
  if (Object.keys(resourceValue.properties).length === 0) {
    logger.debug(`No properties found in GET response for ${resourceType}`);
    return null;
  }

  // Grab domain properties from the PUT request
  const domain = put ? requestSchema(put.operation, normalizer) : undefined;
  if (domain) {
    // If it's a writeable resource, the result must have ID so we can update/delete
    if (!(resourceValue.properties && "id" in resourceValue.properties)) {
      throw new Error(
        `No id property in GET response: ${get.operation.operationId}\n\n${util.inspect(get.operation, { depth: 12 })
        }\n\n${util.inspect(
          get.operation.responses?.["200"]?.schema,
          { depth: 4 },
        )
        }`,
      );
    }

    // Only stub resource_value references for writable resources
    for (const prop of Object.values(resourceValue.properties)) {
      stubResourceReferences(prop, 0);
    }

    // Remove readonly properties from the domain
    domain.properties = removeReadOnlyProperties(
      domain.properties ?? {},
      new Set(),
    );
    if (Object.keys(domain.properties).length === 0) {
      logger.debug(`No properties found in PUT request for ${resourceType}`);
      return null;
    }
  }

  // Get discriminators from the shared collector
  const discriminators =
    Object.keys(normalizer.discriminatorCollector).length > 0
      ? normalizer.discriminatorCollector
      : undefined;

  const description = htmlToMarkdown(
    get.operation.description ||
      (get.operation.summary as string) ||
      `Azure ${resourceType} resource`,
  ) || `Azure ${resourceType} resource`;

  const primaryIdentifier = ["id"];
  const schema: AzureSchema = {
    typeName: resourceType,
    description,
    requiredProperties: new Set(domain?.required ?? []),
    handlers,
    apiVersion: openApiDoc.info.version,
    discriminators,
    resourceId: (put ?? get).path,
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
    (domain?.properties ?? {}) as Record<string, AzureProperty>,
    resourceValue.properties as Record<string, AzureProperty>,
  );
}

function requestSchema(
  operation: AzureOpenApiOperation | null,
  normalizer: AzureNormalizer,
) {
  const schema = { type: "object" } as NormalizedAzureSchema;

  // Pull parameters from path and schema from body
  // TODO query params? i.e. ApiVersion?
  for (const param of operation?.parameters ?? []) {
    switch (param.in) {
      case "path": {
        // Create the schema for the parameter itself
        const paramSchema = normalizer.normalize(param.schema ?? {});
        if ("type" in param) {
          normalizer.intersect({ type: param.type }, paramSchema);
        }
        assert(!param.style, "Parameter style not supported");
        if (param.description) {
          normalizer.intersect({ description: param.description }, paramSchema);
        }

        // Path parameters are never readonly - they're required inputs
        paramSchema.readOnly = false;

        // Add the parameter into the overall schema
        normalizer.intersect(
          { properties: { [param.name]: paramSchema } },
          schema,
        );
        // All parameters must be required since they're in the path
        normalizer.intersect({ required: [param.name] }, schema);
        break;
      }
      case "body":
        assert(param.schema, "Body parameter missing schema");
        normalizer.intersect(param.schema, schema);
        break;
      default:
        break;
    }
  }

  return schema;
}

function responseSchema(
  operation: AzureOpenApiOperation | null,
  normalizer: AzureNormalizer,
) {
  const schema = operation?.responses?.["200"]?.schema;
  if (!schema) return { properties: {}, required: [] };

  // Normalize the schema (discriminators will be collected during flattening)
  const azureProp = normalizer.normalize(schema);
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

export function parseEndpointPath(path: string) {
  if (!path.startsWith("/")) {
    throw new Error(`Endpoint path not absolute: ${path}`);
  }
  const segments = path.slice(1).split("/");

  // /subscriptions/{subscriptionId}
  if (segments.shift()?.toLowerCase() !== "subscriptions") return undefined;
  if (!segments.shift()) return undefined; // skip value

  // /resourceGroups/{resourceGroupName} (optional)
  const hasResourceGroupParam = segments[0]?.toLowerCase() === "resourcegroups";
  if (segments[0]?.toLowerCase() === "resourcegroups") {
    segments.shift();
    if (!segments.shift()) return undefined; // skip value
  }

  // /providers/Microsoft.Compute
  if (segments.shift()?.toLowerCase() !== "providers") return undefined;
  let resourceType = segments.shift();
  if (!resourceType) return undefined;
  // List operations (top level operations with no resource group param)
  if (segments.length === 1 && !hasResourceGroupParam) {
    return {
      isCrudPath: false,
      resourceType: `${resourceType}/${segments.shift()!}`,
    };
  }
  // CRUD operations require resource group param
  if (!hasResourceGroupParam) return undefined;

  // /virtualMachines/{vmName}[/extensions/{vmExtensionName}...]
  while (segments.length >= 2) {
    resourceType = `${resourceType}/${segments.shift()!}`;
    // Validate that the resource name segment is a parameter
    // TODO maybe constants or substring replacements are supported? Check
    if (!segments[0].startsWith("{")) return undefined;
    if (!segments[0].endsWith("}")) return undefined;
    segments.shift();
  }

  // /operation (the final segment)
  if (segments.shift()) return undefined;
  if (segments.length !== 0) {
    throw new Error(`Internal error: unexpected extra segments in ${path}`);
  }

  return {
    isCrudPath: true,
    resourceType,
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
