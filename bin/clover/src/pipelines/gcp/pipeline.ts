import { ExpandedPkgSpec } from "../../spec/pkgs.ts";
import { PipelineOptions } from "../types.ts";
import { gcpProviderConfig } from "./provider.ts";
import { generateDefaultFuncsFromConfig } from "../generic/index.ts";
import { getExistingSpecs } from "../../specUpdates.ts";
import { generateIntrinsicFuncs } from "../generic/generateIntrinsicFuncs.ts";
import { reorderProps } from "../generic/reorderProps.ts";
import { updateSchemaIdsForExistingSpecs } from "../generic/updateSchemaIdsForExistingSpecs.ts";
import { generateAssetFuncs } from "../generic/generateAssetFuncs.ts";
import { applyAssetOverrides } from "../generic/applyAssetOverrides.ts";
import { addDefaultProps } from "./pipeline-steps/addDefaultProps.ts";
import { readGcpDiscoveryDocument } from "./schema.ts";
import { parseGcpDiscoveryDocument } from "./spec.ts";
import { join } from "https://deno.land/std@0.201.0/path/mod.ts";
import logger from "../../logger.ts";

export async function generateGcpSpecs(
  options: PipelineOptions,
): Promise<ExpandedPkgSpec[]> {
  const existingSpecs = await getExistingSpecs(options);
  let specs = await getGcpSpecs(options);

  // Apply pipeline steps
  specs = addDefaultProps(specs);
  specs = generateDefaultFuncsFromConfig(specs, gcpProviderConfig);
  specs = generateIntrinsicFuncs(specs);

  // Apply provider-specific overrides
  specs = applyAssetOverrides(specs, gcpProviderConfig);

  specs = reorderProps(specs);
  specs = generateAssetFuncs(specs);
  specs = updateSchemaIdsForExistingSpecs(existingSpecs, specs);

  return specs;
}

async function getGcpSpecs(
  options: PipelineOptions,
): Promise<ExpandedPkgSpec[]> {
  const schemaDir = join(options.providerSchemasPath, "gcp");
  logger.info(`Loading GCP discovery documents from ${schemaDir}...`);

  const specs: ExpandedPkgSpec[] = [];
  let processed = 0;
  let totalResources = 0;

  // Read all JSON files in the GCP schema directory
  for await (const entry of Deno.readDir(schemaDir)) {
    if (!entry.isFile || !entry.name.endsWith(".json")) {
      continue;
    }

    const filePath = join(schemaDir, entry.name);

    try {
      const doc = await readGcpDiscoveryDocument(filePath);
      const resourceSpecs = parseGcpDiscoveryDocument(doc);
      specs.push(...resourceSpecs);
      totalResources += resourceSpecs.length;

      processed++;
      if (processed % 10 === 0) {
        logger.info(
          `Processed ${processed} discovery documents (${totalResources} resources)...`,
        );
      }
    } catch (e) {
      logger.error(
        `Failed to process ${filePath}: ${
          e instanceof Error ? e.message : String(e)
        }`,
      );
      // Continue processing other files
    }
  }

  logger.info(
    `Processed ${processed} GCP discovery documents and produced ${specs.length} schemas`,
  );

  return specs;
}
