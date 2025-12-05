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
import logger from "../../logger.ts";

function createDocLink(
  schema: SuperSchema,
  defName: string | undefined,
  propName?: string,
): string {
  const docLink = "https://docs.digitalocean.com/reference/api/digitalocean/";

  // Use the docTag from the schema if available
  const digitalOceanSchema = schema as DigitalOceanSchema;
  const tagName = digitalOceanSchema.docTag || schema.typeName.replace("DigitalOcean ", "");

  // Replace spaces with hyphens for URL compatibility
  const tag = tagName.replace(/ /g, '-');

  if (defName) {
    return `${docLink}#tag/${tag}`;
  }

  if (propName) {
    return `${docLink}#tag/${tag}`;
  }

  return `${docLink}#tag/${tag}`;
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

      const pathParts = [] as string[];

      const urlSections = endpoint.split("/");
      if (urlSections.length < 3 || urlSections[1] !== 'v2') {
        logger.warn(`DigitalOcean endpoint ${endpoint} does not match expected format (/v2/*). Skipping.`);
        return;
      }

      let isSubAsset = false;
      let hasId = false;
      for (const urlSection of urlSections.slice(2)) {
        // If we got an id and are still moving down, this is a sub-asset that should be skipped.
        if (hasId) {
          isSubAsset = true;
          break;
        }
        // If id section, we don't add it to the pathParts, but we use it
        if (urlSection.startsWith("{")) {
          hasId = true;
          continue;
        }

        pathParts.push(urlSection);
      }

      if (isSubAsset) return;

      // Skipping endpoints that do not represent assets
      if (pathParts[0] === "add-ons") return;
      if (pathParts[0] === "gen-ai") return;

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

      logger.debug(`${endpoint} -> ${methods}`);

      // Generate a name for the resource based on the path parts
      // Make them singular and capitalized before joining
      const acronymList = ['nfs', 'vpc', 'cdn', 'api', 'ssh', 'ip', 'ipv6', 'uuid', 'byoip'];
      const acronyms = new Set(acronymList);

      // Since tech words are often weird, we set a list of words that don't lose the final S when singular.
      acronymList.forEach(pluralize.addUncountableRule);
      pluralize.addUncountableRule('kubernetes');

      const schemaNameRaw = pathParts
        // Split on underscores and hyphens to handle compound words
        .flatMap(part => part.split(/[-_]/))
        .map(word => {


          // Singularize the word
          const singular = pluralize.singular(word);

          if (acronyms.has(singular.toLowerCase())) {
            return singular.toUpperCase();
          }

          // Otherwise capitalize the first letter
          return singular.charAt(0).toUpperCase() + singular.slice(1);
        })
        .join(' ');

      // For historical reasons we need to rename these assets to match the names we already had
      const nameOverrides = {
        "App": "App Platform",
        "Registry": "Container Registry",
        "Image": "Custom Image",
        "Database": "Database Cluster",
        "Monitoring Alert": "Monitoring Alert Policy",
        "Account Key": "SSH Key",
        "Certificate": "SSL Certificate",
      } as Record<string, string>;

      const schemaName = nameOverrides[schemaNameRaw] || schemaNameRaw;

      // DO has /v2/registry and v2/registries, and we only use one of them since the operations overlap.
      if (
        schemaName === "Container Registry" &&
        endpoint === "/v2/registry"
      ) {
        return;
      }

      resourceOperations[schemaName] ??= [];

      resourceOperations[schemaName].push({
        endpoint,
        endpointHasId: hasId,
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

  Object.entries(resourceOperations).forEach(([schemaName, operations]) => {

    const hasPost = operations.some(op => op.openApiDescription.post !== undefined);
    const hasGet = operations.some(op => op.openApiDescription.get !== undefined);

    if (operations.length > 2) {
      logger.warn(`DigitalOcean ${schemaName} has more than 2 operations. Skipping.`);
      return;
    }

    const endpointWithId = operations
      .find(op => op.endpointHasId);

    const endpointWithoutId = operations
      .find(op => !op.endpointHasId);

    if (!hasPost || !hasGet || !endpointWithId || !endpointWithoutId) return;

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
      endpointWithoutId.endpoint,
      operations,
      description,
    );

    if (!result) return;

    const spec = makeModule(
      result.schema,
      htmlToMarkdown(result.schema.description) ?? result.schema.description,
      result.onlyProperties,
      digitalOceanProviderConfig,
      result.domainProperties,
      result.resourceValueProperties,
    );
    specs.push(spec);

  });

  logger.info(`Parsed ${specs.length} DigitalOcean schemas.`)

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
  isStable: true,
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
