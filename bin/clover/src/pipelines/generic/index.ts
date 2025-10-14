import {
  ExpandedPkgSpec,
  ExpandedSchemaSpec,
  ExpandedSchemaVariantSpec,
} from "../../spec/pkgs.ts";
import { ulid } from "https://deno.land/x/ulid@v0.3.0/mod.ts";
import { CfProperty, ProviderConfig, SuperSchema } from "../types.ts";
import { createDefaultPropFromCf, OnlyProperties } from "../../spec/props.ts";
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

export function makeModule(
  schema: SuperSchema,
  description: string,
  onlyProperties: OnlyProperties,
  providerConfig: ProviderConfig,
  domainProperties: Record<string, CfProperty>,
  resourceValueProperties: Record<string, CfProperty>,
) {
  const { createDocLink: docFn, getCategory: categoryFn } =
    providerConfig.functions;
  const color = providerConfig.metadata?.color || "#FF9900"; // Default to AWS orange

  // Create prop specs using the provided properties
  const domain = createDefaultPropFromCf(
    "domain",
    domainProperties,
    schema,
    onlyProperties,
    docFn,
    providerConfig,
  );

  const resourceValue = createDefaultPropFromCf(
    "resource_value",
    resourceValueProperties,
    schema,
    onlyProperties,
    docFn,
    providerConfig,
  );

  const secrets = createDefaultPropFromCf(
    "secrets",
    {},
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
  specs = generateDefaultActionFuncs(specs, () =>
    createActionFuncs(config.funcSpecs.actions),
  );
  specs = generateDefaultLeafFuncs(specs, (domainId: string) =>
    createCodeGenFuncs(config.funcSpecs.codeGeneration, domainId),
  );
  specs = generateDefaultManagementFuncs(specs, () =>
    createManagementFuncs(config.funcSpecs.management),
  );
  specs = generateDefaultQualificationFuncs(specs, (domainId: string) =>
    createQualificationFuncs(config.funcSpecs.qualification, domainId),
  );

  return specs;
}
