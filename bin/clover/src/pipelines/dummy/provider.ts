import { OnlyProperties } from "../../spec/props.ts";
import {
  CfProperty,
  PipelineOptions,
  PropertyNormalizationContext,
  PROVIDER_REGISTRY,
  ProviderConfig,
  ProviderFuncSpecs,
  ProviderFunctions,
  SuperSchema,
} from "../types.ts";
import {
  ACTION_FUNC_SPECS,
  CODE_GENERATION_FUNC_SPECS,
  MANAGEMENT_FUNCS,
  QUALIFICATION_FUNC_SPECS,
} from "./funcs.ts";
import { databaseSchema, serverSchema } from "./schema.ts";
import { ExpandedPropSpecFor } from "../../spec/props.ts";
import { generateDummySpecs } from "./pipeline.ts";
import { DUMMY_PROP_OVERRIDES, DUMMY_SCHEMA_OVERRIDES } from "./overrides.ts";

function createDocLink(
  _schema: SuperSchema,
  _defName: string | undefined,
  _propName?: string,
): string {
  return "https://dummy.example.com/docs";
}

function dummyCategory(schema: SuperSchema): string {
  return schema.typeName;
}

function dummyNormalizeProperty(
  prop: CfProperty,
  _context: PropertyNormalizationContext,
): CfProperty {
  if ("properties" in prop && prop.properties && !prop.type) {
    return { ...prop, type: "object" } as CfProperty;
  }
  return prop;
}

function dummyIsChildRequired(
  schema: SuperSchema,
  _parentProp: ExpandedPropSpecFor["object" | "array" | "map"] | undefined,
  childName: string,
): boolean {
  if ("requiredProperties" in schema) {
    return schema.requiredProperties.has(childName);
  }
  return false;
}

function dummyParseRawSchema(_rawSchema: unknown): SuperSchema[] {
  return [serverSchema, databaseSchema];
}

function dummyClassifyProperties(_schema: SuperSchema): OnlyProperties {
  return {
    createOnly: [],
    readOnly: ["id", "ipAddress", "status"],
    writeOnly: [],
    primaryIdentifier: ["id"],
  };
}

async function dummyLoadSchemas(
  options: PipelineOptions,
) {
  return await generateDummySpecs(options);
}

const dummyProviderFunctions: ProviderFunctions = {
  createDocLink,
  getCategory: dummyCategory,
};

const dummyProviderFuncSpecs: ProviderFuncSpecs = {
  actions: ACTION_FUNC_SPECS,
  codeGeneration: CODE_GENERATION_FUNC_SPECS,
  management: MANAGEMENT_FUNCS,
  qualification: QUALIFICATION_FUNC_SPECS,
};

export const dummyProviderConfig: ProviderConfig = {
  name: "dummy",
  functions: dummyProviderFunctions,
  funcSpecs: dummyProviderFuncSpecs,
  loadSchemas: dummyLoadSchemas,
  parseRawSchema: dummyParseRawSchema,
  classifyProperties: dummyClassifyProperties,
  metadata: {
    color: "#808080",
    displayName: "Dummy Provider",
    description: "Test provider for development and testing",
  },
  normalizeProperty: dummyNormalizeProperty,
  isChildRequired: dummyIsChildRequired,
  overrides: {
    propOverrides: DUMMY_PROP_OVERRIDES,
    schemaOverrides: DUMMY_SCHEMA_OVERRIDES,
  },
};

PROVIDER_REGISTRY[dummyProviderConfig.name] = dummyProviderConfig;
