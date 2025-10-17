import { ExpandedPropSpecFor } from "../../spec/props.ts";
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
  HETZNER_PROP_OVERRIDES,
  HETZNER_SCHEMA_OVERRIDES,
} from "./overrides.ts";
import { makeModule } from "../generic/index.ts";
import {
  ACTION_FUNC_SPECS,
  CODE_GENERATION_FUNC_SPECS,
  MANAGEMENT_FUNCS,
  QUALIFICATION_FUNC_SPECS,
} from "./funcs.ts";
import {
  HetznerSchema,
  type JsonSchema,
  type OperationData,
} from "./schema.ts";
import { mergeResourceOperations, normalizeHetznerProperty } from "./spec.ts";
import { generateHetznerSpecs } from "./pipeline.ts";
import { JSONSchema } from "../draft_07.ts";

function createDocLink(
  { typeName }: SuperSchema,
  defName: string | undefined,
  propName?: string,
): string {
  const docLink = "https://docs.hetzner.cloud/reference/cloud";
  const resourceName = typeName.toLowerCase().replace(/_/g, "-");

  if (defName) {
    return `${docLink}#${resourceName}-${defName.toLowerCase()}`;
  }

  if (propName) {
    return `${docLink}#${resourceName}-${propName.toLowerCase()}`;
  }

  return `${docLink}#${resourceName}`;
}

function hCategory(schema: SuperSchema): string {
  const parts = schema.typeName.split("::");
  if (parts.length >= 2) {
    return `${parts[0]}::${parts[1]}`;
  }
  return schema.typeName;
}

function hetznerIsChildRequired(
  schema: SuperSchema | HetznerSchema,
  _parentProp: ExpandedPropSpecFor["object" | "array" | "map"] | undefined,
  childName: string,
): boolean {
  if (!("requiredProperties" in schema)) {
    throw new Error("Expected Hetzner schema with requiredProperties Set");
  }
  return schema.requiredProperties.has(childName);
}

function hetznerNormalizeProperty(
  prop: JSONSchema,
  _context: PropertyNormalizationContext,
): CfProperty {
  let propToNormalize = prop;
  if (
    typeof prop === "object" &&
    "properties" in prop &&
    prop.properties &&
    !prop.type
  ) {
    propToNormalize = { ...prop, type: "object" } as CfProperty;
  }

  return normalizeHetznerProperty(propToNormalize as JsonSchema) as CfProperty;
}

export function hetznerParseRawSchema(rawSchema: unknown): ExpandedPkgSpec[] {
  const allSchemas = rawSchema as JsonSchema;
  const specs: ExpandedPkgSpec[] = [];

  const resourceOperations: Record<string, OperationData[]> = {};
  Object.entries((allSchemas.paths as JsonSchema) || {}).forEach(
    ([endpoint, openApiDescription]) => {
      const noun = endpoint.split("/")[1];

      // Skip action endpoints
      if (endpoint.includes("actions")) return;

      // Skip sub-resource endpoints like /servers/{id}/metrics
      const pathSegments = endpoint.split("/").filter((s) => s);
      if (pathSegments.length > 2) return;

      if (!resourceOperations[noun]) {
        resourceOperations[noun] = [];
      }
      resourceOperations[noun].push({
        endpoint,
        openApiDescription: openApiDescription as JsonSchema,
      });
    },
  );

  Object.entries(resourceOperations).forEach(([noun, operations]) => {
    const result = mergeResourceOperations(noun, operations, allSchemas);
    if (result) {
      const spec = makeModule(
        result.schema,
        result.schema.description,
        result.onlyProperties,
        hetznerProviderConfig,
        result.domainProperties,
        result.resourceValueProperties,
      );
      specs.push(spec);
    }
  });

  return specs;
}

async function hetznerLoadSchemas(options: PipelineOptions) {
  return await generateHetznerSpecs(options);
}

async function hetznerFetchSchema() {
  const url = "https://docs.hetzner.cloud/cloud.spec.json";
  const resp = await fetch(url);
  if (!resp.ok) {
    throw new Error(`Hetzner unreachable at: ${url}`);
  }
  const schema = await resp.json();
  await Deno.writeTextFile(
    "./src/provider-schemas/hetzner.json",
    JSON.stringify(schema, null, 2),
  );
}

const hetznerProviderFunctions: ProviderFunctions = {
  createDocLink,
  getCategory: hCategory,
};

const hetznerProviderFuncSpecs: ProviderFuncSpecs = {
  actions: ACTION_FUNC_SPECS,
  codeGeneration: CODE_GENERATION_FUNC_SPECS,
  management: MANAGEMENT_FUNCS,
  qualification: QUALIFICATION_FUNC_SPECS,
};

export const hetznerProviderConfig: ProviderConfig = {
  name: "hetzner",
  isStable: true,
  functions: hetznerProviderFunctions,
  funcSpecs: hetznerProviderFuncSpecs,
  loadSchemas: hetznerLoadSchemas,
  fetchSchema: hetznerFetchSchema,
  metadata: {
    color: "#D50C2D",
    displayName: "Hetzner Cloud",
    description: "Hetzner Cloud infrastructure resources",
  },
  normalizeProperty: hetznerNormalizeProperty,
  isChildRequired: hetznerIsChildRequired,
  overrides: {
    propOverrides: HETZNER_PROP_OVERRIDES,
    schemaOverrides: HETZNER_SCHEMA_OVERRIDES,
  },
};

// Register this provider
PROVIDER_REGISTRY[hetznerProviderConfig.name] = hetznerProviderConfig;
