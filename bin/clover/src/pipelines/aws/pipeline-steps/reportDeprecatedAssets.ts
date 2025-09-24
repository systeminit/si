import _logger from "../../../logger.ts";
import _ from "npm:lodash";
import { ExpandedPkgSpec } from "../../../spec/pkgs.ts";
const logger = _logger.ns("reportDeprecated").seal();

export function reportDeprecatedAssets(
  existing_specs: Record<string, string>,
  specs: ExpandedPkgSpec[],
): ExpandedPkgSpec[] {
  const deprecatedSpecs = _.clone(existing_specs);

  for (const spec of specs) {
    if (deprecatedSpecs[spec.name]) {
      logger.debug(`Found existing spec ${spec.name}`);
      delete deprecatedSpecs[spec.name];
    }
  }

  const deprecatedSpecNames = _.keys(deprecatedSpecs);
  if (deprecatedSpecNames.length) {
    logger.warn(
      `${deprecatedSpecNames.length} asset(s) haven't been generated but are on module index. They've been saved to existing-packages/deprecatedSpecs.json`,
    );
    Deno.writeTextFileSync(
      "existing-packages/deprecatedSpecs.json",
      JSON.stringify(deprecatedSpecs, null, 2),
    );
  }

  return specs;
}
