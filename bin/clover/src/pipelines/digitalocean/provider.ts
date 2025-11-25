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
import { DIGITALOCEAN_PROP_OVERRIDES, DIGITALOCEAN_SCHEMA_OVERRIDES } from "./overrides.ts";
import { makeModule } from "../generic/index.ts";
import { htmlToMarkdown } from "../../util.ts";
import {
  ACTION_FUNC_SPECS,
  CODE_GENERATION_FUNC_SPECS,
  MANAGEMENT_FUNCS,
  QUALIFICATION_FUNC_SPECS,
} from "./funcs.ts";
import {
  DigitalOceanOpenApiDocument,
  DigitalOceanSchema,
  type JsonSchemaObject,
  type OperationData,
} from "./schema.ts";
import { mergeResourceOperations, normalizeDigitalOceanProperty } from "./spec.ts";
import { generateDigitalOceanSpecs } from "./pipeline.ts";
import { JSONSchema } from "../draft_07.ts";
import SwaggerParser from "@apidevtools/swagger-parser";

function createDocLink(
  { typeName }: SuperSchema,
  defName: string | undefined,
  propName?: string,
): string {
  const docLink = "https://docs.digitalocean.com/reference/api";
  // Extract resource name from DigitalOcean/droplets format
  const resourceName = typeName.split("/")[1] || typeName;

  if (defName) {
    return `${docLink}/${resourceName}#${defName.toLowerCase()}`;
  }

  if (propName) {
    return `${docLink}/${resourceName}#properties`;
  }

  return `${docLink}/${resourceName}`;
}

function digitalOceanCategory(_schema: SuperSchema): string {
  // Return just the provider name for category
  return "DigitalOcean";
}

function digitalOceanIsChildRequired(
  schema: SuperSchema | DigitalOceanSchema,
  _parentProp: ExpandedPropSpecFor["object" | "array" | "map"] | undefined,
  childName: string,
): boolean {
  if (!("requiredProperties" in schema)) {
    throw new Error("Expected DigitalOcean schema with requiredProperties Set");
  }
  return schema.requiredProperties.has(childName);
}

function digitalOceanNormalizeProperty(
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

  return normalizeDigitalOceanProperty(
    propToNormalize as JsonSchemaObject,
  ) as CfProperty;
}

export function digitalOceanParseRawSchema(
  allSchemas: DigitalOceanOpenApiDocument,
): ExpandedPkgSpec[] {
  const specs: ExpandedPkgSpec[] = [];

  const resourceOperations: Record<string, OperationData[]> = {};
  Object.entries(allSchemas.paths || {}).forEach(
    ([endpoint, openApiDescription]) => {
      // Extract resource name from path (e.g., /v2/droplets -> droplets)
      const pathParts = endpoint.split("/").filter((s) =>
        s && !s.startsWith("{")
      );
      if (pathParts.length === 0) return;

      // Skip version prefix
      const noun = pathParts[0] === "v2" ? pathParts[1] : pathParts[0];
      if (!noun) return;

      // TODO: Skip action endpoints and sub-resources for now
      if (pathParts.length > 2) return;

      if (!resourceOperations[noun]) {
        resourceOperations[noun] = [];
      }
      if (openApiDescription) {
        resourceOperations[noun].push({
          endpoint,
          openApiDescription: {
            get: openApiDescription.get,
            post: openApiDescription.post,
            put: openApiDescription.put,
            patch: openApiDescription.patch,
            delete: openApiDescription.delete,
          },
        });
      }
    },
  );

  Object.entries(resourceOperations).forEach(([noun, operations]) => {
    // Extract description from tag
    const firstOp = operations[0]?.openApiDescription;
    const tags = firstOp?.get?.tags || firstOp?.post?.tags;
    const tagName = tags?.[0];
    let description: string | undefined;

    if (tagName && allSchemas.tags) {
      const tag = allSchemas.tags.find((t) => t.name === tagName);
      if (tag?.description) {
        description = tag.description;
      }
    }

    const result = mergeResourceOperations(
      noun,
      operations,
      description,
    );
    if (result) {
      const spec = makeModule(
        result.schema,
        htmlToMarkdown(result.schema.description) ?? result.schema.description,
        result.onlyProperties,
        digitalOceanProviderConfig,
        result.domainProperties,
        result.resourceValueProperties,
      );
      specs.push(spec);
    }
  });

  return specs;
}

async function digitalOceanLoadSchemas(options: PipelineOptions) {
  return await generateDigitalOceanSpecs(options);
}

async function digitalOceanFetchSchema() {
  const url =
    "https://api-engineering.nyc3.cdn.digitaloceanspaces.com/spec-ci/DigitalOcean-public.v2.yaml";
  console.log(`Fetching DigitalOcean OpenAPI spec from ${url}...`);

  // Use SwaggerParser to fetch and parse (but not dereference) the spec
  const spec = await SwaggerParser.parse(url);

  await Deno.writeTextFile(
    "./src/provider-schemas/digitalocean.json",
    JSON.stringify(spec, null, 2),
  );
}

const digitalOceanProviderFunctions: ProviderFunctions = {
  createDocLink,
  getCategory: digitalOceanCategory,
};

const digitalOceanProviderFuncSpecs: ProviderFuncSpecs = {
  actions: ACTION_FUNC_SPECS,
  codeGeneration: CODE_GENERATION_FUNC_SPECS,
  management: MANAGEMENT_FUNCS,
  qualification: QUALIFICATION_FUNC_SPECS,
};

export const digitalOceanProviderConfig: ProviderConfig = {
  name: "digitalocean",
  isStable: false,
  functions: digitalOceanProviderFunctions,
  funcSpecs: digitalOceanProviderFuncSpecs,
  loadSchemas: digitalOceanLoadSchemas,
  fetchSchema: digitalOceanFetchSchema,
  metadata: {
    color: "#0080FF",
    displayName: "DigitalOcean",
    description: "DigitalOcean cloud infrastructure",
  },
  normalizeProperty: digitalOceanNormalizeProperty,
  isChildRequired: digitalOceanIsChildRequired,
  overrides: {
    propOverrides: DIGITALOCEAN_PROP_OVERRIDES,
    schemaOverrides: DIGITALOCEAN_SCHEMA_OVERRIDES,
  },
};

// Register this provider
PROVIDER_REGISTRY[digitalOceanProviderConfig.name] = digitalOceanProviderConfig;
