import {
  CfProperty,
  CfSchema,
  loadDatabase,
  normalizeAnyOfAndOneOfTypes,
  normalizePropertyType,
} from "../cfDb.ts";
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

  const domain: PropSpec = createDefaultProp("domain");
  Object.entries(src.properties).forEach(([name, cfData]) => {
    domain.entries.push(createProp(name, cfData));
  });

  const variant: SchemaVariantSpec = {
    version: "",
    data: {
      version: "",
      link: null,
      color: null,
      displayName: null,
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
    secretDefinition: undefined,
    resourceValue: createDefaultProp("resource"),
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
    createdAt: new Date().toISOString(),
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
    console.log(`Building: ${cfSchema.typeName}`);
    const pkg = pkgSpecFromCf(cfSchema);
    // console.log(JSON.stringify(pkg, null, 2));
    console.log(`Built: ${cfSchema.typeName}`);
  }
}

function createProp(name: string, cfData: CfProperty): PropSpec {
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
  let typeProp;
  const entries: PropSpec[] = [];
  let normalizedCfData = normalizePropertyType(cfData);
  normalizedCfData = normalizeAnyOfAndOneOfTypes(normalizedCfData);

  switch (normalizedCfData.type) {
    case "integer":
    case "number":
      kind = "number";
      break;
    case "boolean":
    case "string":
      kind = normalizedCfData.type;
      break;
    case "array":
      kind = normalizedCfData.type;
      typeProp = createProp(`${name}Item`, normalizedCfData.items);
      break;
    case "object":
      if (normalizedCfData.patternProperties) {
        kind = "map";
        const patternProps = Object.entries(normalizedCfData.patternProperties);

        const [_, patternProp] = patternProps[0];

        if (patternProps.length !== 1 || !patternProp) {
          console.log(patternProps);
          throw new Error("too many pattern props you fool");
        }

        typeProp = createProp(`${name}Item`, patternProp);
      } else if (normalizedCfData.properties) {
        kind = normalizedCfData.type;
        Object.entries(normalizedCfData.properties).forEach(
          ([objName, objProp]) => {
            entries.push(createProp(objName, objProp));
          },
        );
      }
      break;
    default:
      console.log(normalizedCfData);
      console.log("no matching kind");
  }

  return {
    // deno-lint-ignore no-explicit-any
    kind: kind as any,
    data,
    entries,
    name,
    typeProp,
    uniqueId: propUniqueId,
  };
}

type DefaultPropType = "domain" | "secrets" | "resource";

function createDefaultProp(
  type: DefaultPropType,
): Extract<PropSpec, { kind: "object" }> {
  const data: PropSpecData = {
    name: type,
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

  return {
    // deno-lint-ignore no-explicit-any
    kind: "object",
    data,
    name: type,
    entries: [],
    uniqueId: ulid(),
  };
}
