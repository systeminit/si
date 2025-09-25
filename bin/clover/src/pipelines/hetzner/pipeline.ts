import { ExpandedPkgSpec } from "../../spec/pkgs.ts";
import _ from "npm:lodash";
import { createDefaultPropFromCf, OnlyProperties } from "../../spec/props.ts";
import rawSchema from "../../provider-schemas/hetzner.json" with {
  type: "json",
};
import { getExistingSpecs } from "../../specUpdates.ts";
import { HDB, HetznerSchema, SuperSchema } from "../types.ts";
import { makeModule } from "../generic/index.ts";
import { generateIntrinsicFuncs } from "../generic/generateIntrinsicFuncs.ts";
import { createSuggestionsForPrimaryIdentifiers } from "../generic/createSuggestionsAcrossAssets.ts";
import { reorderProps } from "../generic/reorderProps.ts";
import { updateSchemaIdsForExistingSpecs } from "../generic/updateSchemaIdsForExistingSpecs.ts";
import { generateAssetFuncs } from "../generic/generateAssetFuncs.ts";
import { createDefaultProp } from "./prop.ts";
import { generateCredentialModule } from "./credential.ts";
import { generateDefaultActionFuncs } from "./pipeline-steps/generateDefaultActionFuncs.ts";
import { generateDefaultLeafFuncs } from "./pipeline-steps/generateDefaultLeafFuncs.ts";
import { generateDefaultManagementFuncs } from "./pipeline-steps/generateDefaultManagementFuncs.ts";
import { generateDefaultQualificationFuncs } from "./pipeline-steps/generateQualificationFuncs.ts";

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
  specs = generateCredentialModule(specs);

  specs = generateIntrinsicFuncs(specs);
  specs = createSuggestionsForPrimaryIdentifiers(specs);

  specs = generateDefaultActionFuncs(specs);
  specs = generateDefaultLeafFuncs(specs);
  specs = generateDefaultManagementFuncs(specs);
  specs = generateDefaultQualificationFuncs(specs);

  specs = reorderProps(specs);
  specs = generateAssetFuncs(specs);
  specs = updateSchemaIdsForExistingSpecs(existing_specs, specs);

  console.log(specs);
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
        description: "PAUL FIGURE IT OUT",
        properties,
        requiredProperties,
        primaryIdentifier: ["id"],
      };
      schemas[noun] = schema;
    },
  );

  Object.values(schemas).forEach((schema: HetznerSchema) => {
    const onlyProperties: OnlyProperties = {
      createOnly: [],
      readOnly: [],
      writeOnly: [],
      primaryIdentifier: ["id"],
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

    const secrets = createDefaultPropFromCf(
      "secrets",
      {},
      schema,
      onlyProperties,
    );

    const m = makeModule(
      schema,
      createDocLink(schema, undefined),
      schema.description,
      domain,
      resourceValue,
      secrets,
      hCategory,
    );
    specs.push(m);
  });

  return specs;
}

function createDocLink(
  { typeName }: SuperSchema,
  defName: string | undefined,
  propName?: string,
): string {
  return "https://LATERGATOR";
}

export function hCategory(schema: SuperSchema): string {
  const name = _.camelCase(schema.typeName.replace("_", " "));
  return `Hetzner::${name}`;
}
