import { ExpandedPkgSpec } from "../../../spec/pkgs.ts";
import {
  addPropSuggestSource,
  ExpandedPropSpecFor,
  findPropByName,
} from "../../../spec/props.ts";

// We want to ensure that the first suggestion for any region
// prop in the generated assets have a suggestion of a Region schema
// and prop /domain/region
// This ensures that we can keep things easy to compose using the new
// suggestions format
export function createRegionSuggestion(
  specs: ExpandedPkgSpec[],
): ExpandedPkgSpec[] {
  for (const spec of specs) {
    const variant = spec.schemas[0].variants[0];

    const extraProp = findPropByName(variant.domain, "extra");
    if (!extraProp) continue;

    let regionProp = findPropByName(
      extraProp as ExpandedPropSpecFor["object"],
      "Region",
    );
    if (!regionProp) continue;

    regionProp = addPropSuggestSource(regionProp, {
      schema: "Region",
      prop: "/domain/region",
    });
  }

  return specs;
}

// We want to ensure that the first suggestion for any credentials
// prop in the generated assets have a suggestion of an AWS Credential schema
// and prop /secrets/AWS Credential
// This ensures that we can keep things easy to compose using the new
// suggestions format
export function createCredentialSuggestion(
  specs: ExpandedPkgSpec[],
): ExpandedPkgSpec[] {
  for (const spec of specs) {
    const variant = spec.schemas[0].variants[0];

    const secretsProp = variant.secrets;
    let credentialProp = findPropByName(secretsProp, "AWS Credential");
    if (!credentialProp) continue;

    credentialProp = addPropSuggestSource(credentialProp, {
      schema: "AWS Credential",
      prop: "/secrets/AWS Credential",
    });
  }

  return specs;
}
