import _ from "lodash";
import {
  createObjectProp,
  createScalarProp,
  ExpandedPropSpec,
} from "../spec/props.ts";
import { createInputSocketFromProp } from "../spec/sockets.ts";
import { ExpandedPkgSpecWithSockets } from "../spec/pkgs.ts";

export interface PropUsageMap {
  createOnly: string[];
  updatable: string[];
  secrets: {
    secretKey: string;
    propPath: string[];
  }[];
}

export function addDefaultPropsAndSockets(
  specs: readonly ExpandedPkgSpecWithSockets[],
) {
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
        secrets: [],
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

    // Create Region prop and socket
    {
      const regionProp = createScalarProp(
        "Region",
        "string",
        extraProp.metadata.propPath,
        true,
      );

      schemaVariant.sockets.push(createInputSocketFromProp(regionProp));
      extraProp.entries.push(regionProp);
    }

    // Create AwsResourceType
    {
      const typeProp = createScalarProp(
        "AwsResourceType",
        "string",
        extraProp.metadata.propPath,
        false,
      );

      typeProp.data.defaultValue = schema.name;
      typeProp.data.hidden = true;

      extraProp.entries.push(typeProp);
    }

    // Create Permissions Map
    {
      const permissionsMapProp = createScalarProp(
        "AwsPermissionsMap",
        "string",
        extraProp.metadata.propPath,
        false,
      );

      permissionsMapProp.data.defaultValue = JSON.stringify(
        schemaVariant.cfSchema.handlers,
      );
      permissionsMapProp.data.hidden = true;

      extraProp.entries.push(permissionsMapProp);
    }

    // Create Credential prop and socket under root/secrets
    {
      const credProp = createScalarProp(
        "AWS Credential",
        "string",
        extraProp.metadata.propPath,
        true,
      );
      credProp.data.widgetKind = "Secret";
      credProp.data.widgetOptions = [{
        "label": "secretKind",
        "value": "AWS Credential",
      }];

      schemaVariant.sockets.push(createInputSocketFromProp(credProp));

      if (schemaVariant.secrets.kind !== "object") {
        throw new Error(
          `Could not generate default props and sockets for ${spec.name}: secrets is not object`,
        );
      }

      schemaVariant.secrets.entries.push(credProp);
    }

    // Finalize
    domain.entries.push(extraProp);
  }
}
