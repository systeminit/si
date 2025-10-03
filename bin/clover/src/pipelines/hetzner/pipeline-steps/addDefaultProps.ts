import _ from "lodash";
import { ExpandedPkgSpec } from "../../../spec/pkgs.ts";
import {
  addPropSuggestSource,
  createObjectProp,
  createScalarProp,
  ExpandedPropSpec,
  findPropByName,
} from "../../../spec/props.ts";
import { HetznerSchema } from "../schema.ts";

export interface PropUsageMap {
  createOnly: string[];
  updatable: string[];
}

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

    extraProp.data.hidden = true;

    // Create Endpoint prop
    {
      const endpointProp = createScalarProp(
        "endpoint",
        "string",
        extraProp.metadata.propPath,
        false,
      );

      // Get endpoint from the HetznerSchema stored in the variant's superSchema
      const hSchema = schemaVariant.superSchema as HetznerSchema;
      endpointProp.data.defaultValue = hSchema?.endpoint || schema.name;

      extraProp.entries.push(endpointProp);
    }

    // Create HetznerResourceType prop
    {
      const resourceTypeProp = createScalarProp(
        "HetznerResourceType",
        "string",
        extraProp.metadata.propPath,
        false,
      );

      resourceTypeProp.data.defaultValue = schema.name;
      resourceTypeProp.data.hidden = true;

      extraProp.entries.push(resourceTypeProp);
    }

    // Create PropUsageMap
    {
      const propUsageMapProp = createScalarProp(
        "PropUsageMap",
        "string",
        extraProp.metadata.propPath,
        false,
      );
      const propUsageMap: PropUsageMap = {
        createOnly: [],
        updatable: [],
      };

      const queue: ExpandedPropSpec[] = _.cloneDeep(domain.entries);

      while (queue.length > 0) {
        const prop = queue.pop();
        if (!prop) break;

        if (prop.metadata.createOnly) {
          propUsageMap.createOnly.push(prop.name);
        } else if (!prop.metadata.readOnly) {
          propUsageMap.updatable.push(prop.name);
        }

        if (prop.kind === "object") {
          prop.entries.forEach((p) => queue.unshift(p));
        }
      }

      propUsageMapProp.data.defaultValue = JSON.stringify(propUsageMap);
      propUsageMapProp.data.hidden = true;
      extraProp.entries.push(propUsageMapProp);
    }

    {
      const credProp = createScalarProp(
        "Hetzner Api Token",
        "string",
        extraProp.metadata.propPath,
        true,
      );
      credProp.data.widgetKind = "Secret";
      credProp.data.widgetOptions = [
        {
          label: "secretKind",
          value: "Hetzner::Credential::ApiToken",
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
      "Hetzner Api Token",
    );
    if (!credentialProp) continue;

    credentialProp = addPropSuggestSource(credentialProp, {
      schema: "Hetzner::Credential::ApiToken",
      prop: "/secrets/Hetzner::Credential::ApiToken",
    });

    // Finalize
    domain.entries.push(extraProp);
    newSpecs.push(spec);
  }

  return newSpecs;
}
