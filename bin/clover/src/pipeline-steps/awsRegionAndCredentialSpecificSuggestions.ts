import { ExpandedPkgSpec } from "../spec/pkgs.ts";
import {
  addPropSuggestSource,
  ExpandedPropSpecFor,
  findPropByName,
} from "../spec/props.ts";

// We want to ensure that the first suggestion for any region
// prop in the generated assets have a suggestion of a Region schema
// and prop /domain/region
// This ensures that we can keep things easy to compose using the new
// suggestions format
export function createRegionSuggestion(
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
      prop: "/domain/region",
    });

    newSpecs.push(spec);
  }

  return newSpecs;
}

// We want to ensure that the first suggestion for any credentials
// prop in the generated assets have a suggestion of an AWS Credential schema
// and prop /secrets/AWS Credential
// This ensures that we can keep things easy to compose using the new
// suggestions format
export function createCredentialSuggestion(
  specs: ExpandedPkgSpec[],
): ExpandedPkgSpec[] {
  const newSpecs = [] as ExpandedPkgSpec[];
  for (const spec of specs) {
    const [schema] = spec.schemas;
    const [schemaVariant] = schema.variants;

    const secretsProp = schemaVariant.secrets;
    let credentialProp = findPropByName(secretsProp, "AWS Credential");
    if (!credentialProp) {
      newSpecs.push(spec);
      continue;
    }

    credentialProp = addPropSuggestSource(credentialProp, {
      schema: "AWS Credential",
      prop: "/secrets/AWS Credential",
    });

    newSpecs.push(spec);
  }

  return newSpecs;
}
