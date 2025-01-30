import _logger from "../logger.ts";
import { PkgSpec } from "../bindings/PkgSpec.ts";
import _ from "npm:lodash";

// This exists so the frontend feature flag can ignore clover generated modules
// It should be removed when clover assets are launched
export function addSignatureToCategoryName(
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

    const schemaData = schema.data;
    if (schemaData) {
      schemaData.category = `Clover: ${schemaData.category}`;
    }
    newSpecs.push(spec);
  }

  return newSpecs;
}
