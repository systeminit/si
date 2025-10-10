import { ExpandedPkgSpec } from "../../spec/pkgs.ts";
import { PipelineOptions } from "../types.ts";
import { join } from "https://deno.land/std@0.224.0/path/mod.ts";
import {
  generateDefaultFuncsFromConfig,
  generateSpecsFromRawSchema,
} from "../generic/index.ts";
import { getExistingSpecs } from "../../specUpdates.ts";
import { generateIntrinsicFuncs } from "../generic/generateIntrinsicFuncs.ts";
import { createSuggestionsForPrimaryIdentifiers } from "../generic/createSuggestionsAcrossAssets.ts";
import { reorderProps } from "../generic/reorderProps.ts";
import { updateSchemaIdsForExistingSpecs } from "../generic/updateSchemaIdsForExistingSpecs.ts";
import { generateAssetFuncs } from "../generic/generateAssetFuncs.ts";
import { applyAssetOverrides } from "../generic/applyAssetOverrides.ts";
import { addDefaultProps } from "./pipeline-steps/addDefaultProps.ts";

export async function generateAzureSpecs(
  options: PipelineOptions,
): Promise<ExpandedPkgSpec[]> {
  const azureConfig = (await import("./provider.ts")).azureProviderConfig;

  const schemasDir = "./src/provider-schemas/azure";
  let specs: ExpandedPkgSpec[] = [];

  const existing_specs = await getExistingSpecs(options);

  try {
    for await (const entry of Deno.readDir(schemasDir)) {
      if (!entry.isFile || !entry.name.endsWith(".json")) continue;

      const filePath = join(schemasDir, entry.name);
      const rawSchema = JSON.parse(await Deno.readTextFile(filePath));

      const serviceSpecs = generateSpecsFromRawSchema(rawSchema, azureConfig);
      specs.push(...serviceSpecs);
    }
  } catch (error) {
    if (error instanceof Deno.errors.NotFound) {
      console.warn(
        "Azure schemas not found. Run 'deno task run fetch-schema --provider=azure' first.",
      );
      return [];
    }
    throw error;
  }

  // Apply pipeline steps
  specs = addDefaultProps(specs);
  specs = generateDefaultFuncsFromConfig(specs, azureConfig);
  specs = generateIntrinsicFuncs(specs);
  specs = createSuggestionsForPrimaryIdentifiers(specs);

  // Apply provider-specific overrides
  specs = applyAssetOverrides(specs, azureConfig);

  specs = reorderProps(specs);
  specs = generateAssetFuncs(specs);
  specs = updateSchemaIdsForExistingSpecs(existing_specs, specs);

  return specs;
}
