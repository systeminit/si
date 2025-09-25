import {
  ExpandedPkgSpec,
  ExpandedSchemaSpec,
  ExpandedSchemaVariantSpec,
} from "../../spec/pkgs.ts";
import { ulid } from "https://deno.land/x/ulid@v0.3.0/mod.ts";
import {
  createDefaultPropFromCf,
  createPropFromCf,
  DefaultPropType,
  ExpandedPropSpec,
  ExpandedPropSpecFor,
  OnlyProperties,
} from "../../spec/props.ts";
import rawSchema from "../../provider-schemas/hetzner.json" with {
  type: "json",
};
import { getExistingSpecs } from "../../specUpdates.ts";
import { CfProperty, CfSchema } from "../../cfDb.ts";

export type HetznerSchema = {
  typeName: string;
  sourceUrl?: string;
  documentationUrl?: string;
  properties: Record<string, CfProperty>;
  requiredProperties: Set<string>;
};

export type HDB = Record<string, HetznerSchema>;

type CreatePropArgs = {
  // The path to this prop, e.g. ["root", "domain"]
  propPath: string[];
  // The definition for this prop in the schema
  cfProp: CfProperty & { defName?: string };
  // The parent prop's definition
  parentProp: ExpandedPropSpecFor["object" | "array" | "map"] | undefined;
  // A handler to add the prop to its parent after it has been created
  addTo?: (data: ExpandedPropSpec) => undefined;
};

export type HQueue = {
  cfSchema: HetznerSchema;
  onlyProperties: OnlyProperties;
  queue: CreatePropArgs[];
};

export async function generateHetznerSpecs(options: {
  forceUpdateExistingPackages?: boolean;
  moduleIndexUrl: string;
  docLinkCache: string;
  inferred: string;
  services?: string[];
}): Promise<ExpandedPkgSpec[]> {
  let specs: ExpandedPkgSpec[] = [];

  const existing_specs = await getExistingSpecs(options);

  // skipping inferred combo boxes

  specs = pkgSpecFromHetnzer(rawSchema);

  return specs;
}

function pkgSpecFromHetnzer(allSchemas: any) {
  const schemas: HDB = {};
  const specs: ExpandedPkgSpec[] = [];
  Object.entries(allSchemas.paths).forEach(
    ([endpoint, _openApiDescription]) => {
      const noun = endpoint.split("/")[1];
      // skipping actions for now
      if (endpoint.includes("actions")) return;
      const openApiDescription = _openApiDescription as any;
      if (!openApiDescription.get) throw new Error(`WHY NO GET? ${noun}`);

      // skipping list endpoints for now
      if (openApiDescription.get.operationId.startsWith("list_")) return;

      if (schemas[noun]) {
        console.error(`ALREADY GOT ${noun}`);
        return;
      }

      const content =
        openApiDescription.get.responses["200"].content["application/json"];
      // the key of `properties` seems to be the singular form of the noun, but its alone, so just popping
      const objShape = Object.values(content.schema.properties).pop() as
        | any
        | undefined;
      if (!objShape) {
        console.error("SHAPE EXPECTED", content);
        return;
      }
      const properties = objShape.properties;
      const requiredProperties = new Set(objShape.required as string[]);
      const schema: HetznerSchema = {
        typeName: noun,
        properties,
        requiredProperties,
      };
      schemas[noun] = schema;
    },
  );

  Object.values(schemas).forEach((schema: HetznerSchema) => {
    const isBuiltin = true;

    const variantUniqueKey = ulid();
    const assetFuncUniqueKey = ulid();
    const schemaUniqueKey = ulid();
    const version = versionFromDate();

    const onlyProperties: OnlyProperties = {
      createOnly: [],
      readOnly: [],
      writeOnly: [],
      primaryIdentifier: [],
    };

    const domain = createDefaultProp(
      "domain",
      schema.properties,
      onlyProperties,
      schema,
    );

    const resourceValue = createDefaultProp(
      "resource_value",
      schema.properties,
      onlyProperties,
      schema,
    );

    const variant: ExpandedSchemaVariantSpec = {
      version,
      data: {
        version,
        link: schema.documentationUrl || null,
        color: "#FF9900",
        displayName: null, // siPkg does not store this
        componentType: "component",
        funcUniqueId: assetFuncUniqueKey,
        description: null, // TODO: can we get this?
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
      secrets: createDefaultPropFromCf("secrets", {}, {}, onlyProperties),
      secretDefinition: null,
      resourceValue,
      rootPropFuncs: [],
      cfSchema: undefined,
    };

    const moduleSchema: ExpandedSchemaSpec = {
      name: schema.typeName,
      data: {
        name: schema.typeName,
        category: `Hetzner::${schema.typeName}::${schema.typeName}`,
        categoryName: null,
        uiHidden: false,
        defaultSchemaVariant: variantUniqueKey,
      },
      uniqueId: schemaUniqueKey,
      deleted: false,
      isBuiltin,
      variants: [variant],
    };

    specs.push({
      kind: "module",
      name: schema.typeName,
      version,
      description: "",
      createdAt: new Date().toISOString(),
      createdBy: "Clover",
      defaultChangeSet: null,
      workspacePk: null,
      workspaceName: null,
      schemas: [moduleSchema],
      funcs: [],
      changeSets: [], // always empty
    });
  });

  return specs;
}

const MAX_PROP_DEPTH = 30;

function createDocLink(
  { typeName }: CfSchema,
  defName: string | undefined,
  propName?: string,
): string {
  return "https://LATERGATOR";
}

function childIsRequired(
  schema: HetznerSchema,
  parentProp: ExpandedPropSpecFor["object" | "array" | "map"] | undefined,
  childName: string,
) {
  // not correctly accounting for depth here, parent prop path is missing
  // probably need to embed `required` into the record of properties somehow
  return schema.requiredProperties.has(childName);
}

function createDefaultProp(
  name: DefaultPropType,
  properties: Record<string, CfProperty>,
  onlyProperties: OnlyProperties,
  cfSchema: HetznerSchema,
) {
  let rootProp: ExpandedPropSpecFor["object"] | undefined;

  const queue: HQueue = {
    cfSchema,
    onlyProperties,
    queue: [
      {
        propPath: ["root", name],
        // Pretend the prop only has the specified properties (since we split it up)
        cfProp: { ...cfSchema, properties },
        parentProp: undefined,
        addTo: (prop: ExpandedPropSpec) => {
          if (prop.kind !== "object") {
            throw new Error(`${name} prop is not an object`);
          }
          // Set "rootProp" before returning it
          rootProp = prop;
        },
      },
    ],
  };

  while (queue.queue.length > 0) {
    const propArgs = queue.queue.shift()!;
    if (propArgs.propPath.length > MAX_PROP_DEPTH) {
      throw new Error(
        `Prop tree loop detected: Tried creating prop more than ${MAX_PROP_DEPTH} levels deep in the prop tree: ${propArgs.propPath}`,
      );
    }

    const prop = createPropFromCf(
      propArgs,
      queue,
      createDocLink,
      childIsRequired,
    );
    if (!prop) continue;
    if (propArgs.addTo) propArgs.addTo(prop);
  }

  if (!rootProp) {
    throw new Error(
      `createProp for ${cfSchema.typeName} did not generate a ${name} prop`,
    );
  }

  // Top level prop doesn't actually get the generated doc; we add that to the schema instead
  rootProp.data.inputs = null;
  rootProp.data.widgetOptions = null;
  rootProp.data.hidden = false;
  rootProp.data.documentation = null;

  return rootProp;
}

function versionFromDate(): string {
  return new Date()
    .toISOString()
    .replace(/[-:T.Z]/g, "")
    .slice(0, 14);
}
