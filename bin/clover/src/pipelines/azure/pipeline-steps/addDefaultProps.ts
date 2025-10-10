import _ from "lodash";
import { ExpandedPkgSpec } from "../../../spec/pkgs.ts";
import {
  addPropSuggestSource,
  createObjectProp,
  createScalarProp,
  ExpandedPropSpec,
  findPropByName,
} from "../../../spec/props.ts";
import { AzureSchema } from "../schema.ts";

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

    // add name prop if not exists
    if (!findPropByName(domain, "name")) {
      const nameProp = createScalarProp(
        "name",
        "string",
        domain.metadata.propPath,
        true,
      );

      nameProp.data.documentation = "The name of the Azure resource";
      nameProp.metadata.createOnly = true;

      domain.entries.push(nameProp);
    } else {
      // Mark existing name prop as createOnly
      const nameProp = findPropByName(domain, "name");
      if (nameProp) {
        nameProp.metadata.createOnly = true;
      }
    }

    // add resource group prop if not exists
    if (!findPropByName(domain, "resourceGroup")) {
      const resourceGroupProp = createScalarProp(
        "resourceGroup",
        "string",
        domain.metadata.propPath,
        false,
      );

      resourceGroupProp.data.documentation =
        "The name of the resource group where this resource will be created";
      resourceGroupProp.data.docLink =
        "https://learn.microsoft.com/en-us/azure/azure-resource-manager/management/manage-resource-groups-portal";
      resourceGroupProp.metadata.createOnly = true;

      addPropSuggestSource(resourceGroupProp, {
        schema: "Azure Resource Group",
        prop: "/domain/Name",
      });

      domain.entries.push(resourceGroupProp);
    }

    // add subscription id prop if not exists
    if (!findPropByName(domain, "subscriptionId")) {
      const subscriptionIdProp = createScalarProp(
        "subscriptionId",
        "string",
        domain.metadata.propPath,
        true,
      );

      subscriptionIdProp.data.documentation =
        "The Azure subscription ID where this resource will be created";
      subscriptionIdProp.data.docLink =
        "https://learn.microsoft.com/en-us/azure/azure-resource-manager/management/manage-subscription";
      subscriptionIdProp.metadata.createOnly = true;

      addPropSuggestSource(subscriptionIdProp, {
        schema: "Azure Subscription",
        prop: "/domain/SubscriptionId",
      });

      domain.entries.push(subscriptionIdProp);
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
