import { ExpandedPkgSpec } from "../../spec/pkgs.ts";
import { getExistingSpecs } from "../../specUpdates.ts";
import { generateIntrinsicFuncs } from "../generic/generateIntrinsicFuncs.ts";
import { createSuggestionsForPrimaryIdentifiers } from "../generic/createSuggestionsAcrossAssets.ts";
import { reorderProps } from "../generic/reorderProps.ts";
import { updateSchemaIdsForExistingSpecs } from "../generic/updateSchemaIdsForExistingSpecs.ts";
import { generateAssetFuncs } from "../generic/generateAssetFuncs.ts";
import {
  generateDefaultFuncsFromConfig,
  generateSpecsFromRawSchema,
} from "../generic/index.ts";
import { dummyProviderConfig } from "./provider.ts";

export async function generateDummySpecs(options: {
  forceUpdateExistingPackages?: boolean;
  moduleIndexUrl: string;
  docLinkCache: string;
  inferred: string;
  services?: string[];
}): Promise<ExpandedPkgSpec[]> {
  let specs: ExpandedPkgSpec[] = [];

  const existing_specs = await getExistingSpecs(options);

  // Generate base specs from dummy schemas using the new generic helper
  // Pass null as rawSchema since dummy uses parseRawSchema to return hardcoded schemas
  specs = generateSpecsFromRawSchema(null, dummyProviderConfig);

  // Run through standard pipeline steps
  specs = generateIntrinsicFuncs(specs);
  specs = createSuggestionsForPrimaryIdentifiers(specs);

  specs = generateDefaultFuncsFromConfig(specs, dummyProviderConfig);

  specs = reorderProps(specs);
  specs = generateAssetFuncs(specs);
  specs = updateSchemaIdsForExistingSpecs(existing_specs, specs);

  return specs;
}
