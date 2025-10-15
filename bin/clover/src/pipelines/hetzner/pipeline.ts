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
import { generateDefaultFuncsFromConfig } from "../generic/index.ts";
import { addDefaultProps } from "./pipeline-steps/addDefaultProps.ts";
import { applyAssetOverrides } from "../generic/applyAssetOverrides.ts";
import { hetznerParseRawSchema, hetznerProviderConfig } from "./provider.ts";

export async function generateHetznerSpecs(options: {
  forceUpdateExistingPackages?: boolean;
  moduleIndexUrl: string;
  docLinkCache: string;
  inferred: string;
  services?: string[];
}): Promise<ExpandedPkgSpec[]> {
  let specs: ExpandedPkgSpec[] = [];

  const existing_specs = await getExistingSpecs(options);

  specs = hetznerParseRawSchema(rawSchema);

  specs = addDefaultProps(specs);

  specs = generateDefaultFuncsFromConfig(specs, hetznerProviderConfig);
  specs = generateIntrinsicFuncs(specs);
  specs = createSuggestionsForPrimaryIdentifiers(specs);

  // Apply provider-specific overrides
  specs = applyAssetOverrides(specs, hetznerProviderConfig);

  specs = reorderProps(specs);
  specs = generateAssetFuncs(specs);
  specs = updateSchemaIdsForExistingSpecs(existing_specs, specs);

  return specs;
}
