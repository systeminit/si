import {
  CfProperty,
  CfSchema,
  getServiceByName,
  loadCfDatabase,
  normalizeAnyOfAndOneOfTypes,
  normalizePropertyType,
} from "../cfDb.ts";
import { PkgSpec } from "../bindings/PkgSpec.ts";
import { SchemaSpec } from "../bindings/SchemaSpec.ts";
import { SchemaVariantSpec } from "../bindings/SchemaVariantSpec.ts";
import { ulid } from "https://deno.land/x/ulid@v0.3.0/mod.ts";
import { PropSpec } from "../bindings/PropSpec.ts";
import { PropSpecData } from "../bindings/PropSpecData.ts";
import { FuncSpec } from "../bindings/FuncSpec.ts";
import type {
  FuncSpecData,
} from "../../../../lib/si-pkg/bindings/FuncSpecData.ts";

function pkgSpecFromCf(src: CfSchema): PkgSpec {
  const [aws, category, name] = src.typeName.split("::");

  if (aws !== "AWS" || !category || !name) {
    throw `Bad typeName: ${src.typeName}`;
  }

  const isBuiltin = true;

  const variantUniqueKey = ulid();
  const assetFuncUniqueKey = ulid();
  const schemaUniqueKey = ulid();

  const domain: PropSpec = createDefaultProp("domain");
  // console.log("Creating Props");
  // console.log(`Creating ${name}`);
  Object.entries(src.properties).forEach(([name, cfData]) => {
    try {
      domain.entries.push(createProp(name, cfData));
    } catch (e) {
      console.log(`Err ${e}`);
    }
  });

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
    sockets: [],
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

type CreatePropQueue = {
  addTo: null | ((data: PropSpec) => undefined);
  name: string;
  cfProp: CfProperty;
}[];

function createProp(name: string, cfProp: CfProperty) {
  const queue: CreatePropQueue = [
    {
      name,
      cfProp,
      addTo: null,
    },
  ];

  let rootProp = undefined;

  while (queue.length > 0) {
    const data = queue.shift();
    if (!data) break;

    const prop = createPropInner(data.name, data.cfProp, queue);

    if (!data.addTo) {
      rootProp = prop;
    } else {
      data.addTo(prop);
    }
  }

  if (!rootProp) {
    throw new Error(`createProp for ${name} did not generate a prop`);
  }

  return rootProp;
}

function createPropInner(
  name: string,
  cfProp: CfProperty,
  queue: CreatePropQueue,
): PropSpec {
  const propUniqueId = ulid();
  const data: PropSpecData = {
    name,
    validationFormat: null,
    defaultValue: null,
    funcUniqueId: null,
    inputs: null,
    widgetKind: null,
    widgetOptions: null,
    hidden: false,
    docLink: null,
    documentation: null,
  };

  const partialProp: unknown = {
    name,
    data,
    uniqueId: propUniqueId,
  };

  let normalizedCfData = normalizePropertyType(cfProp);
  normalizedCfData = normalizeAnyOfAndOneOfTypes(normalizedCfData);

  if (
    normalizedCfData.type === "integer" || normalizedCfData.type === "number"
  ) {
    const prop = partialProp as Extract<PropSpec, { kind: "number" }>;
    prop.kind = "number";
    prop.data!.widgetKind = "Text";
    return prop;
  } else if (normalizedCfData.type === "boolean") {
    const prop = partialProp as Extract<PropSpec, { kind: "boolean" }>;
    prop.kind = "boolean";
    prop.data!.widgetKind = "Checkbox";

    return prop;
  } else if (normalizedCfData.type === "string") {
    const prop = partialProp as Extract<PropSpec, { kind: "string" }>;
    prop.kind = "string";
    prop.data!.widgetKind = "Text";

    return prop;
  } else if (normalizedCfData.type === "array") {
    const prop = partialProp as Extract<PropSpec, { kind: "array" }>;
    prop.kind = "array";
    prop.data!.widgetKind = "Array";

    queue.push({
      addTo: (data: PropSpec) => {
        prop.typeProp = data;
      },
      name: `${name}Item`,
      cfProp: normalizedCfData.items,
    });

    return prop;
  } else if (normalizedCfData.type === "object") {
    if (normalizedCfData.patternProperties) {
      const prop = partialProp as Extract<PropSpec, { kind: "map" }>;
      prop.kind = "map";
      prop.data!.widgetKind = "Map";

      const patternProps = Object.entries(normalizedCfData.patternProperties);

      const [_, patternProp] = patternProps[0];

      if (patternProps.length !== 1 || !patternProp) {
        console.log(patternProps);
        throw new Error("too many pattern props you fool");
      }

      queue.push({
        addTo: (data: PropSpec) => {
          prop.typeProp = data;
        },
        name: `${name}Item`,
        cfProp: patternProp,
      });

      return prop;
    } else if (normalizedCfData.properties) {
      const prop = partialProp as Extract<PropSpec, { kind: "object" }>;
      prop.kind = "object";
      prop.data!.widgetKind = "Header";
      prop.entries = [];

      Object.entries(normalizedCfData.properties).forEach(
        ([objName, objProp]) => {
          queue.push({
            addTo: (data: PropSpec) => {
              prop.entries.push(data);
            },
            name: objName,
            cfProp: objProp,
          });
        },
      );
      return prop;
    }
  }

  console.log(cfProp);
  console.log(normalizedCfData);

  throw new Error("no matching kind");
}

type DefaultPropType = "domain" | "secrets" | "resource";

function createDefaultProp(
  type: DefaultPropType,
): Extract<PropSpec, { kind: "object" }> {
  const data: PropSpecData = {
    name: type,
    validationFormat: null,
    defaultValue: null,
    funcUniqueId: null,
    inputs: null,
    widgetKind: "Header",
    widgetOptions: null,
    hidden: null,
    docLink: null,
    documentation: null,
  };

  return {
    kind: "object",
    data,
    name: type,
    entries: [],
    uniqueId: ulid(),
  };
}
