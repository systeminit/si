import { loadCfDatabase } from "../../cfDb.ts";
import { pkgSpecFromCf } from "./spec.ts";
import { awsProviderConfig } from "./provider.ts";
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
import { assetSpecificOverrides } from "./pipeline-steps/assetSpecificOverrides.ts";
import { removeBadDocLinks } from "./pipeline-steps/removeBadDocLinks.ts";
import { reorderProps } from "../generic/reorderProps.ts";
import { updateSchemaIdsForExistingSpecs } from "../generic/updateSchemaIdsForExistingSpecs.ts";
import { generateAssetFuncs } from "../generic/generateAssetFuncs.ts";
import { createSuggestionsForPrimaryIdentifiers } from "../generic/createSuggestionsAcrossAssets.ts";
import {
  createCredentialSuggestion,
  createRegionSuggestion,
} from "./pipeline-steps/genericAwsProperties.ts";

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

  const cfSchemas = Object.values(db);

  let specs = [] as ExpandedPkgSpec[];

  for (const cfSchema of cfSchemas) {
    try {
      const pkg = pkgSpecFromCf(cfSchema);

      specs.push(pkg);
    } catch (e) {
      console.log(`Error Building: ${cfSchema.typeName}: ${e}`);
    }
  }

  // EXECUTE PIPELINE STEPS

  specs = await removeBadDocLinks(specs, options.docLinkCache);
  specs = addInferredEnums(specs, inferred);
  specs = addDefaultPropsAndSockets(specs);
  specs = generateDefaultFuncsFromConfig(specs, awsProviderConfig);
  specs = generateIntrinsicFuncs(specs);
  specs = removeUnneededAssets(specs);

  // this step will eventually replace all the socket stuff. Must come before
  // overrides so it can be... overriden
  specs = createSuggestionsForPrimaryIdentifiers(specs);
  specs = createRegionSuggestion(specs);
  specs = createCredentialSuggestion(specs);

  // Our overrides right now only run after the prop tree and the sockets are generated
  specs = assetSpecificOverrides(specs);

  // prune assets that cannot be created by cloud control and must be create
  // using cf
  specs = pruneCfAssets(specs);

  // These need everything to be complete
  specs = reorderProps(specs);
  specs = generateAssetFuncs(specs);
  specs = updateSchemaIdsForExistingSpecs(existing_specs, specs);

  return specs;
}
