import { CfProperty, CfSchema } from "../../../cfDb.ts";
import { ulid } from "https://deno.land/x/ulid@v0.3.0/mod.ts";
import {
  createDefaultPropFromCf,
  createDocLink,
  OnlyProperties,
} from "../../../spec/props.ts";
import {
  ExpandedPkgSpec,
  ExpandedSchemaSpec,
  ExpandedSchemaVariantSpec,
} from "../../../spec/pkgs.ts";

export function pkgSpecFromCf(cfSchema: CfSchema): ExpandedPkgSpec {
  const [metaCategory, category, name] = cfSchema.typeName.split("::");

  if (!["AWS", "Alexa"].includes(metaCategory) || !category || !name) {
    throw `Bad typeName: ${cfSchema.typeName}`;
  }

  const isBuiltin = true;

  const variantUniqueKey = ulid();
  const assetFuncUniqueKey = ulid();
  const schemaUniqueKey = ulid();
  const version = versionFromDate();

  const onlyProperties: OnlyProperties = {
    createOnly: normalizeOnlyProperties(cfSchema.createOnlyProperties),
    readOnly: normalizeOnlyProperties(cfSchema.readOnlyProperties),
    writeOnly: normalizeOnlyProperties(cfSchema.writeOnlyProperties),
    primaryIdentifier: normalizeOnlyProperties(cfSchema.primaryIdentifier),
  };

  const domain = createDefaultPropFromCf(
    "domain",
    pruneDomainValues(cfSchema.properties, onlyProperties),
    cfSchema,
    onlyProperties,
  );

  const resourceValue = createDefaultPropFromCf(
    "resource_value",
    pruneResourceValues(cfSchema.properties, onlyProperties),
    cfSchema,
    onlyProperties,
  );

  const variant: ExpandedSchemaVariantSpec = {
    version,
    data: {
      version,
      link: createDocLink(cfSchema, undefined),
      color: "#FF9900",
      displayName: null, // siPkg does not store this
      componentType: "component",
      funcUniqueId: assetFuncUniqueKey,
      description: cfSchema.description,
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
    secrets: createDefaultPropFromCf("secrets", {}, cfSchema, onlyProperties),
    secretDefinition: null,
    resourceValue,
    rootPropFuncs: [],
    cfSchema,
  };

  const schema: ExpandedSchemaSpec = {
    name: cfSchema.typeName,
    data: {
      name: cfSchema.typeName,
      category: `${metaCategory}::${category}`,
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
    kind: "module",
    name: cfSchema.typeName,
    version,
    description: cfSchema.description,
    createdAt: new Date().toISOString(),
    createdBy: "Clover",
    defaultChangeSet: null,
    workspacePk: null,
    workspaceName: null,
    schemas: [schema],
    funcs: [],
    changeSets: [], // always empty
  };
}

function versionFromDate(): string {
  return new Date()
    .toISOString()
    .replace(/[-:T.Z]/g, "")
    .slice(0, 14);
}

// Remove all read only props from this list, since readonly props go on the
// resource value tree
function pruneDomainValues(
  properties: Record<string, CfProperty>,
  onlyProperties: OnlyProperties,
): Record<string, CfProperty> {
  if (!properties || !onlyProperties.readOnly) {
    return {};
  }

  const readOnlySet = new Set(onlyProperties.readOnly);
  return Object.fromEntries(
    Object.entries(properties)
      // Include properties that either have a type OR have oneOf/anyOf
      .filter(
        ([name, prop]) =>
          (prop.type || prop.oneOf || prop.anyOf) && !readOnlySet.has(name),
      ),
  );
}

function pruneResourceValues(
  properties: Record<string, CfProperty>,
  onlyProperties: OnlyProperties,
): Record<string, CfProperty> {
  if (!properties || !onlyProperties?.readOnly) {
    return {};
  }

  const readOnlySet = new Set(onlyProperties.readOnly);
  return Object.fromEntries(
    Object.entries(properties)
      // Include properties that either have a type OR have oneOf/anyOf
      .filter(
        ([name, prop]) =>
          (prop.type || prop.oneOf || prop.anyOf) && readOnlySet.has(name),
      ),
  );
}

function normalizeOnlyProperties(props: string[] | undefined): string[] {
  const newProps: string[] = [];
  for (const prop of props ?? []) {
    const newProp = prop.split("/").pop();
    if (newProp) {
      newProps.push(newProp);
    }
  }
  return newProps;
}
