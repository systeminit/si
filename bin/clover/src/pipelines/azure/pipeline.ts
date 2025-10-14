import { ExpandedPkgSpec } from "../../spec/pkgs.ts";
import { PipelineOptions } from "../types.ts";
import { generateDefaultFuncsFromConfig } from "../generic/index.ts";
import { getExistingSpecs } from "../../specUpdates.ts";
import { generateIntrinsicFuncs } from "../generic/generateIntrinsicFuncs.ts";
import { createSuggestionsForPrimaryIdentifiers } from "../generic/createSuggestionsAcrossAssets.ts";
import { reorderProps } from "../generic/reorderProps.ts";
import { updateSchemaIdsForExistingSpecs } from "../generic/updateSchemaIdsForExistingSpecs.ts";
import { generateAssetFuncs } from "../generic/generateAssetFuncs.ts";
import { applyAssetOverrides } from "../generic/applyAssetOverrides.ts";
import { addDefaultProps } from "./pipeline-steps/addDefaultProps.ts";
import path from "node:path";
import { azureRestApiSpecsRepo } from "./provider.ts";
import { readAzureSwaggerSpec } from "./schema.ts";
import { parseAzureSpec } from "./spec.ts";
import { assert } from "node:console";

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
  specs = createSuggestionsForPrimaryIdentifiers(specs);

  // Apply provider-specific overrides
  specs = applyAssetOverrides(specs, azureConfig);

  specs = reorderProps(specs);
  specs = generateAssetFuncs(specs);
  specs = updateSchemaIdsForExistingSpecs(existingSpecs, specs);

  return specs;
}

const EXCLUDE_SPECS = [
  // The dereferencer has trouble with # formats like "$ref": "#/parameters/projectTask" for whatever reason
  "/azure-rest-api-specs/specification/cognitiveservices/data-plane/QnAMaker/stable/v4.0/QnAMaker.json",
  "/azure-rest-api-specs/specification/datamigration/resource-manager/Microsoft.DataMigration/DataMigration/stable/2025-06-30/datamigration.json",
  "/azure-rest-api-specs/specification/machinelearningservices/resource-manager/Microsoft.MachineLearningServices/stable/2025-09-01/machineLearningServices.json",
  "/azure-rest-api-specs/specification/managementgroups/resource-manager/Microsoft.Management/ManagementGroups/stable/2023-04-01/management.json",
  "/azure-rest-api-specs/specification/securityinsights/resource-manager/Microsoft.SecurityInsights/stable/2025-09-01/Metadata.json",
  "/azure-rest-api-specs/specification/workloads/resource-manager/Microsoft.Workloads/stable/2023-04-01/monitors.json",
  "/azure-rest-api-specs/specification/securityinsights/resource-manager/Microsoft.SecurityInsights/stable/2025-09-01/ContentTemplates.json",
  "/azure-rest-api-specs/specification/securityinsights/resource-manager/Microsoft.SecurityInsights/stable/2025-09-01/ContentPackages.json",
];

async function* findLatestAzureSpecFiles(dir: string): AsyncGenerator<string> {
  // Update the bin/clover/src/provider-schemas/azure-rest-api-specs submodule
  const command = new Deno.Command("git", {
    args: ["submodule", "update", "--init"],
  });
  const { code, stderr } = await command.output();
  if (code !== 0) {
    const errorText = new TextDecoder().decode(stderr);
    throw new Error(`Failed to update Azure specs: ${errorText}`);
  }

  // Now find the latest stable (or preview if no stable) version in each service directory
  let latest: { parent: "stable" | "preview"; version: string } | undefined;
  for await (const entry of Deno.readDir(dir)) {
    if (entry.isDirectory) {
      const entryPath = path.join(dir, entry.name);
      // If it's a "stable" directory, look for the latest version and yield its specs
      if (entry.name === "stable" || entry.name === "preview") {
        // Pick the directory with the latest version
        for await (const version of Deno.readDir(entryPath)) {
          if (version.isDirectory) {
            if (
              !latest ||
              (latest.parent === entry.name && version.name > latest.version) ||
              (latest.parent === "preview" && entry.name === "stable")
            ) {
              latest = { parent: entry.name, version: version.name };
            }
          }
        }

        if (entry.name === "stable") {
          assert(latest, `No latest version in ${entryPath}`);
        }
      } else {
        yield* findLatestAzureSpecFiles(entryPath);
      }
    }
  }

  if (latest) {
    // Read the specs
    const latestVersionDir = path.join(dir, latest.parent, latest.version);
    let foundFiles = false;
    for await (const spec of Deno.readDir(latestVersionDir)) {
      if (spec.isFile && spec.name.endsWith(".json")) {
        const specPath = path.join(latestVersionDir, spec.name);
        foundFiles = true;
        if (!EXCLUDE_SPECS.some((s) => specPath.endsWith(s))) {
          yield specPath;
        }
      }
    }
    assert(foundFiles, `No spec files found in ${latestVersionDir}`);
  }
}

export async function getLatestAzureSpecs(options: PipelineOptions) {
  const specs: ExpandedPkgSpec[] = [];

  const specsRoot = path.join(azureRestApiSpecsRepo(options), "specification");
  console.log(`Loading Azure specs from ${specsRoot} ...`);
  let processed = 0;
  for await (const specPath of findLatestAzureSpecFiles(specsRoot)) {
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
