import { CfProperty, CfSchema } from "./cfDb.ts";
import { ulid } from "https://deno.land/x/ulid@v0.3.0/mod.ts";
import {
  createDefaultProp,
  createPropFromCf,
  DefaultPropType,
  OnlyProperties,
} from "./spec/props.ts";
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

  const domain = createDomainFromSrc(src, onlyProperties);

  const resourceValue = createResourceValueFromSrc(
    src,
    onlyProperties,
  );

  const variant: ExpandedSchemaVariantSpec = {
    version,
    data: {
      version,
      link: null,
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
    secrets: createDefaultProp("secrets"),
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

function createDomainFromSrc(
  src: CfSchema,
  onlyProperties: OnlyProperties,
) {
  return createRootFromProperties(
    "domain",
    pruneDomainValues(src.properties, onlyProperties),
    onlyProperties,
    src.typeName,
  );
}

function createResourceValueFromSrc(
  src: CfSchema,
  onlyProperties: OnlyProperties,
) {
  return createRootFromProperties(
    "resource_value",
    pruneResourceValues(src.properties, onlyProperties),
    onlyProperties,
    src.typeName,
  );
}

function createRootFromProperties(
  root_name: DefaultPropType,
  properties: Record<string, CfProperty>,
  onlyProperties: OnlyProperties,
  typeName: string,
) {
  const root = createDefaultProp(root_name);
  Object.entries(properties).forEach(([name, cfData]) => {
    const newProp = createPropFromCf(name, cfData, onlyProperties, typeName, [
      ...root.metadata.propPath,
    ]);

    if (!newProp) return;

    root.entries.push(newProp);
  });

  return root;
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
      .filter(([name]) => !readOnlySet.has(name)),
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
      .filter(([name]) => readOnlySet.has(name)),
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
