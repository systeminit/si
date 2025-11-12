import assert from "node:assert";
import {
  FetchSchemaOptions,
  PropOverrideFn,
  PROVIDER_REGISTRY,
  ProviderConfig,
  SchemaOverrideFn,
  SuperSchema,
} from "../types.ts";
import { ExpandedPropSpecFor } from "../../spec/props.ts";
import {
  ACTION_FUNC_SPECS,
  CODE_GENERATION_FUNC_SPECS,
  MANAGEMENT_FUNCS,
  QUALIFICATION_FUNC_SPECS,
} from "./funcs.ts";
import { generateAzureSpecs } from "./pipeline.ts";
import {
  AzureProperty,
  AzureSchema,
  initAzureRestApiSpecsRepo,
} from "./schema.ts";
import { JSONSchema } from "../draft_07.ts";
import { fixNames, suggest } from "../generic/overrides.ts";

async function azureFetchSchema(options: FetchSchemaOptions) {
  const specsRepo = initAzureRestApiSpecsRepo(options);
  console.log(`Updating Azure specs in ${specsRepo} ...`);

  // Update the bin/clover/src/provider-schemas/azure-rest-api-specs submodule
  const command = new Deno.Command("git", {
    args: ["submodule", "update", "--init", "--recursive"],
  });

  const { code, stderr } = await command.output();

  if (code !== 0) {
    const errorText = new TextDecoder().decode(stderr);
    throw new Error(`Failed to update Azure specs: ${errorText}`);
  }

  console.log("Update complete");
}

function createDocLink(
  { typeName: resourceType }: SuperSchema,
  _defName: string | undefined,
  _propName?: string,
): string {
  return `https://learn.microsoft.com/en-us/rest/api/${resourceType}`;
}

function azureCategory(schema: SuperSchema): string {
  return schema.typeName.split("/", 1)[0];
}

function azureIsChildRequired(
  schema: SuperSchema | AzureSchema,
  _parentProp: ExpandedPropSpecFor["object" | "array" | "map"] | undefined,
  childName: string,
): boolean {
  if (!("requiredProperties" in schema)) {
    throw new Error("Expected Azure schema with requiredProperties Set");
  }
  return schema.requiredProperties.has(childName);
}

// NOTE(nick,jkeiser): here is an example of what overrides look like...
const AZURE_PROP_OVERRIDES: Record<
  string,
  Record<string, PropOverrideFn | PropOverrideFn[]>
> = {
  // "Microsoft.Network/loadBalancers": {
  //   "properties/frontendIPConfigurations/frontendIPConfigurationsItem/properties/publicIPAddress/id": suggest("Microsoft.Network/publicIPAddresses", "id"),
  // }
};

const AZURE_SCHEMA_OVERRIDES: ProviderConfig["overrides"]["schemaOverrides"] =
  new Map([
    [
      "Microsoft.Aad/domainServices/ouContainer",
      fixNames({
        categoryName: "Microsoft.AAD",
        schemaName: "Microsoft.AAD/domainServices/ouContainer",
      }),
    ],
    [
      "microsoft.insights/guestDiagnosticSettings",
      fixNames({
        categoryName: "Microsoft.Insights",
        schemaName: "Microsoft.Insights/guestDiagnosticSettings",
      }),
    ],
    [
      "microsoft.insights/components/linkedStorageAccounts",
      fixNames({
        categoryName: "Microsoft.Insights",
        schemaName: "Microsoft.Insights/components/linkedStorageAccounts",
      }),
    ],
    [
      "microsoft.alertsManagement/smartDetectorAlertRules",
      fixNames({
        categoryName: "Microsoft.AlertsManagement",
        schemaName: "Microsoft.AlertsManagement/smartDetectorAlertRules",
      }),
    ],
    [
      "Microsoft.DBForMySql/flexibleServers/keys",
      fixNames({
        categoryName: "Microsoft.DBforMySQL",
        schemaName: "Microsoft.DBforMySQL/flexibleServers/keys",
      }),
    ],
    [
      "Microsoft.DBForPostgreSql/flexibleServers/keys",
      fixNames({
        categoryName: "Microsoft.DBforPostgreSQL",
        schemaName: "Microsoft.DBforPostgreSQL/flexibleServers/keys",
      })
    ],
  ]);

export const AZURE_PROVIDER_CONFIG: ProviderConfig = {
  name: "azure",
  isStable: true,
  fetchSchema: azureFetchSchema,
  functions: {
    createDocLink,
    getCategory: azureCategory,
  },
  funcSpecs: {
    actions: ACTION_FUNC_SPECS,
    codeGeneration: CODE_GENERATION_FUNC_SPECS,
    management: MANAGEMENT_FUNCS,
    qualification: QUALIFICATION_FUNC_SPECS,
  },
  loadSchemas: generateAzureSpecs,
  // These are normalized earlier in the process
  normalizeProperty: (prop: JSONSchema) => prop as AzureProperty,
  isChildRequired: azureIsChildRequired,
  overrides: {
    propOverrides: AZURE_PROP_OVERRIDES,
    schemaOverrides: AZURE_SCHEMA_OVERRIDES,
  },
  metadata: {
    color: "#0078D4",
    displayName: "Microsoft Azure",
    description: "Microsoft Azure cloud resources",
  },
};

PROVIDER_REGISTRY[AZURE_PROVIDER_CONFIG.name] = AZURE_PROVIDER_CONFIG;
