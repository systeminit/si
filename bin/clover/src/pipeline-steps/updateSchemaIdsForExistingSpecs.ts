import _logger from "../logger.ts";
import { PkgSpec } from "../bindings/PkgSpec.ts";
import _ from "npm:lodash";
const logger = _logger.ns("updateExisting").seal();

export function updateSchemaIdsForExistingSpecs(
  existing_specs: Record<string, string>,
  specs: PkgSpec[],
): PkgSpec[] {
  const newSpecs = [] as PkgSpec[];

  for (const spec of specs) {
    const schema = spec.schemas[0];

    if (!schema) {
      console.log(
        `Could not generate default props and sockets for ${spec.name}: missing schema`,
      );
      continue;
    }

    const schema_id = existing_specs[spec.name];
    if (schema_id) {
      logger.debug(`Found existing spec ${spec.name}, updating schema id`);
      spec.schemas[0].uniqueId = schema_id;
    }
    newSpecs.push(spec);
  }

  return newSpecs;
}
