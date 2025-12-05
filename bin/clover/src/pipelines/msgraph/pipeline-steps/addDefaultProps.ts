import _ from "lodash";
import { ExpandedPkgSpec } from "../../../spec/pkgs.ts";
import {
  addPropSuggestSource,
  bfsPropTree,
  createObjectProp,
  createScalarProp,
  ExpandedPropSpec,
  toPropPath,
} from "../../../spec/props.ts";
import { EntraSchema } from "../schema.ts";

export interface PropUsageMap {
  createOnly: string[];
  updatable: string[];
  // Maps discriminator property -> subtype definition name -> enum value
  // e.g., { "kind": { "AzurePowerShellScript": "AzurePowerShell", "AzureCliScript": "AzureCLI" } }
  discriminators: Record<string, Record<string, string>> | undefined;
  // Maps discriminator property -> subtype definition name -> list of properties defined by that subtype
  // e.g., { "kind": { "AzurePowerShellScript": ["properties"], "AzureCliScript": ["properties"] } }
  discriminatorSubtypeProps:
    | Record<string, Record<string, string[]>>
    | undefined;
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

    // Create EntraResourceType prop (for reference)
    {
      const resourceTypeProp = createScalarProp(
        "EntraResourceType",
        "string",
        extraProp.metadata.propPath,
        false,
      );

      resourceTypeProp.data.defaultValue = schema.name;
      resourceTypeProp.data.hidden = true;

      extraProp.entries.push(resourceTypeProp);
    }

    // Create endpoint prop - used to construct Graph API URLs
    {
      const endpointProp = createScalarProp(
        "endpoint",
        "string",
        extraProp.metadata.propPath,
        false,
      );

      const entraSchema = schemaVariant.superSchema as EntraSchema;
      endpointProp.data.defaultValue = entraSchema.endpoint;
      endpointProp.data.hidden = true;

      extraProp.entries.push(endpointProp);
    }

    // Create apiVersion prop - Graph API version
    {
      const apiVersionProp = createScalarProp(
        "apiVersion",
        "string",
        extraProp.metadata.propPath,
        false,
      );

      // Microsoft Graph API defaults to v1.0
      apiVersionProp.data.defaultValue = "v1.0";

      extraProp.entries.push(apiVersionProp);
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
        discriminators: {},
        discriminatorSubtypeProps: {},
      };

      const entraSchema = schemaVariant.superSchema as EntraSchema;
      propUsageMap.discriminators = entraSchema.discriminators;

      // Derive discriminatorSubtypeProps by walking the domain schema structure
      const discriminatorSubtypePropsMap: Record<
        string,
        Record<string, string[]>
      > = {};

      if (entraSchema.discriminators) {
        for (
          const [discriminatorProp, subtypeMap] of Object.entries(
            entraSchema.discriminators,
          )
        ) {
          const domainProp = domain.entries.find((p) =>
            p.name === discriminatorProp
          );

          if (domainProp && domainProp.kind === "object") {
            discriminatorSubtypePropsMap[discriminatorProp] = {};

            for (const subtypeName of Object.keys(subtypeMap)) {
              const subtypeProp = domainProp.entries.find((e) =>
                e.name === subtypeName
              );

              if (subtypeProp && subtypeProp.kind === "object") {
                const allPropNames: string[] = [];
                bfsPropTree(subtypeProp.entries, (prop) => {
                  allPropNames.push(prop.name);
                });
                discriminatorSubtypePropsMap[discriminatorProp][subtypeName] =
                  allPropNames;
              }
            }
          }
        }
      }

      propUsageMap.discriminatorSubtypeProps = discriminatorSubtypePropsMap;

      const queue: ExpandedPropSpec[] = _.cloneDeep(domain.entries);

      while (queue.length > 0) {
        const prop = queue.pop();
        if (!prop) break;

        const fullPath = toPropPath(prop.metadata.propPath);

        if (prop.metadata.createOnly) {
          propUsageMap.createOnly.push(fullPath);
        } else if (!prop.metadata.readOnly) {
          propUsageMap.updatable.push(fullPath);
        }

        if (prop.kind === "object") {
          prop.entries.forEach((p) => queue.unshift(p));
        }
      }

      propUsageMapProp.data.defaultValue = JSON.stringify(propUsageMap);
      propUsageMapProp.data.hidden = true;
      extraProp.entries.push(propUsageMapProp);
    }

    // Add Microsoft credential to secrets (shared with Azure)
    {
      const credProp = createScalarProp(
        "Microsoft Credential",
        "string",
        extraProp.metadata.propPath,
        true,
      );
      credProp.data.widgetKind = "Secret";
      credProp.data.widgetOptions = [
        {
          label: "secretKind",
          value: "Microsoft Credential",
        },
      ];

      if (schemaVariant.secrets.kind !== "object") {
        console.log(
          `Could not generate default props for ${spec.name}: secrets is not object`,
        );
        continue;
      }

      addPropSuggestSource(credProp, {
        schema: "Microsoft Credential",
        prop: "/secrets/Microsoft Credential",
      });

      schemaVariant.secrets.entries.push(credProp);
    }

    domain.entries.push(extraProp);
    newSpecs.push(spec);
  }

  return newSpecs;
}
