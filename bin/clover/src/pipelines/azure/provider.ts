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

async function azureFetchSchema() {
  let repoPath: string | null = null;

  try {
    repoPath = await cloneAzureSpecs();

    const swaggerFiles = await discoverSwaggerFiles(repoPath);
    console.log(`Discovered ${swaggerFiles.length} swagger files`);

    const swaggers = await processSwaggerFiles(swaggerFiles);
    const serviceSpecs = await consolidateSpecsByService(swaggers);

    // Write each service to its own file
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

export const azureProviderConfig: ProviderConfig = {
  name: "azure",
  fetchSchema: azureFetchSchema,

  // Stub implementations - will be replaced with full implementation later
  functions: {
    createDocLink: () => "https://learn.microsoft.com/en-us/azure",
    getCategory: (schema) => schema.typeName,
  },

  funcSpecs: {
    actions: {},
    codeGeneration: {},
    management: {},
    qualification: {},
  },

  loadSchemas: async (_options: PipelineOptions) => {
    console.warn(
      "Azure spec generation not yet implemented - skipping Azure provider",
    );
    return [];
  },

  normalizeProperty: (
    prop: CfProperty,
    _context: PropertyNormalizationContext,
  ) => prop,

  isChildRequired: (
    _schema: SuperSchema,
    _parentProp: ExpandedPropSpecFor["object" | "array" | "map"] | undefined,
    _childName: string,
  ) => false,

  classifyProperties: (_schema: SuperSchema): OnlyProperties => ({
    createOnly: [],
    readOnly: [],
    writeOnly: [],
    primaryIdentifier: ["id"],
  }),

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

PROVIDER_REGISTRY[azureProviderConfig.name] = azureProviderConfig;
