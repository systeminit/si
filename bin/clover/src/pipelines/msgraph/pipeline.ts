import { ExpandedPkgSpec } from "../../spec/pkgs.ts";
import { PipelineOptions } from "../types.ts";
import { entraParseRawSchema, msgraphProviderConfig } from "./provider.ts";
import { generateDefaultFuncsFromConfig } from "../generic/index.ts";
import { getExistingSpecs } from "../../specUpdates.ts";
import { generateIntrinsicFuncs } from "../generic/generateIntrinsicFuncs.ts";
import { reorderProps } from "../generic/reorderProps.ts";
import { updateSchemaIdsForExistingSpecs } from "../generic/updateSchemaIdsForExistingSpecs.ts";
import { generateAssetFuncs } from "../generic/generateAssetFuncs.ts";
import { applyAssetOverrides } from "../generic/applyAssetOverrides.ts";
import { createSuggestionsForPrimaryIdentifiers } from "../generic/createSuggestionsAcrossAssets.ts";
import { readEntraOpenApiSpec } from "./schema.ts";
import { addDefaultProps } from "./pipeline-steps/addDefaultProps.ts";
import path from "node:path";

export async function generateEntraSpecs(
  options: PipelineOptions,
): Promise<ExpandedPkgSpec[]> {
  let specs: ExpandedPkgSpec[] = [];

  const existing_specs = await getExistingSpecs(options);

  const schemaPath = path.join(options.providerSchemasPath, "msgraph.json");
  const openApiSpec = await readEntraOpenApiSpec(schemaPath);
  specs = entraParseRawSchema(openApiSpec);

  specs = addDefaultProps(specs);
  specs = generateDefaultFuncsFromConfig(specs, msgraphProviderConfig);
  specs = generateIntrinsicFuncs(specs);
  specs = createSuggestionsForPrimaryIdentifiers(specs);
  specs = applyAssetOverrides(specs, msgraphProviderConfig);
  specs = reorderProps(specs);
  specs = generateAssetFuncs(specs);
  specs = updateSchemaIdsForExistingSpecs(existing_specs, specs);

  return specs;
}
