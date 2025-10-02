import { ExpandedPkgSpec } from "../../spec/pkgs.ts";
import rawSchema from "../../provider-schemas/hetzner.json" with {
  type: "json",
};
import { getExistingSpecs } from "../../specUpdates.ts";
import { generateIntrinsicFuncs } from "../generic/generateIntrinsicFuncs.ts";
import { createSuggestionsForPrimaryIdentifiers } from "../generic/createSuggestionsAcrossAssets.ts";
import { reorderProps } from "../generic/reorderProps.ts";
import { updateSchemaIdsForExistingSpecs } from "../generic/updateSchemaIdsForExistingSpecs.ts";
import { generateAssetFuncs } from "../generic/generateAssetFuncs.ts";
import { generateDefaultFuncsFromConfig, generateSpecsFromRawSchema } from "../generic/index.ts";
import { addDefaultProps } from "./pipeline-steps/addDefaultProps.ts";
import { hetznerProviderConfig } from "./provider.ts";

export async function generateHetznerSpecs(options: {
  forceUpdateExistingPackages?: boolean;
  moduleIndexUrl: string;
  docLinkCache: string;
  inferred: string;
  services?: string[];
}): Promise<ExpandedPkgSpec[]> {
  let specs: ExpandedPkgSpec[] = [];

  const existing_specs = await getExistingSpecs(options);

  // Generate base specs from Hetzner schema using the new generic helper
  specs = generateSpecsFromRawSchema(rawSchema, hetznerProviderConfig);
  specs = addDefaultProps(specs);

  specs = generateDefaultFuncsFromConfig(specs, hetznerProviderConfig);
  specs = generateIntrinsicFuncs(specs);
  specs = createSuggestionsForPrimaryIdentifiers(specs);

  specs = reorderProps(specs);
  specs = generateAssetFuncs(specs);
  specs = updateSchemaIdsForExistingSpecs(existing_specs, specs);

  return specs;
}
