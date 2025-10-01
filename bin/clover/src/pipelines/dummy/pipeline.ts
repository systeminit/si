import { ExpandedPkgSpec } from "../../spec/pkgs.ts";
import { getExistingSpecs } from "../../specUpdates.ts";
import { generateIntrinsicFuncs } from "../generic/generateIntrinsicFuncs.ts";
import { createSuggestionsForPrimaryIdentifiers } from "../generic/createSuggestionsAcrossAssets.ts";
import { reorderProps } from "../generic/reorderProps.ts";
import { updateSchemaIdsForExistingSpecs } from "../generic/updateSchemaIdsForExistingSpecs.ts";
import { generateAssetFuncs } from "../generic/generateAssetFuncs.ts";
import { generateDefaultActionFuncs } from "../generic/generateDefaultActionFuncs.ts";
import { generateDefaultLeafFuncs } from "../generic/generateDefaultLeafFuncs.ts";
import { generateDefaultManagementFuncs } from "../generic/generateDefaultManagementFuncs.ts";
import { generateDefaultQualificationFuncs } from "../generic/generateDefaultQualificationFuncs.ts";
import {
  createDefaultActionFuncs,
  createDefaultCodeGenFuncs,
  createDefaultManagementFuncs,
  createDefaultQualificationFuncs,
} from "./funcs.ts";
import { pkgSpecFromDummy } from "./spec.ts";

export async function generateDummySpecs(options: {
  forceUpdateExistingPackages?: boolean;
  moduleIndexUrl: string;
  docLinkCache: string;
  inferred: string;
  services?: string[];
}): Promise<ExpandedPkgSpec[]> {
  let specs: ExpandedPkgSpec[] = [];

  const existing_specs = await getExistingSpecs(options);

  // Generate base specs from dummy schemas
  specs = pkgSpecFromDummy();

  // Run through standard pipeline steps
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