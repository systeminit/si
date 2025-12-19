import { ExpandedPropSpecFor } from "../../spec/props.ts";
import {
  CfProperty,
  PropertyNormalizationContext,
  PROVIDER_REGISTRY,
  ProviderConfig,
  ProviderFuncSpecs,
  ProviderFunctions,
  SuperSchema,
} from "../types.ts";
import { GCP_PROP_OVERRIDES, GCP_SCHEMA_OVERRIDES } from "./overrides.ts";
import {
  ACTION_FUNC_SPECS,
  CODE_GENERATION_FUNC_SPECS,
  MANAGEMENT_FUNCS,
  QUALIFICATION_FUNC_SPECS,
} from "./funcs.ts";
import { GcpSchema, NormalizedGcpSchema } from "./schema.ts";
import { normalizeGcpProperty } from "./spec.ts";
import { generateGcpSpecs } from "./pipeline.ts";
import { fetchGcpDiscoveryDocuments } from "./discovery.ts";
import { JSONSchema } from "../draft_07.ts";

function createDocLink(
  schema: SuperSchema,
  _defName: string | undefined,
  propName?: string,
): string {
  const gcpSchema = schema as GcpSchema;
  const baseLink = gcpSchema.documentationLink ||
    `https://cloud.google.com/${gcpSchema.service}/docs`;

  if (propName) {
    return `${baseLink}#${propName.toLowerCase()}`;
  }

  return baseLink;
}

function gcpCategory(schema: SuperSchema): string {
  const gcpSchema = schema as GcpSchema;
  // Remove "API" and version suffixes (v1, v2, II, III, etc.) from title
  const cleanTitle = gcpSchema.title
    .replace(/\s+API\s*$/i, "") // Remove trailing "API"
    .replace(/\s+API\s+/gi, " ") // Remove "API" in the middle
    .replace(/\s+v\d+$/i, "") // Remove version numbers like "v1", "v2"
    .replace(/\s+I{1,3}$/i, "") // Remove roman numerals I, II, III
    .trim();
  const category = `Google Cloud ${cleanTitle}`;
  // Remove duplicate "Cloud" words (e.g., "Google Cloud Cloud Storage" -> "Google Cloud Storage")
  return category.replace(/\bCloud\s+Cloud\b/gi, "Cloud");
}

function gcpIsChildRequired(
  schema: SuperSchema | GcpSchema,
  _parentProp: ExpandedPropSpecFor["object" | "array" | "map"] | undefined,
  childName: string,
): boolean {
  if (!("requiredProperties" in schema)) {
    throw new Error("Expected GCP schema with requiredProperties Set");
  }
  return schema.requiredProperties.has(childName);
}

function gcpNormalizeProperty(
  prop: JSONSchema,
  _context: PropertyNormalizationContext,
): CfProperty {
  // JSONSchema can be a boolean, but we only handle objects
  if (typeof prop === "boolean") {
    return { type: prop ? "object" : "never" } as CfProperty;
  }

  let propToNormalize = prop;
  if (
    typeof prop === "object" &&
    "properties" in prop &&
    prop.properties &&
    !prop.type
  ) {
    propToNormalize = { ...prop, type: "object" } as CfProperty;
  }

  return normalizeGcpProperty(
    propToNormalize as NormalizedGcpSchema,
  ) as CfProperty;
}

const gcpProviderFunctions: ProviderFunctions = {
  createDocLink,
  getCategory: gcpCategory,
};

const gcpProviderFuncSpecs: ProviderFuncSpecs = {
  actions: ACTION_FUNC_SPECS,
  codeGeneration: CODE_GENERATION_FUNC_SPECS,
  management: MANAGEMENT_FUNCS,
  qualification: QUALIFICATION_FUNC_SPECS,
};

export const gcpProviderConfig: ProviderConfig = {
  name: "google cloud",
  isStable: true,
  functions: gcpProviderFunctions,
  funcSpecs: gcpProviderFuncSpecs,
  loadSchemas: generateGcpSpecs,
  fetchSchema: fetchGcpDiscoveryDocuments,
  metadata: {
    color: "#EF6255",
    displayName: "Google Cloud",
    description: "Google Cloud Platform infrastructure resources",
  },
  normalizeProperty: gcpNormalizeProperty,
  isChildRequired: gcpIsChildRequired,
  overrides: {
    propOverrides: GCP_PROP_OVERRIDES,
    schemaOverrides: GCP_SCHEMA_OVERRIDES,
  },
};

// Register this provider
PROVIDER_REGISTRY[gcpProviderConfig.name] = gcpProviderConfig;
