import {
  CfProperty,
  PipelineOptions,
  PropertyNormalizationContext,
  PROVIDER_REGISTRY,
  ProviderConfig,
  SuperSchema,
} from "../types.ts";
import { ExpandedPropSpecFor, OnlyProperties } from "../../spec/props.ts";
import { type JsonSchema } from "./schema.ts";
import {
  cleanupRepo,
  cloneAzureSpecs,
  consolidateSpecsByService,
  discoverSwaggerFiles,
  processSwaggerFiles,
} from "./fetchSchema.ts";
import {
  ACTION_FUNC_SPECS,
  CODE_GENERATION_FUNC_SPECS,
  MANAGEMENT_FUNCS,
  QUALIFICATION_FUNC_SPECS,
} from "./funcs.ts";
import { normalizeAzureProperty, parseAzureSchema } from "./spec.ts";
import { generateAzureSpecs } from "./pipeline.ts";
import { normalizeOnlyProperties } from "../generic/index.ts";

async function azureFetchSchema() {
  let repoPath: string | null = null;

  try {
    repoPath = await cloneAzureSpecs();

    const swaggerFiles = await discoverSwaggerFiles(repoPath);
    console.log(`Discovered ${swaggerFiles.length} swagger files`);

    const swaggers = await processSwaggerFiles(swaggerFiles);
    const serviceSpecs = consolidateSpecsByService(swaggers);

    const schemasDir = "./src/provider-schemas/azure";
    await Deno.mkdir(schemasDir, { recursive: true });

    for (const [serviceName, spec] of serviceSpecs) {
      const filename = `${schemasDir}/${serviceName}.json`;
      await Deno.writeTextFile(filename, JSON.stringify(spec, null, 2));
    }

    console.log(
      `Azure schemas saved to ${schemasDir}/ (${serviceSpecs.size} files)`,
    );
  } finally {
    if (repoPath) {
      await cleanupRepo(repoPath);
    }
  }
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
    propToNormalize = { ...prop, type: "object" } as CfProperty;
  }

  return normalizeAzureProperty(propToNormalize as JsonSchema) as CfProperty;
}

function azureClassifyProperties(schema: SuperSchema): OnlyProperties {
  const inferredOnlyProperties = (schema as any)._inferredOnlyProperties as
    | OnlyProperties
    | undefined;

  if (!inferredOnlyProperties) {
    throw new Error("Expected Azure schema to have _inferredOnlyProperties");
  }

  return {
    createOnly: normalizeOnlyProperties(inferredOnlyProperties.createOnly),
    readOnly: inferredOnlyProperties.readOnly,
    writeOnly: normalizeOnlyProperties(inferredOnlyProperties.writeOnly),
    primaryIdentifier: normalizeOnlyProperties(
      inferredOnlyProperties.primaryIdentifier,
    ),
  };
}

async function azureLoadSchemas(
  options: PipelineOptions,
) {
  return await generateAzureSpecs(options);
}

export const azureProviderConfig: ProviderConfig = {
  name: "azure",
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
  parseRawSchema: parseAzureSchema,
  classifyProperties: azureClassifyProperties,
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

// PROVIDER_REGISTRY[azureProviderConfig.name] = azureProviderConfig;
