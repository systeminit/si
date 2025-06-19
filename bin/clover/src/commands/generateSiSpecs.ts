import { getServiceByName, loadCfDatabase } from "../cfDb.ts";
import { pkgSpecFromCf } from "../specPipeline.ts";
import { generateAssetFuncs } from "../pipeline-steps/generateAssetFuncs.ts";
import { attachDefaultActionFuncs } from "../pipeline-steps/attachDefaultActionFuncs.ts";
import { generateDefaultLeafFuncs } from "../pipeline-steps/generateDefaultLeafFuncs.ts";
import { generateDefaultQualificationFuncs } from "../pipeline-steps/generateQualificationFuncs.ts";
import { attachDefaultManagementFuncs } from "../pipeline-steps/attachDefaultManagementFuncs.ts";
import { addDefaultPropsAndSockets } from "../pipeline-steps/addDefaultPropsAndSockets.ts";
import { generateSubAssets } from "../pipeline-steps/generateSubAssets.ts";
import { generateIntrinsicFuncs } from "../pipeline-steps/generateIntrinsicFuncs.ts";
import { createInputSocketsBasedOnOutputSockets } from "../pipeline-steps/createInputSocketsAcrossAssets.ts";
import { emptyDirectory } from "../util.ts";
import { updateSchemaIdsForExistingSpecs } from "../pipeline-steps/updateSchemaIdsForExistingSpecs.ts";
import { getExistingSpecs } from "../specUpdates.ts";

import _logger from "../logger.ts";
import { assetSpecificOverrides } from "../pipeline-steps/assetSpecificOverrides.ts";
import { generateOutputSocketsFromProps } from "../pipeline-steps/generateOutputSocketsFromProps.ts";
import {
  ExpandedPkgSpec,
  ExpandedSchemaSpec,
  ExpandedSchemaVariantSpec,
} from "../spec/pkgs.ts";
import { createPolicyDocumentInputSockets } from "../pipeline-steps/createPolicyDocumentInputSockets.ts";
import { annotateCommonOutputSockets } from "../pipeline-steps/annotateCommonOutputSockets.ts";
import { prettifySocketNames } from "../pipeline-steps/prettifySocketNames.ts";
import { loadInferred } from "../spec/inferred.ts";
import { addInferredEnums } from "../pipeline-steps/addInferredEnums.ts";
import { PkgSpec } from "../bindings/PkgSpec.ts";
import { SchemaVariantSpec } from "../bindings/SchemaVariantSpec.ts";
import { SchemaSpec } from "../bindings/SchemaSpec.ts";
import { bfsPropTree, ExpandedPropSpec } from "../spec/props.ts";
import { PropSpec } from "../bindings/PropSpec.ts";
import { pruneCfAssets } from "../pipeline-steps/pruneCfAssets.ts";
import { removeUnneededAssets } from "../pipeline-steps/removeUnneededAssets.ts";
import {
  reportDeprecatedAssets,
} from "../pipeline-steps/reportDeprecatedAssets.ts";
import { removeBadDocLinks } from "../pipeline-steps/removeBadDocLinks.ts";
import { reorderProps } from "../pipeline-steps/reorderProps.ts";
import { createSuggestionsForPrimaryIdentifiers } from "../pipeline-steps/createSuggestionsAcrossAssets.ts";
import { createAwsRegionSpecificSuggestion } from "../pipeline-steps/awsRegionSpecificSuggestions.ts";

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
    docLinkCache: string;
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
  specs = await removeBadDocLinks(specs, options.docLinkCache);
  specs = addInferredEnums(specs, inferred);
  specs = generateOutputSocketsFromProps(specs);
  specs = annotateCommonOutputSockets(specs);
  specs = addDefaultPropsAndSockets(specs);
  specs = attachDefaultActionFuncs(specs);
  specs = generateDefaultLeafFuncs(specs);
  specs = attachDefaultManagementFuncs(specs);
  specs = generateDefaultQualificationFuncs(specs);
  // subAssets should not have any of the above, but need an asset func and
  // intrinsics
  specs = generateSubAssets(specs);
  specs = generateIntrinsicFuncs(specs);
  specs = createPolicyDocumentInputSockets(specs);
  // don't generate input sockets until we have all of the output sockets
  specs = createInputSocketsBasedOnOutputSockets(specs);
  specs = prettifySocketNames(specs);
  // remove these after socket generation so we can still connect to their
  // alternatives
  specs = removeUnneededAssets(specs);

  // this step will eventually replace all the socket stuff. Must come before
  // overrides so it can be... overriden
  specs = createSuggestionsForPrimaryIdentifiers(specs);
  specs = createAwsRegionSpecificSuggestion(specs);

  // Our overrides right now only run after the prop tree and the sockets are generated
  specs = assetSpecificOverrides(specs);

  // prune assets that cannot be created by cloud control and must be create
  // using cf
  specs = pruneCfAssets(specs);

  // These need everything to be complete
  specs = reorderProps(specs);
  specs = generateAssetFuncs(specs);
  specs = updateSchemaIdsForExistingSpecs(existing_specs, specs);

  // Reporting steps
  reportDeprecatedAssets(existing_specs, specs);

  // WRITE OUTS SPECS
  await emptyDirectory(SI_SPEC_DIR);
  for (const spec of specs) {
    const specJson = JSON.stringify(unexpandPackageSpec(spec), null, 2);
    const name = spec.name;

    try {
      logger.debug(`Writing ${name}.json`);
      const blob = new Blob([specJson]);
      if (blob.size > 4 * 1024 * 1024) {
        logger.warn(`${spec.name} is bigger than 4MBs. Skipping.`);
        continue;
      }
      await Deno.writeFile(`${SI_SPEC_DIR}/${name}.json`, blob.stream());
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
  bfsPropTree([
    variant.domain,
    variant.resourceValue,
    variant.secrets,
    variant.secretDefinition,
  ], unexpandProperty);
  return variant;
}

function unexpandProperty(expanded: ExpandedPropSpec): PropSpec {
  const deleteable = expanded as Partial<ExpandedPropSpec>;
  delete deleteable.metadata;
  delete deleteable.joiValidation;
  delete deleteable.cfProp;
  return expanded;
}
