import { CfProperty, CfSchema } from "./cfDb.ts";
import { ulid } from "https://deno.land/x/ulid@v0.3.0/mod.ts";
import { createDefaultPropFromCf, createDocLink, OnlyProperties } from "./spec/props.ts";
import {
  ExpandedPkgSpec,
  ExpandedSchemaSpec,
  ExpandedSchemaVariantSpec,
} from "./spec/pkgs.ts";

export function pkgSpecFromCf(src: CfSchema): ExpandedPkgSpec {
  const [aws, category, name] = src.typeName.split("::");

  if (!["AWS", "Alexa"].includes(aws) || !category || !name) {
    throw `Bad typeName: ${src.typeName}`;
  }

  const isBuiltin = true;

  const variantUniqueKey = ulid();
  const assetFuncUniqueKey = ulid();
  const schemaUniqueKey = ulid();
  const version = versionFromDate();

  const onlyProperties: OnlyProperties = {
    createOnly: normalizeOnlyProperties(src.createOnlyProperties),
    readOnly: normalizeOnlyProperties(src.readOnlyProperties),
    writeOnly: normalizeOnlyProperties(src.writeOnlyProperties),
    primaryIdentifier: normalizeOnlyProperties(src.primaryIdentifier),
  };

  const domain = createDefaultPropFromCf(
    "domain",
    pruneDomainValues(src.properties, onlyProperties),
    src,
    onlyProperties,
  );

  const resourceValue = createDefaultPropFromCf(
    "resource_value",
    pruneResourceValues(src.properties, onlyProperties),
    src,
    onlyProperties,
  );

  const variant: ExpandedSchemaVariantSpec = {
    version,
    data: {
      version,
      link: createDocLink(src),
      color: "#b64017",
      displayName: name,
      componentType: "component",
      funcUniqueId: assetFuncUniqueKey,
      description: src.description,
    },
    uniqueId: variantUniqueKey,
    deleted: false,
    isBuiltin,
    actionFuncs: [],
    authFuncs: [],
    leafFunctions: [],
    sockets: [],
    siPropFuncs: [],
    managementFuncs: [],
    domain,
    secrets: createDefaultPropFromCf("secrets", {}, src, onlyProperties),
    secretDefinition: null,
    resourceValue,
    rootPropFuncs: [],
  };

  const schema: ExpandedSchemaSpec = {
    name: src.typeName,
    data: {
      name: src.typeName,
      category: `AWS ${category}`,
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
    name: src.typeName,
    version,
    description: src.description,
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
  return new Date().toISOString().replace(/[-:T.Z]/g, "").slice(0, 14);
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
      // TODO we shouldn't be ignoring things just because "type" isn't set
      .filter(([name, prop]) => prop.type && !readOnlySet.has(name)),
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
      .filter(([name, prop]) => prop.type && readOnlySet.has(name)),
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
