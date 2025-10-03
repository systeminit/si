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
import { azureProviderConfig } from "./provider.ts";
import { JsonSchema } from "./schema.ts";
import { addDefaultProps } from "./pipeline-steps/addDefaultProps.ts";

export async function generateAzureSpecs(options: {
  forceUpdateExistingPackages?: boolean;
  moduleIndexUrl: string;
  docLinkCache: string;
  inferred: string;
  services?: string[];
}): Promise<ExpandedPkgSpec[]> {
  let specs: ExpandedPkgSpec[] = [];

  const existing_specs = await getExistingSpecs(options);

  try {
    const schemasDir = "./src/provider-schemas/azure";
    const rawSchemas: JsonSchema[] = [];

    // Load all service schema files
    for await (const entry of Deno.readDir(schemasDir)) {
      if (entry.isFile && entry.name.endsWith(".json")) {
        const schemaPath = `${schemasDir}/${entry.name}`;
        const schema = JSON.parse(await Deno.readTextFile(schemaPath));
        rawSchemas.push(schema);
      }
    }

    // Generate specs from all raw schemas
    for (const rawSchema of rawSchemas) {
      const serviceSpecs = generateSpecsFromRawSchema(
        rawSchema,
        azureProviderConfig,
      );
      specs.push(...serviceSpecs);
    }

    specs = addDefaultProps(specs);

    specs = generateDefaultFuncsFromConfig(specs, azureProviderConfig);
    specs = generateIntrinsicFuncs(specs);
    specs = createSuggestionsForPrimaryIdentifiers(specs);
    specs = reorderProps(specs);
    specs = generateAssetFuncs(specs);
    specs = updateSchemaIdsForExistingSpecs(existing_specs, specs);
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    console.error(`Error generating Azure specs: ${errorMessage}`);
    throw error;
  }

  return specs;
}
