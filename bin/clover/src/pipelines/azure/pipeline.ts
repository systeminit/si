import { ExpandedPkgSpec } from "../../spec/pkgs.ts";
import { PipelineOptions } from "../types.ts";
import { generateDefaultFuncsFromConfig } from "../generic/index.ts";
import { getExistingSpecs } from "../../specUpdates.ts";
import { generateIntrinsicFuncs } from "../generic/generateIntrinsicFuncs.ts";
// import { createSuggestionsForPrimaryIdentifiers } from "../generic/createSuggestionsAcrossAssets.ts";
import { reorderProps } from "../generic/reorderProps.ts";
import { updateSchemaIdsForExistingSpecs } from "../generic/updateSchemaIdsForExistingSpecs.ts";
import { generateAssetFuncs } from "../generic/generateAssetFuncs.ts";
import { applyAssetOverrides } from "../generic/applyAssetOverrides.ts";
import { addDefaultProps } from "./pipeline-steps/addDefaultProps.ts";
import {
  findLatestAzureOpenApiSpecFiles,
  initAzureRestApiSpecsRepo,
  readAzureSwaggerSpec,
} from "./schema.ts";
import { parseAzureSpec } from "./spec.ts";

export async function generateAzureSpecs(
  options: PipelineOptions,
): Promise<ExpandedPkgSpec[]> {
  const azureConfig = (await import("./provider.ts")).AZURE_PROVIDER_CONFIG;

  const existingSpecs = await getExistingSpecs(options);
  let specs = await getLatestAzureSpecs(options);

  // Apply pipeline steps
  specs = addDefaultProps(specs);
  specs = generateDefaultFuncsFromConfig(specs, azureConfig);
  specs = generateIntrinsicFuncs(specs);
  // specs = createSuggestionsForPrimaryIdentifiers(specs);

  // Apply provider-specific overrides
  specs = applyAssetOverrides(specs, azureConfig);

  specs = reorderProps(specs);
  specs = generateAssetFuncs(specs);
  specs = updateSchemaIdsForExistingSpecs(existingSpecs, specs);

  return specs;
}

async function getLatestAzureSpecs(options: PipelineOptions) {
  const specsRepo = await initAzureRestApiSpecsRepo(options);
  console.log(`Loading Azure specs from ${specsRepo} ...`);

  const specs: ExpandedPkgSpec[] = [];
  let processed = 0;
  for await (const specPath of findLatestAzureOpenApiSpecFiles(specsRepo)) {
    try {
      const openApiSpec = await readAzureSwaggerSpec(specPath);
      const schemas = parseAzureSpec(openApiSpec);
      specs.push(...schemas);
    } catch (e) {
      console.error(`Failed to process ${specPath}: ${e}`);
      throw e;
    }
    processed++;
    if (processed % 50 === 0) {
      console.log(`Processed ${processed} specs...`);
    }
  }

  console.log(
    `Processed ${processed} OpenAPI specs and produced ${specs.length} schemas ...`,
  );

  return specs;
}
