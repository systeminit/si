import {
  ExpandedPkgSpec,
  ExpandedSchemaSpec,
  ExpandedSchemaVariantSpec,
} from "../../spec/pkgs.ts";
import { ulid } from "https://deno.land/x/ulid@v0.3.0/mod.ts";
import { CfProperty, ProviderConfig, SuperSchema } from "../types.ts";
import {
  createDefaultPropFromJsonSchema,
  OnlyProperties,
} from "../../spec/props.ts";
import { SiPkgKind } from "../../bindings/SiPkgKind.ts";
import { FuncSpec } from "../../bindings/FuncSpec.ts";
import { generateDefaultActionFuncs } from "./generateDefaultActionFuncs.ts";
import { generateDefaultLeafFuncs } from "./generateDefaultLeafFuncs.ts";
import { generateDefaultManagementFuncs } from "./generateDefaultManagementFuncs.ts";
import { generateDefaultQualificationFuncs } from "./generateDefaultQualificationFuncs.ts";
import {
  createActionFuncs,
  createCodeGenFuncs,
  createManagementFuncs,
  createQualificationFuncs,
} from "./funcFactories.ts";

function versionFromDate(): string {
  return new Date()
    .toISOString()
    .replace(/[-:T.Z]/g, "")
    .slice(0, 14);
}

/**
 * Normalizes property paths from JSON Pointer format to simple property names.
 * E.g., "/properties/foo" or "/foo" -> "foo"
 * @param props - Array of property paths (may be undefined)
 * @returns Array of normalized property names
 */
export function normalizeOnlyProperties(props: string[] | undefined): string[] {
  const newProps: string[] = [];
  for (const prop of props ?? []) {
    const newProp = prop.split("/").pop();
    if (newProp) {
      newProps.push(newProp);
    }
  }
  return newProps;
}

/**
 * SI's property path separator - vertical tab character.
 * Used to construct hierarchical property paths like "root\x0Bdomain\x0BpropertyName"
 */
export const PROP_PATH_SEPARATOR = "\x0B";

/**
 * Builds a property path string by joining parts with SI's separator.
 *
 * @param parts - Array of path segments (e.g., ["root", "domain", "propertyName"])
 * @returns Property path string with parts joined by \x0B
 *
 * @example
 * ```typescript
 * buildPropPath(["root", "domain", "region"])
 * // Returns: "root\x0Bdomain\x0Bregion"
 * ```
 */
export function buildPropPath(parts: string[]): string {
  return parts.join(PROP_PATH_SEPARATOR);
}

/**
 * Creates a helper function to find properties by name or path within a schema variant.
 * Supports both simple names and nested paths using dot notation.
 *
 * @param variant - The expanded schema variant containing domain entries
 * @param schemaName - Optional schema name for better error messages
 * @returns A function that takes a property name/path and returns the property or throws
 *
 * @example
 * ```typescript
 * const findProp = createPropFinder(variant, "AWS::EC2::AMI");
 * const region = findProp("region");                    // Simple property
 * const timeout = findProp("Config.Timeout");           // Nested property
 * const filterName = findProp("Filters.Name");          // Array element property
 * ```
 */
export function createPropFinder(
  variant: ExpandedSchemaVariantSpec,
  schemaName?: string,
) {
  return (path: string): any => {
    const parts = path.split(".");
    const schemaInfo = schemaName ? ` in ${schemaName}` : "";

    // Find the root property
    let currentProp: any = variant.domain.entries.find((p) => p.name === parts[0]);
    if (!currentProp) {
      throw new Error(
        `Property ${parts[0]} not found${schemaInfo} domain. Available properties: ${
          variant.domain.entries.map((p) => p.name).join(", ")
        }`,
      );
    }

    // Traverse nested properties if path has dots
    for (let i = 1; i < parts.length; i++) {
      const partName = parts[i];

      if (currentProp.kind === "object" && currentProp.entries) {
        const nextProp: any = currentProp.entries.find((e: any) => e.name === partName);
        if (!nextProp) {
          throw new Error(
            `Property ${partName} not found in ${parts.slice(0, i).join(".")}${schemaInfo}. ` +
            `Available properties: ${currentProp.entries.map((e: any) => e.name).join(", ")}`,
          );
        }
        currentProp = nextProp;
      } else if (currentProp.kind === "array" && currentProp.typeProp) {
        if (currentProp.typeProp.kind === "object" && currentProp.typeProp.entries) {
          const nextProp: any = currentProp.typeProp.entries.find((e: any) => e.name === partName);
          if (!nextProp) {
            throw new Error(
              `Property ${partName} not found in array element type for ${parts.slice(0, i).join(".")}${schemaInfo}`,
            );
          }
          currentProp = nextProp;
        } else {
          throw new Error(
            `Cannot traverse into ${partName} - array element is not an object in ${parts.slice(0, i).join(".")}${schemaInfo}`,
          );
        }
      } else {
        throw new Error(
          `Cannot traverse into ${partName} - ${parts.slice(0, i).join(".")} is not an object or array${schemaInfo}`,
        );
      }
    }

    return currentProp;
  };
}

export function makeModule(
  schema: SuperSchema,
  description: string,
  onlyProperties: OnlyProperties,
  providerConfig: ProviderConfig,
  domainProperties: Record<string, CfProperty>,
  resourceValueProperties: Record<string, CfProperty>,
  secretProperties: Record<string, CfProperty> = {},
) {
  const { createDocLink: docFn, getCategory: categoryFn } =
    providerConfig.functions;
  const color = providerConfig.metadata?.color || "#FF9900"; // Default to AWS orange

  // Create prop specs using the provided properties
  const domain = createDefaultPropFromJsonSchema(
    "domain",
    domainProperties,
    schema,
    onlyProperties,
    docFn,
    providerConfig,
  );

  const resourceValue = createDefaultPropFromJsonSchema(
    "resource_value",
    resourceValueProperties,
    schema,
    onlyProperties,
    docFn,
    providerConfig,
  );

  const secrets = createDefaultPropFromJsonSchema(
    "secrets",
    secretProperties,
    schema,
    onlyProperties,
    docFn,
    providerConfig,
  );

  const docLink = docFn(schema, undefined);
  const isBuiltin = true;

  const variantUniqueKey = ulid();
  const assetFuncUniqueKey = ulid();
  const schemaUniqueKey = ulid();
  const version = versionFromDate();

  const variant: ExpandedSchemaVariantSpec = {
    version,
    data: {
      version,
      link: docLink,
      color,
      displayName: null, // siPkg does not store this
      componentType: "component",
      funcUniqueId: assetFuncUniqueKey,
      description,
    },
    uniqueId: variantUniqueKey,
    deleted: false,
    isBuiltin,
    actionFuncs: [],
    authFuncs: [],
    leafFunctions: [],
    siPropFuncs: [],
    managementFuncs: [],
    domain,
    secrets,
    secretDefinition: null,
    resourceValue,
    rootPropFuncs: [],
    superSchema: schema,
  };

  const moduleSchema: ExpandedSchemaSpec = {
    name: schema.typeName,
    data: {
      name: schema.typeName,
      category: categoryFn(schema),
      categoryName: null,
      uiHidden: false,
      defaultSchemaVariant: variantUniqueKey,
    },
    uniqueId: schemaUniqueKey,
    deleted: false,
    isBuiltin,
    variants: [variant],
  };

  return {
    kind: "module" as SiPkgKind,
    name: schema.typeName,
    version,
    description: description,
    createdAt: new Date().toISOString(),
    createdBy: "Clover",
    defaultChangeSet: null,
    workspacePk: null,
    workspaceName: null,
    schemas: [moduleSchema] as [ExpandedSchemaSpec],
    funcs: [] as FuncSpec[],
    changeSets: [], // always empty
  };
}

/**
 * Generates all default funcs (actions, leaf, management, qualification) from a ProviderConfig.
 * This helper eliminates the need to call each generateDefault* function separately.
 * @param specs - Array of expanded package specs to process
 * @param config - Provider configuration containing func specs
 * @returns Updated array of specs with funcs added
 */
export function generateDefaultFuncsFromConfig(
  specs: ExpandedPkgSpec[],
  config: ProviderConfig,
): ExpandedPkgSpec[] {
  specs = generateDefaultActionFuncs(
    specs,
    () => createActionFuncs(config.funcSpecs.actions),
  );
  specs = generateDefaultLeafFuncs(
    specs,
    (domainId: string) =>
      createCodeGenFuncs(config.funcSpecs.codeGeneration, domainId),
  );
  specs = generateDefaultManagementFuncs(
    specs,
    () => createManagementFuncs(config.funcSpecs.management),
  );
  specs = generateDefaultQualificationFuncs(
    specs,
    (domainId: string) =>
      createQualificationFuncs(config.funcSpecs.qualification, domainId),
  );

  return specs;
}

// Re-export commonly used pipeline steps
export { applyAssetOverrides } from "./applyAssetOverrides.ts";
export { loadExtraAssets } from "./loadExtraAssets.ts";
