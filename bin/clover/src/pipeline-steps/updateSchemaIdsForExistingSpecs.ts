import _logger from "../logger.ts";
import _ from "npm:lodash";
import { ExpandedPkgSpec } from "../spec/pkgs.ts";
const logger = _logger.ns("updateExisting").seal();

export function updateSchemaIdsForExistingSpecs(
  existing_specs: Record<string, string>,
  specs: readonly ExpandedPkgSpec[],
) {
  for (const spec of specs) {
    const schema_id = existing_specs[spec.name];
    if (schema_id) {
      logger.debug(`Found existing spec ${spec.name}, updating schema id`);
      spec.schemas[0].uniqueId = schema_id;
    }
  }
}
