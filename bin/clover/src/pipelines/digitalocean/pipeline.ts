import { ExpandedPkgSpec } from "../../spec/pkgs.ts";
import { PipelineOptions } from "../types.ts";
import { digitalOceanParseRawSchema, digitalOceanProviderConfig } from "./provider.ts";
import { generateDefaultFuncsFromConfig } from "../generic/index.ts";
import { getExistingSpecs } from "../../specUpdates.ts";
import { generateIntrinsicFuncs } from "../generic/generateIntrinsicFuncs.ts";
import { reorderProps } from "../generic/reorderProps.ts";
import { updateSchemaIdsForExistingSpecs } from "../generic/updateSchemaIdsForExistingSpecs.ts";
import { generateAssetFuncs } from "../generic/generateAssetFuncs.ts";
import { applyAssetOverrides } from "../generic/applyAssetOverrides.ts";
import { createSuggestionsForPrimaryIdentifiers } from "../generic/createSuggestionsAcrossAssets.ts";
import { readDigitalOceanOpenApiSpec } from "./schema.ts";
import { addDefaultProps } from "./pipeline-steps/addDefaultProps.ts";
import path from "node:path";

export async function generateDigitalOceanSpecs(
  options: PipelineOptions,
): Promise<ExpandedPkgSpec[]> {
  let specs: ExpandedPkgSpec[] = [];

  const existing_specs = await getExistingSpecs(options);

  const schemaPath = path.join(options.providerSchemasPath, "digitalocean.json");
  const openApiSpec = await readDigitalOceanOpenApiSpec(schemaPath);
  specs = digitalOceanParseRawSchema(openApiSpec);

  specs = addDefaultProps(specs);

  // Apply standard pipeline steps
  specs = generateDefaultFuncsFromConfig(specs, digitalOceanProviderConfig);
  specs = generateIntrinsicFuncs(specs);
  specs = createSuggestionsForPrimaryIdentifiers(specs);

  // Apply provider-specific overrides
  specs = applyAssetOverrides(specs, digitalOceanProviderConfig);

  specs = reorderProps(specs);
  specs = generateAssetFuncs(specs);
  specs = updateSchemaIdsForExistingSpecs(existing_specs, specs);

  return specs;
}
