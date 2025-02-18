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
import {
  ExpandedPkgSpec,
  ExpandedSchemaSpec,
  ExpandedSchemaVariantSpec,
} from "../spec/pkgs.ts";
import { createPolicyDocumentInputSockets } from "../pipeline-steps/createPolicyDocumentInputSockets.ts";
import { prettifySocketNames } from "../pipeline-steps/prettifySocketNames.ts";
import { loadInferred } from "../spec/inferred.ts";
import { addInferredEnums } from "../pipeline-steps/addInferredEnums.ts";
import { PkgSpec } from "../bindings/PkgSpec.ts";
import { ignoreSpecsWithoutHandlers } from "../pipeline-steps/ignoreSpecsWithoutHandlers.ts";
import { SchemaVariantSpec } from "../bindings/SchemaVariantSpec.ts";
import { SchemaSpec } from "../bindings/SchemaSpec.ts";

const logger = _logger.ns("siSpecs").seal();
const SI_SPEC_DIR = "si-specs";

export function generateSiSpecForService(serviceName: string) {
  const cf = getServiceByName(serviceName);
  return pkgSpecFromCf(cf);
}

export async function generateSiSpecs(
  options: {
    forceUpdateExistingPackages?: boolean;
    moduleIndexUrl: string;
    inferred: string;
    services?: string[];
  },
) {
  const db = await loadCfDatabase(options);
  const existing_specs = await getExistingSpecs(options);
  const inferred = await loadInferred(options.inferred);

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
  specs = ignoreSpecsWithoutHandlers(specs);
  specs = addInferredEnums(specs, inferred);
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
    const specJson = JSON.stringify(unexpandPackageSpec(spec), null, 2);
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

function unexpandPackageSpec(expandedSpec: ExpandedPkgSpec): PkgSpec {
  // Take out cfSchema and other props we don't want in the final spec
  return {
    ...expandedSpec,
    schemas: expandedSpec.schemas.map(unexpandSchema),
  };
}

function unexpandSchema(
  expanded: ExpandedSchemaSpec,
): SchemaSpec {
  return {
    ...expanded,
    variants: expanded.variants.map(unexpandVariant),
  };
}
function unexpandVariant(
  expanded: ExpandedSchemaVariantSpec,
): SchemaVariantSpec {
  const { cfSchema: _, ...variant } = expanded;
  return variant;
}
