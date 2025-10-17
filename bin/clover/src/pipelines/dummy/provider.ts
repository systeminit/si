import { OnlyProperties } from "../../spec/props.ts";
import { ExpandedPkgSpec } from "../../spec/pkgs.ts";
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
import { databaseSchema, DummySchema, serverSchema } from "./schema.ts";
import { ExpandedPropSpecFor } from "../../spec/props.ts";
import { generateDummySpecs } from "./pipeline.ts";
import { DUMMY_PROP_OVERRIDES, DUMMY_SCHEMA_OVERRIDES } from "./overrides.ts";
import { makeModule } from "../generic/index.ts";
import { JSONSchema } from "../draft_07.ts";

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
  prop: JSONSchema,
  _context: PropertyNormalizationContext,
): CfProperty {
  if (
    typeof prop === "object" &&
    "properties" in prop &&
    prop.properties &&
    !prop.type
  ) {
    return { ...prop, type: "object" } as CfProperty;
  }
  return prop as CfProperty;
}

function dummyIsChildRequired(
  schema: SuperSchema | DummySchema,
  _parentProp: ExpandedPropSpecFor["object" | "array" | "map"] | undefined,
  childName: string,
): boolean {
  if ("requiredProperties" in schema) {
    return schema.requiredProperties.has(childName);
  }
  return false;
}

export function dummyParseRawSchema(_rawSchema: unknown): ExpandedPkgSpec[] {
  const schemas = [serverSchema, databaseSchema];
  const specs: ExpandedPkgSpec[] = [];

  for (const schema of schemas) {
    const onlyProperties: OnlyProperties = {
      createOnly: [],
      readOnly: ["id", "ipAddress", "status"],
      writeOnly: [],
      primaryIdentifier: ["id"],
    };

    const readOnlySet = new Set(onlyProperties.readOnly);
    const domainProperties: Record<string, CfProperty> = {};
    const resourceValueProperties: Record<string, CfProperty> = {};

    for (const [name, prop] of Object.entries(schema.properties)) {
      if (readOnlySet.has(name)) {
        resourceValueProperties[name] = prop as CfProperty;
      } else {
        domainProperties[name] = prop as CfProperty;
      }
    }

    const spec = makeModule(
      schema,
      schema.description,
      onlyProperties,
      DUMMY_PROVIDER_CONFIG,
      domainProperties,
      resourceValueProperties,
    );

    specs.push(spec);
  }

  return specs;
}

async function dummyLoadSchemas(options: PipelineOptions) {
  return await generateDummySpecs(options);
}

function dummyFetchSchema(): Promise<void> {
  console.log("Dummy provider uses hardcoded schemas - no fetching needed");
  return Promise.resolve();
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

export const DUMMY_PROVIDER_CONFIG: ProviderConfig = {
  name: "dummy",
  isStable: false,
  functions: dummyProviderFunctions,
  funcSpecs: dummyProviderFuncSpecs,
  loadSchemas: dummyLoadSchemas,
  fetchSchema: dummyFetchSchema,
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

PROVIDER_REGISTRY[DUMMY_PROVIDER_CONFIG.name] = DUMMY_PROVIDER_CONFIG;
