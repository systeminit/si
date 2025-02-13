import { getServiceByName, loadCfDatabase } from "../cfDb.ts";
import { pkgSpecFromCf } from "../specPipeline.ts";
import { generateAssetFuncs } from "../pipeline-steps/generateAssetFuncs.ts";
import { generateDefaultActionFuncs } from "../pipeline-steps/generateActionFuncs.ts";
import { generateDefaultLeafFuncs } from "../pipeline-steps/generateLeafFuncs.ts";
import { generateDefaultQualificationFuncs } from "../pipeline-steps/generateQualificationFuncs.ts";
import { generateDefaultManagementFuncs } from "../pipeline-steps/generateManagementFuncs.ts";
import { addDefaultPropsAndSockets } from "../pipeline-steps/addDefaultPropsAndSockets.ts";
import { generateSubAssets } from "../pipeline-steps/generateSubAssets.ts";
import { generateIntrinsicFuncs } from "../pipeline-steps/generateIntrinsicFuncs.ts";
import { createInputSocketsBasedOnOutputSockets } from "../pipeline-steps/createInputSocketsAcrossAssets.ts";
import { emptyDirectory } from "../util.ts";
import { updateSchemaIdsForExistingSpecs } from "../pipeline-steps/updateSchemaIdsForExistingSpecs.ts";
import { getExistingSpecs } from "../specUpdates.ts";

import _logger from "../logger.ts";
import { assetSpecificOverrides } from "../pipeline-steps/assetSpecificOverrides.ts";
import { addSignatureToCategoryName } from "../pipeline-steps/addSignatureToCategoryName.ts";
import { generateOutputSocketsFromProps } from "../pipeline-steps/generateOutputSocketsFromProps.ts";
import { ExpandedPkgSpec } from "../spec/pkgs.ts";
import { createPolicyDocumentInputSockets } from "../pipeline-steps/createPolicyDocumentInputSockets.ts";
import { prettifySocketNames } from "../pipeline-steps/prettifySocketNames.ts";

const logger = _logger.ns("siSpecs").seal();
const SI_SPEC_DIR = "si-specs";

export function generateSiSpecForService(serviceName: string) {
  const cf = getServiceByName(serviceName);
  return pkgSpecFromCf(cf);
}

export async function generateSiSpecs(
  moduleIndexUrl: string,
  services?: string[],
) {
  if (services?.length == 0) services = undefined;
  const db = await loadCfDatabase({ services });
  const existing_specs = await getExistingSpecs(moduleIndexUrl);

  let imported = 0;
  let importSubAssets = 0;
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
  specs = generateOutputSocketsFromProps(specs);
  specs = addDefaultPropsAndSockets(specs);
  specs = generateDefaultActionFuncs(specs);
  specs = generateDefaultLeafFuncs(specs);
  specs = generateDefaultManagementFuncs(specs);
  specs = generateDefaultQualificationFuncs(specs);
  // subAssets should not have any of the above, but need an asset func and
  // intrinsics
  specs = generateSubAssets(specs);
  specs = generateIntrinsicFuncs(specs);
  specs = createPolicyDocumentInputSockets(specs);
  // don't generate input sockets until we have all of the output sockets
  specs = createInputSocketsBasedOnOutputSockets(specs);
  specs = prettifySocketNames(specs);

  // Our overrides right now only run after the prop tree and the sockets are generated
  specs = assetSpecificOverrides(specs);

  // These need everything to be complete
  specs = generateAssetFuncs(specs);
  specs = updateSchemaIdsForExistingSpecs(existing_specs, specs);
  specs = addSignatureToCategoryName(specs);

  // WRITE OUTS SPECS
  await emptyDirectory(SI_SPEC_DIR);
  for (const spec of specs) {
    const specJson = JSON.stringify(spec, null, 2);
    const name = spec.name;

    try {
      logger.debug(`Writing ${name}.json`);
      await Deno.writeTextFile(`${SI_SPEC_DIR}/${name}.json`, specJson);
    } catch (e) {
      console.log(`Error writing to file: ${name}: ${e}`);
      continue;
    }

    if (name.includes("::")) {
      imported += 1;
    } else {
      importSubAssets += 1;
    }
  }

  console.log(
    `built ${imported} out of ${cfSchemas.length}, including ${importSubAssets} sub-assets`,
  );
}
