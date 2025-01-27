import _logger from "../logger.ts";
import { PkgSpec } from "../bindings/PkgSpec.ts";
import _ from "npm:lodash";

export function updateSchemaIdsForExistingSpecs(
  existing_specs: Record<string, PkgSpec>,
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

    const existing_spec = existing_specs[spec.name];
    if (existing_spec) {
      spec.schemas[0].uniqueId = existing_spec.schemas[0].uniqueId;
    }
    newSpecs.push(spec);
  }

  return newSpecs;
}
