import _logger from "../logger.ts";
import _ from "npm:lodash";
import { ExpandedPkgSpec } from "../spec/pkgs.ts";

// This exists so the frontend feature flag can ignore clover generated modules
// It should be removed when clover assets are launched
export function addSignatureToCategoryName(
  specs: ExpandedPkgSpec[],
): ExpandedPkgSpec[] {
  const newSpecs = [] as ExpandedPkgSpec[];

  for (const spec of specs) {
    const [schema] = spec.schemas;

    const schemaData = schema.data;
    if (schemaData) {
      schemaData.category = `Clover: ${schemaData.category}`;
    }
    newSpecs.push(spec);
  }

  return newSpecs;
}
