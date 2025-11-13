import { emptyDirectory } from "../util.ts";
import _logger from "../logger.ts";
import { PkgSpec } from "../bindings/PkgSpec.ts";
import { SchemaVariantSpec } from "../bindings/SchemaVariantSpec.ts";
import { SchemaSpec } from "../bindings/SchemaSpec.ts";
import { bfsPropTree, ExpandedPropSpec } from "../spec/props.ts";
import {
  ExpandedPkgSpec,
  ExpandedSchemaSpec,
  ExpandedSchemaVariantSpec,
} from "../spec/pkgs.ts";
import { PropSpec } from "../bindings/PropSpec.ts";
import { PipelineOptions, selectedProviders } from "../pipelines/types.ts";
// Import all providers to ensure they register themselves
import "../pipelines/aws/spec.ts";
import "../pipelines/hetzner/provider.ts";
import "../pipelines/dummy/spec.ts";

// set this boolean to true to print out duplicates instead of throwing an error
const PRINT_DUPLICATES = false;

const logger = _logger.ns("siSpecs").seal();
const SI_SPEC_DIR = "si-specs";
const MAX_SPEC_SIZE_MB = 20;

export async function generateSiSpecs(options: PipelineOptions) {
  const specs: ExpandedPkgSpec[] = [];

  for (const config of selectedProviders(options)) {
    logger.info(`Generating specs for provider: ${config.name}`);
    const providerSpecs = await config.loadSchemas(options);
    specs.push(...providerSpecs);
  }

  // WRITE OUT SPECS
  await emptyDirectory(SI_SPEC_DIR);
  let imported = 0;
  const wroteSchemas = {} as Record<string, string[]>;
  const categories = {} as Record<string, string>;
  const multipleSchemas = {} as Record<string, string>;
  const multipleCategories = [] as Array<string>;
  for (const spec of specs) {
    const specJson = JSON.stringify(unexpandPackageSpec(spec), null, 2);
    const name = spec.name.replaceAll("/", ".");

    const schema = spec.schemas[0];

    // two schemas cannot have the same name, case-insensitive
    if (wroteSchemas[schema.name.toLowerCase()]) {
      if (!PRINT_DUPLICATES) {
        throw new Error(
          `Multiple ${schema.name} schemas, possibly with different case`,
        );
      }
      wroteSchemas[schema.name.toLowerCase()].push(schema.name);
      multipleSchemas[schema.name.toLowerCase()] = `${
        wroteSchemas[schema.name.toLowerCase()]
      }`;
    } else {
      wroteSchemas[schema.name.toLowerCase()] = [
        schema.name,
      ];
    }

    // two categories cannot have the same name with different case
    if (schema.data?.category) {
      const category = schema.data.category;
      const categoryLower = category.toLowerCase();
      categories[categoryLower] ??= category;
      const s = `${schema.name} - ${categories[categoryLower]} and ${category}`;
      if (
        categories[categoryLower] !== category &&
        !multipleCategories.includes(s)
      ) {
        if (!PRINT_DUPLICATES) {
          throw new Error(
            `Multiple categories with the same name but different case found for schema ${s}`,
          );
        }
        multipleCategories.push(s);
      }
    }

    logger.debug(`Writing ${name}.json`);
    const blob = new Blob([specJson]);
    if (blob.size > MAX_SPEC_SIZE_MB * 1024 * 1024) {
      // TODO throw an error, once we've got Azure specs under control
      console.warn(
        `${spec.name} is bigger than ${MAX_SPEC_SIZE_MB}MBs. Generating anyway ...`,
      );
    }
    await Deno.writeFile(`${SI_SPEC_DIR}/${name}.json`, blob.stream());

    imported += 1;
  }
  if (PRINT_DUPLICATES) {
    console.warn("Duplicate schema names -");
    console.warn(multipleSchemas);
    console.warn("Duplicate category names -");
    console.warn(multipleCategories);
  }

  console.log(`built ${imported} out of ${specs.length}`);
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
  const { superSchema: _, ...variant } = expanded;
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
