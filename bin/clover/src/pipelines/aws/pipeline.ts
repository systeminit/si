import { assetSpecificOverrides } from "./pipeline-steps/assetSpecificOverrides.ts";
import { loadCfDatabase } from "../../cfDb.ts";
import { pkgSpecFromCf } from "./pipeline-steps/specPipeline.ts";
import { generateAssetFuncs } from "./pipeline-steps/generateAssetFuncs.ts";
import { attachDefaultActionFuncs } from "./pipeline-steps/attachDefaultActionFuncs.ts";
import { generateDefaultLeafFuncs } from "./pipeline-steps/generateDefaultLeafFuncs.ts";
import { generateDefaultQualificationFuncs } from "./pipeline-steps/generateQualificationFuncs.ts";
import { attachDefaultManagementFuncs } from "./pipeline-steps/attachDefaultManagementFuncs.ts";
import { addDefaultPropsAndSockets } from "./pipeline-steps/addDefaultPropsAndSockets.ts";
import { generateSubAssets } from "./pipeline-steps/generateSubAssets.ts";
import { generateIntrinsicFuncs } from "./pipeline-steps/generateIntrinsicFuncs.ts";
import { updateSchemaIdsForExistingSpecs } from "./pipeline-steps/updateSchemaIdsForExistingSpecs.ts";
import { getExistingSpecs } from "../../specUpdates.ts";

import _logger from "../../logger.ts";
import { ExpandedPkgSpec } from "../../spec/pkgs.ts";
import { loadInferred } from "../../spec/inferred.ts";
import { addInferredEnums } from "./pipeline-steps/addInferredEnums.ts";
import { pruneCfAssets } from "./pipeline-steps/pruneCfAssets.ts";
import { removeUnneededAssets } from "./pipeline-steps/removeUnneededAssets.ts";
import { removeBadDocLinks } from "./pipeline-steps/removeBadDocLinks.ts";
import { reorderProps } from "./pipeline-steps/reorderProps.ts";
import { createSuggestionsForPrimaryIdentifiers } from "./pipeline-steps/createSuggestionsAcrossAssets.ts";
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
  specs = attachDefaultActionFuncs(specs);
  specs = generateDefaultLeafFuncs(specs);
  specs = attachDefaultManagementFuncs(specs);
  specs = generateDefaultQualificationFuncs(specs);

  // subAssets should not have any of the above, but need an asset func and
  // intrinsics
  specs = generateSubAssets(specs);
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
