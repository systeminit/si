import { loadCfDatabase } from "../../cfDb.ts";
import { AWS_PROVIDER_CONFIG } from "./provider.ts";
import { generateDefaultFuncsFromConfig } from "../generic/index.ts";
import { addDefaultPropsAndSockets } from "./pipeline-steps/addDefaultPropsAndSockets.ts";
import { generateIntrinsicFuncs } from "./../generic/generateIntrinsicFuncs.ts";
import { getExistingSpecs } from "../../specUpdates.ts";

import _logger from "../../logger.ts";
import { ExpandedPkgSpec } from "../../spec/pkgs.ts";
import { loadInferred } from "../../spec/inferred.ts";
import { addInferredEnums } from "./pipeline-steps/addInferredEnums.ts";
import { pruneCfAssets } from "./pipeline-steps/pruneCfAssets.ts";
import { removeUnneededAssets } from "./pipeline-steps/removeUnneededAssets.ts";
import { applyAssetOverrides } from "../generic/applyAssetOverrides.ts";
import { removeBadDocLinks } from "./pipeline-steps/removeBadDocLinks.ts";
import { reorderProps } from "../generic/reorderProps.ts";
import { updateSchemaIdsForExistingSpecs } from "../generic/updateSchemaIdsForExistingSpecs.ts";
import { generateAssetFuncs } from "../generic/generateAssetFuncs.ts";
import { createSuggestionsForPrimaryIdentifiers } from "../generic/createSuggestionsAcrossAssets.ts";
import {
  createCredentialSuggestion,
  createRegionSuggestion,
} from "./pipeline-steps/genericAwsProperties.ts";
import { parseSchema } from "./spec.ts";

export async function generateAwsSpecs(options: {
  forceUpdateExistingPackages?: boolean;
  moduleIndexUrl: string;
  docLinkCache: string;
  inferred: string;
  services?: string[];
}): Promise<ExpandedPkgSpec[]> {
  const db = await loadCfDatabase(options);
  const existing_specs = await getExistingSpecs(options);
  const inferred = await loadInferred(options.inferred);

  let specs = parseSchema(db);

  // EXECUTE PIPELINE STEPS

  specs = await removeBadDocLinks(specs, options.docLinkCache);
  specs = addInferredEnums(specs, inferred);
  specs = addDefaultPropsAndSockets(specs);
  specs = generateDefaultFuncsFromConfig(specs, AWS_PROVIDER_CONFIG);
  specs = generateIntrinsicFuncs(specs);
  specs = removeUnneededAssets(specs);

  // this step will eventually replace all the socket stuff. Must come before
  // overrides so it can be... overriden
  specs = createSuggestionsForPrimaryIdentifiers(specs);
  specs = createRegionSuggestion(specs);
  specs = createCredentialSuggestion(specs);

  // prune assets that cannot be created by cloud control and must be create
  // using cf
  specs = pruneCfAssets(specs);

  // Apply provider-specific overrides
  specs = applyAssetOverrides(specs, AWS_PROVIDER_CONFIG);

  // These need everything to be complete
  specs = reorderProps(specs);
  specs = generateAssetFuncs(specs);
  specs = updateSchemaIdsForExistingSpecs(existing_specs, specs);

  return specs;
}
