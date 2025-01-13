import { CfSchema, getServiceByName, loadDatabase } from "../cfDb.ts";
import { PkgSpec } from "../bindings/PkgSpec.ts";
import { SchemaSpec } from "../bindings/SchemaSpec.ts";
import { SchemaVariantSpec } from "../bindings/SchemaVariantSpec.ts";
import { ulid } from "https://deno.land/x/ulid@v0.3.0/mod.ts";
import { PropSpec } from "../bindings/PropSpec.ts";
import { PropSpecData } from "../bindings/PropSpecData.ts";

function pkgSpecFromCf(src: CfSchema): PkgSpec {
  const [aws, category, name] = src.typeName.split("::");

  if (aws !== "AWS" || !category || !name) {
    throw `Bad typeName: ${src.typeName}`;
  }

  const isBuiltin = true;

  const variantUniqueKey = ulid();
  const assetFuncUniqueKey = ulid();
  const schemaUniqueKey = ulid();

  const domain: PropSpec[] = [];
  Object.entries(src.properties).forEach(([name, cfData]) => {
    const propUniqueId = ulid();
    const data: PropSpecData = {
      name,
      validationFormat: null,
      defaultValue: undefined,
      funcUniqueId: null,
      inputs: null,
      widgetKind: undefined,
      widgetOptions: undefined,
      hidden: null,
      docLink: null,
      documentation: null,
    };

    let kind;

    switch (cfData.type) {
      case "integer":
        kind = "number";
        break;
      case "boolean":
      case "string":
      case "array":
        kind = cfData.type;
        break;
    }
    if (!kind) return;

    const spec: PropSpec = {
      // deno-lint-ignore no-explicit-any
      kind: kind as any,
      data,
      name,
      uniqueId: propUniqueId,
    };

    domain.push(spec);
  });

  const variant: SchemaVariantSpec = {
    version: "",
    data: {
      version: "",
      link: null,
      color: null,
      displayName: null,
      componentType: undefined,
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
    secrets: {},
    secretDefinition: undefined,
    resourceValue: {},
    rootPropFuncs: [],
  };
  // TODO do an autopsy of a spec from module index to fill these prop specs

  const schema: SchemaSpec = {
    name: src.typeName,
    data: {
      name: src.typeName,
      category: `AWS ${category}`,
      categoryName: `AWS ${category}`,
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
    version: "",
    description: src.description,
    createdAt: new Date().toLocaleString(),
    createdBy: "Cagador", // TODO Figure out a better name
    defaultChangeSet: null,
    workspacePk: null,
    workspaceName: null,
    schemas: [schema],
    funcs: [],
    changeSets: [], // always empty
  };
}

export async function setupDatabase() {
  const db = await loadDatabase();

  for (const cfSchema of Object.values(db)) {
    const pkg = pkgSpecFromCf(cfSchema);
    console.log(pkg);
    console.log(pkg.schemas[0].variants[0].domain);
  }
}
