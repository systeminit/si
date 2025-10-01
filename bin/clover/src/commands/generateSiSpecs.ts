import { emptyDirectory } from "../util.ts";
import _logger from "../logger.ts";
import { PkgSpec } from "../bindings/PkgSpec.ts";
import { SchemaVariantSpec } from "../bindings/SchemaVariantSpec.ts";
import { SchemaSpec } from "../bindings/SchemaSpec.ts";
import { Provider } from "../types.ts";
import { bfsPropTree, ExpandedPropSpec } from "../spec/props.ts";
import {
  ExpandedPkgSpec,
  ExpandedSchemaSpec,
  ExpandedSchemaVariantSpec,
} from "../spec/pkgs.ts";
import { PropSpec } from "../bindings/PropSpec.ts";
import { generateAwsSpecs } from "../pipelines/aws/pipeline.ts";
import { generateHetznerSpecs } from "../pipelines/hetzner/pipeline.ts";
import { generateDummySpecs } from "../pipelines/dummy/pipeline.ts";

const logger = _logger.ns("siSpecs").seal();
const SI_SPEC_DIR = "si-specs";

export async function generateSiSpecs(options: {
  forceUpdateExistingPackages?: boolean;
  moduleIndexUrl: string;
  docLinkCache: string;
  inferred: string;
  services?: string[];
  provider: Provider;
}) {
  let specs: ExpandedPkgSpec[] = [];

  if (options.provider === "all") {
    // Generate specs for all providers
    const providers: Array<Exclude<Provider, "all">> = ["aws", "hetzner"];
    for (const provider of providers) {
      logger.info(`Generating specs for provider: ${provider}`);
      const providerOptions = { ...options, provider };
      let providerSpecs: ExpandedPkgSpec[] = [];

      switch (provider) {
        case "aws":
          providerSpecs = await generateAwsSpecs(providerOptions);
          break;
        case "hetzner":
          providerSpecs = await generateHetznerSpecs(providerOptions);
          break;
        case "dummy":
          providerSpecs = await generateDummySpecs(providerOptions);
          break;
      }

      specs.push(...providerSpecs);
    }
  } else {
    switch (options.provider) {
      case "aws":
        specs = await generateAwsSpecs(options);
        break;
      case "hetzner":
        specs = await generateHetznerSpecs(options);
        break;
      case "dummy":
        specs = await generateDummySpecs(options);
        break;
      default:
        console.log(`Unsupported provider type: "${options.provider}"`);
        Deno.exit();
    }
  }

  // WRITE OUT SPECS
  await emptyDirectory(SI_SPEC_DIR);
  let imported = 0;
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

    imported += 1;
  }

  console.log(
    `built ${imported} out of ${specs.length}`,
  );
}

function unexpandPackageSpec(expandedSpec: ExpandedPkgSpec): PkgSpec {
  // Take out cfSchema and other props we don't want in the final spec
  return {
    ...expandedSpec,
    schemas: expandedSpec.schemas.map(unexpandSchema),
  };
}

function unexpandSchema(expanded: ExpandedSchemaSpec): SchemaSpec {
  return {
    ...expanded,
    variants: expanded.variants.map(unexpandVariant),
  };
}

function unexpandVariant(
  expanded: ExpandedSchemaVariantSpec,
): SchemaVariantSpec {
  const { ...variant } = expanded;
  bfsPropTree(
    [
      variant.domain,
      variant.resourceValue,
      variant.secrets,
      variant.secretDefinition,
    ],
    unexpandProperty,
  );
  return { ...variant, sockets: [] };
}

function unexpandProperty(expanded: ExpandedPropSpec): PropSpec {
  const deleteable = expanded as Partial<ExpandedPropSpec>;
  delete deleteable.metadata;
  delete deleteable.joiValidation;
  delete deleteable.cfProp;
  return expanded;
}
