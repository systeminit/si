import { CfProperty, CfSchema } from "./cfDb.ts";
import { PkgSpec } from "../../../lib/si-pkg/bindings/PkgSpec.ts";
import { ulid } from "https://deno.land/x/ulid@v0.3.0/mod.ts";
import {
  createDefaultProp,
  createProp,
  DefaultPropType,
  ExpandedPropSpec,
  isExpandedPropSpec,
  OnlyProperties,
} from "./spec/props.ts";
import { PropSpec } from "../../../lib/si-pkg/bindings/PropSpec.ts";
import {
  SchemaVariantSpec,
} from "../../../lib/si-pkg/bindings/SchemaVariantSpec.ts";
import { SchemaSpec } from "../../../lib/si-pkg/bindings/SchemaSpec.ts";
import type {
  FuncSpecData,
} from "../../../lib/si-pkg/bindings/FuncSpecData.ts";
import { FuncSpec } from "../../../lib/si-pkg/bindings/FuncSpec.ts";
import { createSiFunc, getSiFuncId } from "./spec/siFuncs.ts";
import { attrFuncInputSpecFromProp } from "./spec/sockets.ts";
import {
  FuncArgumentSpec,
} from "../../../lib/si-pkg/bindings/FuncArgumentSpec.ts";

export function pkgSpecFromCf(src: CfSchema): PkgSpec {
  const [aws, category, name] = src.typeName.split("::");

  if (aws !== "AWS" || !category || !name) {
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
  };

  const domain: PropSpec = createDomainFromSrc(src, onlyProperties);
  const resourceValue: PropSpec = createResourceValueFromSrc(
    src,
    onlyProperties,
  );
  createInputsInDomainFromResource(domain, resourceValue);

  const variant: SchemaVariantSpec = {
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

  const schema: SchemaSpec = {
    name: src.typeName,
    data: {
      name: src.typeName,
      category: `AWS ${category}`,
      categoryName: null,
      uiHidden: false,
      defaultSchemaVariant: variantUniqueKey,
    },
    uniqueId: schemaUniqueKey, // TODO deal with this for existing schemas
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
    createdBy: "Cagador", // TODO Figure out a better name
    defaultChangeSet: null,
    workspacePk: null,
    workspaceName: null,
    schemas: [schema],
    funcs: createSiFuncs().concat(
      createResourcePayloadToValue(),
    ),
    changeSets: [], // always empty
  };
}

function versionFromDate(): string {
  return new Date().toISOString().replace(/[-:T.Z]/g, "").slice(0, 14);
}

function createDomainFromSrc(
  src: CfSchema,
  onlyProperties: OnlyProperties,
): PropSpec {
  return createRootFromProperties("domain", src.properties, onlyProperties);
}

function createResourceValueFromSrc(
  src: CfSchema,
  onlyProperties: OnlyProperties,
): PropSpec {
  return createRootFromProperties(
    "resource_value",
    pruneResourceValues(src.properties, onlyProperties),
    onlyProperties,
  );
}

function createRootFromProperties(
  root_name: DefaultPropType,
  properties: Record<string, CfProperty>,
  onlyProperties: OnlyProperties,
): PropSpec {
  const root: ExpandedPropSpec = createDefaultProp(root_name);
  Object.entries(properties).forEach(([name, cfData]) => {
    try {
      root.entries.push(
        createProp(name, cfData, onlyProperties, [...root.metadata.propPath]),
      );
    } catch (e) {
      console.log(`Err ${e}`);
    }
  });

  return root;
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

function createInputsInDomainFromResource(
  domain: PropSpec,
  resource: PropSpec,
) {
  if (resource.kind === "object" && domain.kind === "object") {
    resource.entries.forEach((resource: PropSpec) => {
      const domainProp = domain.entries.find((d: PropSpec) =>
        d.name === resource.name
      );
      if (domainProp?.data?.inputs) {
        domainProp.data.funcUniqueId = getSiFuncId("si:identity");
        domainProp.data.inputs.push(
          attrFuncInputSpecFromProp(resource as ExpandedPropSpec),
        );
      }
    });
  }
}

function createResourcePayloadToValue(): FuncSpec[] {
  const name = "si:resourcePayloadToValue";
  const data: FuncSpecData = {
    name,
    displayName: name,
    description: null,
    handler: "main",
    codeBase64:
      "YXN5bmMgZnVuY3Rpb24gbWFpbihhcmc6IElucHV0KTogUHJvbWlzZSA8IE91dHB1dCA+IHsKICAgIHJldHVybiBhcmcucGF5bG9hZCA/PyB7fTsKfQ",
    backendKind: "jsAttribute",
    responseType: "object",
    hidden: false,
    link: null,
  };

  const args: FuncArgumentSpec = {
    name: "payload",
    kind: "object",
    elementKind: null,
    uniqueId: ulid(),
    deleted: false,
  };

  const func: FuncSpec = {
    name,
    uniqueId: ulid(),
    data,
    deleted: false,
    isFromBuiltin: null,
    arguments: [args],
  };

  return [func];
}

function createSiFuncs(): FuncSpec[] {
  const ret: FuncSpec[] = [];
  const siFuncs = [
    "si:identity",
    "si:setArray",
    "si:setBoolean",
    "si:setInteger",
    "si:setJson",
    "si:setMap",
    "si:setObject",
    "si:setString",
    "si:unset",
    "si:validation",
  ];

  for (const func of siFuncs) {
    ret.push(createSiFunc(func));
  }

  return ret;
}
