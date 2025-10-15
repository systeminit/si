import {
  CfProperty,
  FetchSchemaOptions,
  PipelineOptions,
  PropertyNormalizationContext,
  PROVIDER_REGISTRY,
  ProviderConfig,
  SuperSchema,
} from "../types.ts";
import { ExpandedPropSpecFor } from "../../spec/props.ts";
import {
  ACTION_FUNC_SPECS,
  CODE_GENERATION_FUNC_SPECS,
  MANAGEMENT_FUNCS,
  QUALIFICATION_FUNC_SPECS,
} from "./funcs.ts";
import { normalizeAzureProperty } from "./spec.ts";
import { generateAzureSpecs } from "./pipeline.ts";
import { initAzureRestApiSpecsRepo } from "./schema.ts";

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
  { typeName }: SuperSchema,
  _defName: string | undefined,
  _propName?: string,
): string {
  const parts = typeName.split("::");
  const service = parts[1]?.toLowerCase();
  const resourceType = parts[2]?.toLowerCase();

  if (service && resourceType) {
    return `https://learn.microsoft.com/en-us/rest/api/${service}/${resourceType}`;
  }

  return `https://learn.microsoft.com/en-us/azure`;
}

function azureCategory(schema: SuperSchema): string {
  const parts = schema.typeName.split("::");
  if (parts.length >= 2) {
    return `${parts[0]}::${parts[1]}`;
  }
  return schema.typeName;
}

function azureIsChildRequired(
  schema: SuperSchema,
  _parentProp: ExpandedPropSpecFor["object" | "array" | "map"] | undefined,
  childName: string,
): boolean {
  if (!("requiredProperties" in schema)) {
    throw new Error("Expected Azure schema with requiredProperties Set");
  }
  return schema.requiredProperties.has(childName);
}

function azureNormalizeProperty(
  prop: CfProperty,
  _context: PropertyNormalizationContext,
): CfProperty {
  let propToNormalize = prop;
  if ("properties" in prop && prop.properties && !prop.type) {
    propToNormalize = { ...prop, type: "object" };
  }

  return normalizeAzureProperty(propToNormalize);
}

async function azureLoadSchemas(options: PipelineOptions) {
  return await generateAzureSpecs(options);
}

export const AZURE_PROVIDER_CONFIG: ProviderConfig = {
  name: "azure",
  isStable: false,
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
  loadSchemas: azureLoadSchemas,
  normalizeProperty: azureNormalizeProperty,
  isChildRequired: azureIsChildRequired,
  overrides: {
    propOverrides: {},
    schemaOverrides: new Map(),
  },
  metadata: {
    color: "#0078D4",
    displayName: "Microsoft Azure",
    description: "Microsoft Azure cloud resources",
  },
};

PROVIDER_REGISTRY[AZURE_PROVIDER_CONFIG.name] = AZURE_PROVIDER_CONFIG;
