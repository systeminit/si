import { CfProperty } from "../cfDb.ts";
import { ExpandedPkgSpec } from "../../spec/pkgs.ts";
import { ExpandedPropSpecFor } from "../../spec/props.ts";
import rawSchema from "../../provider-schemas/hetzner.json" with { type: "json" };

export type HetznerSchema = {
  typeName: string;
  description: string;
  primaryIdentifier: JSONPointer[];
  sourceUrl?: string;
  documentationUrl?: string;
  properties: Record<string, CfProperty>;
}

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
  const nouns = new Set();
  const schemas = Record<string, object> = {};
  Object.entries(allSchemas.paths).forEach([endpoint, openApiDescription] => {
    if (!openApiDescription.get) throw new Error(`WHY NO GET? ${noun}`)

    // skipping list endpoints for now
    if (openApiDescription.get.operationId.startsWith("list_")) continue;

    const noun = endpoint.split("/")[1]
    nouns.add(noun);
    const schemaProps = openApiDescription.get.responses.200.content["application/json"].schema.properties.[noun].properties
    schemas[noune] = schemaProps;
  });

  // create our own "tree"
  // for each schemaProp call createPropFromCf or something like it
}