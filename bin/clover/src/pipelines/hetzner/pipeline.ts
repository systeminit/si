import { CfProperty } from "../cfDb.ts";
import { ExpandedPkgSpec } from "../../spec/pkgs.ts";
import { ExpandedPropSpecFor, DefaultPropType, createPropFromCf } from "../../spec/props.ts";
import rawSchema from "../../provider-schemas/hetzner.json" with { type: "json" };

export type HetznerSchema = {
  typeName: string;
  sourceUrl?: string;
  documentationUrl?: string;
  properties: Record<string, CfProperty>;
  requiredProperties: Set<string>;
}

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
}

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

  for (schema in pkgSpecFromHetnzer(rawSchema)) {

  }

  return specs;
}

function pkgSpecFromHetnzer(allSchemas: object) {
  const schemas: HDB = {};
  Object.entries(allSchemas.paths).forEach([endpoint, openApiDescription] => {
    if (!openApiDescription.get) throw new Error(`WHY NO GET? ${noun}`)

    // skipping list endpoints for now
    if (openApiDescription.get.operationId.startsWith("list_")) continue;

    const noun = endpoint.split("/")[1]

    if (schemas[noun]) {
      console.error(`ALREADY GOT ${noun}`);
      continue;
    }

    const objShape = openApiDescription.get.responses["200"].content["application/json"].schema.properties[noun];
    const properties = objShape.properties
    const requiredProperties = new Set(objShape.required);
    schemas[noun] = {
      typeName: noun,
      properties,
      requiredProperties,
    }
  });

  Object.values(schemas).forEach((schema: HetznerSchema) => {
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
      cfSchema,
    );
  });
}

const MAX_PROP_DEPTH = 30;

function createDocLink(
  { typeName }: CfSchema,
  defName: string | undefined,
  propName?: string,
): string {
 return "LATER GATOR"
}

function childIsRequired(
  schema: HetznerSchema;
  parentProp: ExpandedPropSpecFor["object" | "array" | "map"] | undefined,
  childName: string,
) {
  // not correctly accounting for depth here, parent prop path is missing
  // probably need to embed `required` into the record of properties somehow
  return schema.requiredProperties.has(childName)
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
    ]
  };

  while (queue.queue.length > 0) {
    const propArgs = queue.queue.shift()!;
    if (propArgs.propPath.length > MAX_PROP_DEPTH) {
      throw new Error(
        `Prop tree loop detected: Tried creating prop more than ${MAX_PROP_DEPTH} levels deep in the prop tree: ${propArgs.propPath}`,
      );
    }

    const prop = createPropFromCf(propArgs, queue, createDocLink, childIsRequired);
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