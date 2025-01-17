import { CfSchema, getServiceByName, loadCfDatabase } from "../cfDb.ts";
import {
  createDefaultProp,
  createProp,
  isExpandedPropSpec,
  OnlyProperties,
} from "../spec/props.ts";
import { PkgSpec } from "../bindings/PkgSpec.ts";
import { SchemaSpec } from "../bindings/SchemaSpec.ts";
import { SchemaVariantSpec } from "../bindings/SchemaVariantSpec.ts";
import { ulid } from "https://deno.land/x/ulid@v0.3.0/mod.ts";
import { PropSpec } from "../bindings/PropSpec.ts";
import { FuncSpec } from "../bindings/FuncSpec.ts";
import type { FuncSpecData } from "../bindings/FuncSpecData.ts";
import { SocketSpec } from "../bindings/SocketSpec.ts";
import { createSocketFromProp } from "../spec/sockets.ts";

function pkgSpecFromCf(src: CfSchema): PkgSpec {
  const [aws, category, name] = src.typeName.split("::");

  if (aws !== "AWS" || !category || !name) {
    throw `Bad typeName: ${src.typeName}`;
  }

  const isBuiltin = true;

  const variantUniqueKey = ulid();
  const assetFuncUniqueKey = ulid();
  const schemaUniqueKey = ulid();

  const domain: PropSpec = createDomainFromSrc(src);
  const sockets = createSocketsFromDomain(domain);

  const variant: SchemaVariantSpec = {
    version: "",
    data: {
      version: "",
      link: null,
      color: "#b64017",
      displayName: name,
      componentType: "component",
      funcUniqueId: assetFuncUniqueKey,
      description: null,
    },
    uniqueId: variantUniqueKey,
    deleted: false,
    isBuiltin,
    actionFuncs: [],
    authFuncs: [],
    leafFunctions: [],
    sockets,
    siPropFuncs: [],
    managementFuncs: [],
    domain,
    secrets: createDefaultProp("secrets"),
    secretDefinition: null,
    resourceValue: createDefaultProp("resource"),
    rootPropFuncs: [],
  };
  // TODO do an autopsy of a spec from module index to fill these prop specs

  const schema: SchemaSpec = {
    name: src.typeName,
    data: {
      name: src.typeName,
      category: `AWS ${category}`,
      categoryName: name,
      uiHidden: false,
      defaultSchemaVariant: variantUniqueKey,
    },
    uniqueId: schemaUniqueKey, // TODO deal with this for existing schemas
    deleted: false,
    isBuiltin,
    variants: [variant],
  };

  const assetFuncName = `${src.typeName}_AssetFunc`;

  const assetFuncData: FuncSpecData = {
    name: assetFuncName,
    displayName: null,
    description: null,
    handler: "main",
    codeBase64: btoa(
      "function main() {\n" +
        "  const asset = new AssetBuilder();\n" +
        "  return asset.build();\n" +
        "}",
    ),
    backendKind: "jsSchemaVariantDefinition",
    responseType: "schemaVariantDefinition",
    hidden: false,
    link: null,
  };

  const assetFunc: FuncSpec = {
    name: assetFuncName,
    uniqueId: assetFuncUniqueKey,
    data: assetFuncData,
    deleted: false,
    isFromBuiltin: true,
    arguments: [],
  };

  return {
    kind: "module",
    name: src.typeName,
    version: "",
    description: src.description,
    createdAt: new Date().toISOString(),
    createdBy: "Cagador", // TODO Figure out a better name
    defaultChangeSet: null,
    workspacePk: null,
    workspaceName: null,
    schemas: [schema],
    funcs: [assetFunc],
    changeSets: [], // always empty
  };
}

export function generateSiSpecForService(serviceName: string) {
  const cf = getServiceByName(serviceName);
  return pkgSpecFromCf(cf);
}

export async function generateSiSpecDatabase() {
  const db = await loadCfDatabase();

  let imported = 0;
  const cfSchemas = Object.values(db);
  for (const cfSchema of cfSchemas) {
    // console.log(`Building: ${cfSchema.typeName}`);
    try {
      const pkg = pkgSpecFromCf(cfSchema);
      console.log(JSON.stringify(pkg, null, 2));
    } catch (e) {
      console.log(`Error Building: ${cfSchema.typeName}: ${e}`);
      continue;
    }
    imported += 1;
    // console.log(`Built: ${cfSchema.typeName}`);
  }
  // console.log(`built ${imported} out of ${cfSchemas.length}`);
}

function createDomainFromSrc(
  src: CfSchema,
): PropSpec {
  const onlyProperties: OnlyProperties = {
    "createOnly": normalizeOnlyProperties(src.createOnlyProperties),
    "readOnly": normalizeOnlyProperties(src.readOnlyProperties),
    "writeOnly": normalizeOnlyProperties(src.writeOnlyProperties),
  };

  const domain: PropSpec = createDefaultProp("domain");
  Object.entries(src.properties).forEach(([name, cfData]) => {
    try {
      domain.entries.push(createProp(name, cfData, onlyProperties));
    } catch (e) {
      console.log(`Err ${e}`);
    }
  });

  return domain;
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

function createSocketsFromDomain(domain: PropSpec): SocketSpec[] {
  const sockets: SocketSpec[] = [];
  if (domain.kind == "object") {
    for (const prop of domain.entries) {
      if (
        !["array", "object"].includes(prop.kind) && isExpandedPropSpec(prop)
      ) {
        const socket = createSocketFromProp(prop);
        if (socket) {
          sockets.push(socket);
        }
      }
    }
  }
  return sockets;
}
