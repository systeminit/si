import _logger from "../../logger.ts";
import { ExpandedPkgSpec } from "../../spec/pkgs.ts";
import { bfsPropTree } from "../../spec/props.ts";
import { ProviderConfig } from "../types.ts";

const logger = _logger.ns("assetOverrides").seal();

/**
 * Generic function to apply provider-specific asset and property overrides.
 * This replaces provider-specific assetSpecificOverrides functions.
 */
export function applyAssetOverrides(
  incomingSpecs: ExpandedPkgSpec[],
  providerConfig: ProviderConfig,
): ExpandedPkgSpec[] {
  const newSpecs = [] as ExpandedPkgSpec[];

  // Run overrides on all specs
  for (const spec of incomingSpecs) {
    const variant = spec.schemas[0].variants[0];

    // If there's a schema-level override for this spec, run it
    const schemaOverrideFn = providerConfig.overrides.schemaOverrides.get(spec.name);
    if (schemaOverrideFn) {
      logger.debug(`Running schema override for ${spec.name}`);
      schemaOverrideFn(spec);
    }

    // If there are prop-level overrides for this schema+spec, run them
    bfsPropTree([variant.domain, variant.resourceValue], (prop) => {
      const propPath = "/" + prop.metadata.propPath.slice(1).join("/");

      // Check for overrides that match the schema
      for (const [matchSchema, overrides] of Object.entries(providerConfig.overrides.propOverrides)) {
        if (!spec.name.match(new RegExp(`^${matchSchema}$`))) continue;

        // Check for overrides that match the prop
        for (const [matchProp, overrideFns] of Object.entries(overrides)) {
          if (!propPath.match(new RegExp(`^/domain/${matchProp}$`))) continue;

          // Run the matching override
          logger.debug(`Running prop override for ${spec.name} ${propPath}`);
          if (Array.isArray(overrideFns)) {
            for (const overrideFn of overrideFns) overrideFn(prop, spec);
          } else {
            overrideFns(prop, spec);
          }
        }
      }
    });
    newSpecs.push(spec);
  }

  return newSpecs;
}
