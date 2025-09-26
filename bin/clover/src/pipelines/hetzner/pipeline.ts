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
import { generateCredentialModule } from "./pipeline-steps/createCredential.ts";
import { generateDefaultActionFuncs } from "../generic/generateDefaultActionFuncs.ts";
import { generateDefaultLeafFuncs } from "../generic/generateDefaultLeafFuncs.ts";
import { generateDefaultManagementFuncs } from "../generic/generateDefaultManagementFuncs.ts";
import { generateDefaultQualificationFuncs } from "../generic/generateDefaultQualificationFuncs.ts";
import { addDefaultProps } from "./pipeline-steps/addDefaultProps.ts";
import {
  createDefaultActionFuncs,
  createDefaultCodeGenFuncs,
  createDefaultManagementFuncs,
  createDefaultQualificationFuncs,
} from "./funcs.ts";
import { pkgSpecFromHetnzer } from "./spec.ts";

export async function generateHetznerSpecs(options: {
  forceUpdateExistingPackages?: boolean;
  moduleIndexUrl: string;
  docLinkCache: string;
  inferred: string;
  services?: string[];
}): Promise<ExpandedPkgSpec[]> {
  let specs: ExpandedPkgSpec[] = [];

  const existing_specs = await getExistingSpecs(options);

  // Generate base specs from Hetzner schema
  specs = pkgSpecFromHetnzer(rawSchema);
  specs = addDefaultProps(specs);
  specs = generateCredentialModule(specs);

  specs = generateIntrinsicFuncs(specs);
  specs = createSuggestionsForPrimaryIdentifiers(specs);

  specs = generateDefaultActionFuncs(specs, createDefaultActionFuncs);
  specs = generateDefaultLeafFuncs(specs, createDefaultCodeGenFuncs);
  specs = generateDefaultManagementFuncs(specs, createDefaultManagementFuncs);
  specs = generateDefaultQualificationFuncs(
    specs,
    createDefaultQualificationFuncs,
  );

  specs = reorderProps(specs);
  specs = generateAssetFuncs(specs);
  specs = updateSchemaIdsForExistingSpecs(existing_specs, specs);

  return specs;
}
