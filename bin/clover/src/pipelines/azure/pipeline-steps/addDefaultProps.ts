import _ from "lodash";
import { ExpandedPkgSpec } from "../../../spec/pkgs.ts";
import {
  addPropSuggestSource,
  bfsPropTree,
  createObjectProp,
  createScalarProp,
  ExpandedPropSpec,
  findPropByName,
} from "../../../spec/props.ts";
import { AzureSchema } from "../schema.ts";

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

    // Add suggestions to resourceGroupName
    // TODO if resourceGroupName prop exists with a different name, support that
    const resourceGroupNameProp = findPropByName(domain, "resourceGroupName");
    if (resourceGroupNameProp) {
      resourceGroupNameProp.data.documentation ??=
        "The name of the resource group where this resource will be created";
      resourceGroupNameProp.data.docLink ??=
        "https://learn.microsoft.com/en-us/azure/azure-resource-manager/management/manage-resource-groups-portal";
      resourceGroupNameProp.metadata.createOnly = true;

      addPropSuggestSource(resourceGroupNameProp, {
        schema: "Azure Resource Group",
        prop: "/domain/Name",
      });
    }

    // Add suggestions to subscriptionId
    const subscriptionIdProp = findPropByName(domain, "subscriptionId");
    if (subscriptionIdProp) {
      subscriptionIdProp.data.documentation ??=
        "The Azure subscription ID where this resource will be created";
      subscriptionIdProp.data.docLink ??=
        "https://learn.microsoft.com/en-us/azure/azure-resource-manager/management/manage-subscription";
      subscriptionIdProp.metadata.createOnly = true;

      addPropSuggestSource(subscriptionIdProp, {
        schema: "Azure Subscription",
        prop: "/domain/SubscriptionId",
      });
    }

    // Extra prop
    const extraProp = createObjectProp(
      "extra",
      domain.metadata.propPath,
      undefined,
      true,
    );

    extraProp.data.hidden = true;

    // Create AzureResourceType prop
    // TODO remove this; kept because a few functions reference it right now
    {
      const resourceTypeProp = createScalarProp(
        "AzureResourceType",
        "string",
        extraProp.metadata.propPath,
        false,
      );

      resourceTypeProp.data.defaultValue = schema.name;
      resourceTypeProp.data.hidden = true;

      extraProp.entries.push(resourceTypeProp);
    }

    // Create AzureResourceType prop
    {
      const resourceIdProp = createScalarProp(
        "resourceId",
        "string",
        extraProp.metadata.propPath,
        false,
      );

      const { resourceId } = schemaVariant.superSchema as AzureSchema;
      resourceIdProp.data.defaultValue = resourceId;
      resourceIdProp.data.hidden = true;

      extraProp.entries.push(resourceIdProp);
    }

    // Create apiVersion prop
    {
      const apiVersionProp = createScalarProp(
        "apiVersion",
        "string",
        extraProp.metadata.propPath,
        false,
      );

      const azureSchema = schemaVariant.superSchema as AzureSchema;
      apiVersionProp.data.defaultValue = azureSchema?.apiVersion ||
        "2023-01-01";

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

      const azureSchema = schemaVariant.superSchema as AzureSchema;
      propUsageMap.discriminators = azureSchema.discriminators;

      // Derive discriminatorSubtypeProps by walking the domain schema structure
      const discriminatorSubtypePropsMap: Record<
        string,
        Record<string, string[]>
      > = {};

      if (azureSchema.discriminators) {
        for (
          const [discriminatorProp, subtypeMap] of Object.entries(
            azureSchema.discriminators,
          )
        ) {
          const domainProp = findPropByName(domain, discriminatorProp);

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

    // Add Azure credential to secrets
    {
      const credProp = createScalarProp(
        "Azure Credential",
        "string",
        extraProp.metadata.propPath,
        true,
      );
      credProp.data.widgetKind = "Secret";
      credProp.data.widgetOptions = [
        {
          label: "secretKind",
          value: "Azure Credential",
        },
      ];

      if (schemaVariant.secrets.kind !== "object") {
        console.log(
          `Could not generate default props for ${spec.name}: secrets is not object`,
        );
        continue;
      }

      addPropSuggestSource(credProp, {
        schema: "Azure Credential",
        prop: "/secrets/Azure Credential",
      });

      schemaVariant.secrets.entries.push(credProp);
    }

    // location prop suggest source
    {
      const locationProp = findPropByName(domain, "location");
      if (locationProp) {
        addPropSuggestSource(locationProp, {
          schema: "Azure Location",
          prop: "/domain/location",
        });
      }
    }

    domain.entries.push(extraProp);
    newSpecs.push(spec);
  }

  return newSpecs;
}
