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
import { ENTRA_PROP_OVERRIDES, ENTRA_SCHEMA_OVERRIDES } from "./overrides.ts";
import { makeModule } from "../generic/index.ts";
import { htmlToMarkdown } from "../../util.ts";
import {
  ACTION_FUNC_SPECS,
  CODE_GENERATION_FUNC_SPECS,
  MANAGEMENT_FUNCS,
  QUALIFICATION_FUNC_SPECS,
} from "./funcs.ts";
import {
  EntraOpenApiDocument,
  EntraSchema,
  type OperationData,
} from "./schema.ts";
import { mergeResourceOperations } from "./spec.ts";
import { generateEntraSpecs } from "./pipeline.ts";
import { JSONSchema } from "../draft_07.ts";
import SwaggerParser from "@apidevtools/swagger-parser";

function createDocLink(
  { typeName }: SuperSchema,
  defName: string | undefined,
  propName?: string,
): string {
  const docLink = "https://learn.microsoft.com/en-us/graph/api";
  // Extract resource name from Microsoft.Graph/users format
  const resourceName = typeName.split("/")[1] || typeName;

  if (defName) {
    return `${docLink}/${resourceName}#${defName.toLowerCase()}`;
  }

  if (propName) {
    return `${docLink}/${resourceName}#properties`;
  }

  return `${docLink}/${resourceName}`;
}

function entraCategory(_schema: SuperSchema): string {
  // Return just the provider name for category (like Azure does)
  return "Microsoft.Graph";
}

function entraIsChildRequired(
  schema: SuperSchema | EntraSchema,
  _parentProp: ExpandedPropSpecFor["object" | "array" | "map"] | undefined,
  childName: string,
): boolean {
  if (!("requiredProperties" in schema)) {
    throw new Error("Expected Entra schema with requiredProperties Set");
  }
  return schema.requiredProperties.has(childName);
}

function entraNormalizeProperty(
  prop: JSONSchema,
  _context: PropertyNormalizationContext,
): CfProperty {
  if (typeof prop !== "object" || prop === null) {
    return prop as unknown as CfProperty;
  }

  // Microsoft Graph has some properties with no type constraint (they accept any JSON)
  // These show up with description/title but no type field
  // Treat them as "object" since they're unstructured JSON
  if (!prop.type) {
    // If it has properties, it's definitely an object
    if ("properties" in prop && prop.properties) {
      return { ...prop, type: "object" } as CfProperty;
    }

    // If it has a title or description but no type, it's likely unstructured JSON
    // Examples: contentInfo, content, layout, etc.
    if ("title" in prop || ("description" in prop && prop.description)) {
      return { ...prop, type: "object" } as CfProperty;
    }
  }

  return prop as CfProperty;
}

export function entraParseRawSchema(
  allSchemas: EntraOpenApiDocument,
): ExpandedPkgSpec[] {
  const specs: ExpandedPkgSpec[] = [];

  const resourceOperations: Record<string, OperationData[]> = {};
  Object.entries(allSchemas.paths || {}).forEach(
    ([endpoint, openApiDescription]) => {
      // Extract resource name from path (e.g., /users -> users, /users/{id} -> users)
      // Also strip alternate key syntax: /applications(appId='{appId}') -> applications
      const pathParts = endpoint.split("/").filter((s) =>
        s && !s.startsWith("{")
      );
      if (pathParts.length === 0) return;

      // Remove Microsoft Graph alternate key syntax: resourceName(key='{value}') -> resourceName
      const noun = pathParts[0].replace(/\([^)]*\)$/, "");

      // TODO: Skip action endpoints and sub-resources for now
      if (pathParts.length > 1) return;

      if (!resourceOperations[noun]) {
        resourceOperations[noun] = [];
      }
      if (openApiDescription) {
        resourceOperations[noun].push({
          endpoint,
          openApiDescription: {
            get: openApiDescription.get,
            post: openApiDescription.post,
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
      allSchemas,
    );
    if (result) {
      const spec = makeModule(
        result.schema,
        htmlToMarkdown(result.schema.description) ?? result.schema.description,
        result.onlyProperties,
        entraProviderConfig,
        result.domainProperties,
        result.resourceValueProperties,
      );
      specs.push(spec);
    }
  });

  return specs;
}

async function entraLoadSchemas(options: PipelineOptions) {
  return await generateEntraSpecs(options);
}

async function entraFetchSchema() {
  const url =
    "https://raw.githubusercontent.com/microsoftgraph/msgraph-metadata/master/openapi/v1.0/openapi.yaml";
  console.log(`Fetching Microsoft Entra OpenAPI spec from ${url}...`);

  // Use SwaggerParser to fetch and parse (but not dereference) the spec
  const spec = await SwaggerParser.parse(url);

  await Deno.writeTextFile(
    "./src/provider-schemas/entra.json",
    JSON.stringify(spec, null, 2),
  );
}

const entraProviderFunctions: ProviderFunctions = {
  createDocLink,
  getCategory: entraCategory,
};

const entraProviderFuncSpecs: ProviderFuncSpecs = {
  actions: ACTION_FUNC_SPECS,
  codeGeneration: CODE_GENERATION_FUNC_SPECS,
  management: MANAGEMENT_FUNCS,
  qualification: QUALIFICATION_FUNC_SPECS,
};

export const entraProviderConfig: ProviderConfig = {
  name: "entra",
  isStable: true,
  functions: entraProviderFunctions,
  funcSpecs: entraProviderFuncSpecs,
  loadSchemas: entraLoadSchemas,
  fetchSchema: entraFetchSchema,
  metadata: {
    color: "#0078D4",
    displayName: "Microsoft Entra",
    description: "Microsoft Entra (Azure AD) identity and access management",
  },
  normalizeProperty: entraNormalizeProperty,
  isChildRequired: entraIsChildRequired,
  overrides: {
    propOverrides: ENTRA_PROP_OVERRIDES,
    schemaOverrides: ENTRA_SCHEMA_OVERRIDES,
  },
};

// Register this provider
PROVIDER_REGISTRY[entraProviderConfig.name] = entraProviderConfig;
