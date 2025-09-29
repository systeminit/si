import {
  ExpandedSchemaSpec,
  ExpandedSchemaVariantSpec,
} from "../../spec/pkgs.ts";
import { ulid } from "ulid";
import { CategoryFn, SuperSchema } from "../types.ts";
import { ExpandedPropSpecFor } from "../../spec/props.ts";
import { SiPkgKind } from "../../bindings/SiPkgKind.ts";

function versionFromDate(): string {
  return new Date()
    .toISOString()
    .replace(/[-:T.Z]/g, "")
    .slice(0, 14);
}

export function makeModule(
  schema: SuperSchema,
  docLink: string,
  description: string,
  domain: ExpandedPropSpecFor["object"],
  resourceValue: ExpandedPropSpecFor["object"],
  secrets: ExpandedPropSpecFor["object"],
  categoryFn: CategoryFn,
) {
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
      color: "#FF9900",
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
    funcs: [],
    changeSets: [], // always empty
  };
}

