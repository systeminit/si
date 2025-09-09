import _ from "npm:lodash";

import { bfsPropTree } from "../spec/props.ts";
import pluralize from "npm:pluralize";
import { ExpandedPkgSpec } from "../spec/pkgs.ts";
import { addPropSuggestSource } from "../spec/props.ts";
import _logger from "../logger.ts";
const logger = _logger.ns("test").seal();

export function createSuggestionsForPrimaryIdentifiers(
  specs: ExpandedPkgSpec[],
): ExpandedPkgSpec[] {
  const newSpecs = [] as ExpandedPkgSpec[];
  const schemasToPrimaryIdents = new Map<string, [string, Set<string>]>();

  // gather up all the primary idents
  for (const spec of specs) {
    const [schema] = spec.schemas;
    const [schemaVariant] = schema.variants;
    const resource = schemaVariant.resourceValue;
    const specName = spec.name;
    const category = schema.data.category.split("::")[1];
    const variantName = specName.split("::")[2];

    bfsPropTree(
      resource,
      (prop) => {
        if (!prop.metadata?.primaryIdentifier) return;
        const propName = prop.name;
        const possibleNames = new Set<string>();

        if (propName !== "Id") possibleNames.add(propName);

        [
          variantName,
          `${variantName}${propName}`,
          `${category}${variantName}${propName}`,
          `${variantName} ${propName}`,
          `${category} ${variantName} ${propName}`,
        ].forEach((i) => possibleNames.add(i));

        if (propName.endsWith("Id")) {
          [
            `${variantName}${propName}entifier`,
            `${variantName}${propName}entifier`,
            `${category}${variantName}${propName}entifer`,
            `${variantName} ${propName}entifier`,
            `${variantName} ${propName}entifier`,
            `${category} ${variantName} ${propName}entifer`,
          ].forEach((i) => possibleNames.add(i));
        }

        const nameVariants = new Set<string>();
        for (const name of possibleNames) {
          const plural = pluralize(name);
          const spaced = camelToSpaced(name);
          const pluralSpaced = pluralize(spaced);

          nameVariants.add(name);
          nameVariants.add(plural);
          nameVariants.add(spaced);
          nameVariants.add(pluralSpaced);
          nameVariants.add(`${name}Item`);
          nameVariants.add(`${plural}Item`);
          nameVariants.add(`${spaced}Item`);
          nameVariants.add(`${pluralSpaced}Item`);
        }

        // stripping /root out
        const propPath = "/" + prop.metadata.propPath.slice(1).join("/");
        schemasToPrimaryIdents.set(specName, [propPath, nameVariants]);
      },
      { skipTypeProps: true },
    );
  }

  // iterate through every prop an create a suggestion if the names match
  for (const spec of specs) {
    const [schema] = spec.schemas;
    const [schemaVariant] = schema.variants;
    const domain = schemaVariant.domain;

    bfsPropTree(
      domain,
      (prop) => {
        for (const [
          specName,
          [propName, possibleNames],
        ] of schemasToPrimaryIdents.entries()) {
          if (spec.name != specName && possibleNames.has(prop.name)) {
            logger.debug(
              `suggest {schema:${specName}, prop:${propName}} for prop ${prop.name} on ${spec.name}`,
            );
            prop = addPropSuggestSource(prop, {
              schema: specName,
              prop: propName,
            });
          }
        }
      },
      { skipTypeProps: false },
    );

    newSpecs.push(spec);
  }
  return newSpecs;
}

function camelToSpaced(str: string): string {
  return str.replace(/([a-z])([A-Z])/g, "$1 $2");
}
