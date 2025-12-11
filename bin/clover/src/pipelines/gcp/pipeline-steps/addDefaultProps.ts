import { ExpandedPkgSpec } from "../../../spec/pkgs.ts";
import {
  addPropSuggestSource,
  createObjectProp,
  createScalarProp,
} from "../../../spec/props.ts";

export function addDefaultProps(specs: ExpandedPkgSpec[]): ExpandedPkgSpec[] {
  const gcpSpecs = [] as ExpandedPkgSpec[];

  for (const spec of specs) {
    const [schema] = spec.schemas;
    const [schemaVariant] = schema.variants;
    const { domain } = schemaVariant;

    // Extra prop (for future metadata if needed)
    const extraProp = createObjectProp(
      "extra",
      domain.metadata.propPath,
      undefined,
      true,
    );

    extraProp.data.hidden = true;

    // Add Google Cloud Credential to secrets
    {
      const credProp = createScalarProp(
        "Google Cloud Credential",
        "string",
        extraProp.metadata.propPath,
        true,
      );
      credProp.data.widgetKind = "Secret";
      credProp.data.widgetOptions = [
        {
          label: "secretKind",
          value: "Google Cloud Credential",
        },
      ];

      if (schemaVariant.secrets.kind !== "object") {
        console.log(
          `Could not generate default props for ${spec.name}: secrets is not object`,
        );
        continue;
      }

      addPropSuggestSource(credProp, {
        schema: "Google Cloud Credential",
        prop: "/secrets/Google Cloud Credential",
      });

      schemaVariant.secrets.entries.push(credProp);
    }

    domain.entries.push(extraProp);
    gcpSpecs.push(spec);
  }

  return gcpSpecs;
}
