import { ExpandedPkgSpec } from "../../../spec/pkgs.ts";
import {
  addPropSuggestSource,
  createObjectProp,
  createScalarProp,
  findPropByName,
} from "../../../spec/props.ts";

export function addDefaultProps(
  specs: ExpandedPkgSpec[],
): ExpandedPkgSpec[] {
  const newSpecs = [] as ExpandedPkgSpec[];

  for (const spec of specs) {
    const [schema] = spec.schemas;
    const [schemaVariant] = schema.variants;
    const { domain } = schemaVariant;

    // Extra prop
    const extraProp = createObjectProp(
      "extra",
      domain.metadata.propPath,
      undefined,
      true,
    );

    // Create Endpoint prop
    {
      const endpointProp = createScalarProp(
        "endpoint",
        "string",
        extraProp.metadata.propPath,
        false,
      );

      endpointProp.data.defaultValue = schema.name;
      endpointProp.data.hidden = true;

      extraProp.entries.push(endpointProp);
    }

    {
      const credProp = createScalarProp(
        "Hetzner::Credential::ApiToken",
        "string",
        extraProp.metadata.propPath,
        true,
      );
      credProp.data.widgetKind = "Secret";
      credProp.data.widgetOptions = [
        {
          label: "secretKind",
          value: "Hetzner::Crendential::ApiToken",
        },
      ];

      if (schemaVariant.secrets.kind !== "object") {
        console.log(
          `Could not generate default props for ${spec.name}: secrets is not object`,
        );
        continue;
      }

      schemaVariant.secrets.entries.push(credProp);
    }

    const variant = spec.schemas[0].variants[0];

    const secretsProp = variant.secrets;
    let credentialProp = findPropByName(
      secretsProp,
      "Hetzner::Credential::ApiToken",
    );
    if (!credentialProp) continue;

    credentialProp = addPropSuggestSource(credentialProp, {
      schema: "Hetzner::Crendential::ApiToken",
      prop: "/secrets/Hetzner::Credential::ApiToken",
    });

    // Finalize
    domain.entries.push(extraProp);
    newSpecs.push(spec);
  }

  return newSpecs;
}
