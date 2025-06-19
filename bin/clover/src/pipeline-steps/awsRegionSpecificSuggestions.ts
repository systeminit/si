import { ExpandedPkgSpec } from "../spec/pkgs.ts";
import { addPropSuggestSource, ExpandedPropSpecFor, findPropByName } from "../spec/props.ts";

// We want to ensure that the first suggestion for any region
// prop in the generated assets have a suggestion of a Region schema
// and prop /domain/region
// This ensures that we can keep things easy to compose using the new
// suggestions format
export function createAwsRegionSpecificSuggestion(
  specs: ExpandedPkgSpec[],
): ExpandedPkgSpec[] {
  const newSpecs = [] as ExpandedPkgSpec[];
  for (const spec of specs) {
    const [schema] = spec.schemas;
    const [schemaVariant] = schema.variants;

    const domainProp = schemaVariant.domain;
    const extraProp = findPropByName(domainProp, "extra");
    if (!extraProp) {
      newSpecs.push(spec);
      continue;
    }

    let regionProp = findPropByName(
      extraProp as ExpandedPropSpecFor["object"],
      "Region",
    );
    if (!regionProp) {
      newSpecs.push(spec);
      continue;
    }
    
    regionProp = addPropSuggestSource(regionProp, {
      schema: "Region",
      prop: "/domain/region"
    })
    
    newSpecs.push(spec)
  }

  return newSpecs;
}
