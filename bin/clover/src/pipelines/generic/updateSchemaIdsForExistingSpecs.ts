import _logger from "../../logger.ts";
import _ from "lodash";
import { ExpandedPkgSpec } from "../../spec/pkgs.ts";
const logger = _logger.ns("updateExisting").seal();

export function updateSchemaIdsForExistingSpecs(
  existingSpecs: Record<string, string>,
  specs: ExpandedPkgSpec[],
): ExpandedPkgSpec[] {
  const newSpecs = [] as ExpandedPkgSpec[];

  for (const spec of specs) {
    const schema_id = existingSpecs[spec.name];
    if (schema_id) {
      logger.debug(`Found existing spec ${spec.name}, updating schema id`);
      spec.schemas[0].uniqueId = schema_id;
    }
    newSpecs.push(spec);
  }

  return newSpecs;
}
