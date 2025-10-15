import { ExpandedPkgSpec } from "../../spec/pkgs.ts";
import { getExistingSpecs } from "../../specUpdates.ts";
import { generateIntrinsicFuncs } from "../generic/generateIntrinsicFuncs.ts";
import { createSuggestionsForPrimaryIdentifiers } from "../generic/createSuggestionsAcrossAssets.ts";
import { reorderProps } from "../generic/reorderProps.ts";
import { updateSchemaIdsForExistingSpecs } from "../generic/updateSchemaIdsForExistingSpecs.ts";
import { generateAssetFuncs } from "../generic/generateAssetFuncs.ts";
import { generateDefaultFuncsFromConfig } from "../generic/index.ts";
import { applyAssetOverrides } from "../generic/applyAssetOverrides.ts";
import { DUMMY_PROVIDER_CONFIG, dummyParseRawSchema } from "./provider.ts";

export async function generateDummySpecs(options: {
  forceUpdateExistingPackages?: boolean;
  moduleIndexUrl: string;
  docLinkCache: string;
  inferred: string;
  services?: string[];
}): Promise<ExpandedPkgSpec[]> {
  let specs: ExpandedPkgSpec[] = [];

  const existing_specs = await getExistingSpecs(options);

  specs = dummyParseRawSchema({});

  specs = generateIntrinsicFuncs(specs);
  specs = createSuggestionsForPrimaryIdentifiers(specs);
  specs = generateDefaultFuncsFromConfig(specs, DUMMY_PROVIDER_CONFIG);
  specs = applyAssetOverrides(specs, DUMMY_PROVIDER_CONFIG);
  specs = reorderProps(specs);
  specs = generateAssetFuncs(specs);
  specs = updateSchemaIdsForExistingSpecs(existing_specs, specs);

  return specs;
}
