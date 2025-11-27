import { ExpandedPropSpecFor } from "../../spec/props.ts";
import { ExpandedPkgSpec } from "../../spec/pkgs.ts";
import pluralize from "npm:pluralize@^8.0.0";
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
      if (!openApiDescription) return;

      // Rename Assets that had different names on v1



      // Skip Assets that are not supported
      // if([
      //   // Unmappable Entities
      //   "Add-On",
      //   "Billing",
      //   // Future Assets
      //   "Uptime",
      //   "GradientAI Platform",
      // ].includes(schemaName)) {
      //   return;
      // }



      // Extract resource name from path (e.g., /v2/droplets -> droplets)
      // const pathParts = endpoint.split("/").filter((s) =>
      //   s && !s.startsWith("{")
      // );

      const pathParts = [] as string[];

      const urlSections = endpoint.split("/");
      if (urlSections.length < 3 || urlSections[1] !== 'v2') return; // TODO WARN

      let isSubAsset = false;
      let gotId = false;
      for (const urlSection of urlSections.slice(2)) {
        // If we got an id and are still moving down, this is a sub-asset that should be skipped.
        if (gotId) {
          isSubAsset = true;
          break;
        }
        // If id section, we don't add it to the pathParts, but we use it
        if (urlSection.startsWith("{")) {
          gotId = true;
          continue;
        }

        pathParts.push(urlSection);
      }

      if (isSubAsset) return;

      if (pathParts[0] === "1-clicks") return;
      if (pathParts[0] === "add-ons") return;
      if (pathParts[0] === "gen-ai") return;
      if (pathParts[0] === "sizes") return;
      if (pathParts[0] === "regions") return;
      if (pathParts[0] === "customers") return;
      if (pathParts[0] === "monitoring" && pathParts[1] === "metrics") return;

      const methods = `${
        openApiDescription.get !== undefined ? "GET " : ""
      }${
        openApiDescription.post !== undefined ? "POST " : ""
      }${
        openApiDescription.put !== undefined ? "PUT " : ""
      }${
        openApiDescription.patch !== undefined ? "PATCH " : ""
      }${
        openApiDescription.delete !== undefined ? "DELETE" : ""
      }`

      console.log(endpoint, pathParts, methods);

      // Generate a name for the resource based on the path parts
      // Make them singular and capitalized before joining
      const acronyms = new Set(['nfs', 'vpc', 'cdn', 'api', 'ssh', 'ip', 'ipv6', 'uuid', 'byoip']);

      const schemaName = pathParts
        .flatMap(part => {
          // Split on underscores and hyphens to handle compound words
          return part.split(/[-_]/);
        })
        .map(word => {
          // Special case for kubernetes - don't singularize
          if (word === 'kubernetes') {
            return 'Kubernetes';
          }
          // Check if the original word is an acronym BEFORE singularizing
          if (acronyms.has(word.toLowerCase())) {
            return word.toUpperCase();
          }
          // Singularize the word
          const singular = pluralize.singular(word);
          // Check if singularized form is an acronym
          if (acronyms.has(singular.toLowerCase())) {
            return singular.toUpperCase();
          }
          // Otherwise capitalize first letter
          return singular.charAt(0).toUpperCase() + singular.slice(1);
        })
        .join(' ');

      if (!resourceOperations[schemaName]) {
        resourceOperations[schemaName] = [];
      }

      resourceOperations[schemaName].push({
        endpoint,
        openApiDescription: {
          get: openApiDescription.get,
          post: openApiDescription.post,
          put: openApiDescription.put,
          patch: openApiDescription.patch,
          delete: openApiDescription.delete,
        },
      });
    },
  );

  // With the resource operations list, filter out assets that don't have POST endpoints.

  Object.entries(resourceOperations).forEach(([name, operations]) => {
    console.log(`DigitalOcean ${name}`);
    return;
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
      schemaName,
      name,
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
