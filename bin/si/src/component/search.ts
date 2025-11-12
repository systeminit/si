/**
 * Component Search Module - Search and display component data
 *
 * This module provides functionality to search for components in the System Initiative
 * API using search queries and display results in various formats.
 *
 * @module
 */

import {
  ActionsApi,
  type ActionViewV1,
  ComponentsApi,
  type ComponentSearchResult,
  type GetComponentV1Response,
  type GetSchemaV1Response,
  SearchApi,
  SchemasApi,
} from "@systeminit/api-client";
import { stringify as stringifyYaml } from "@std/yaml";
import { Context } from "../context.ts";
import { apiConfig, getHeadChangeSetId, WORKSPACE_ID } from "../si_client.ts";
import { resolveChangeSet } from "../change_set_utils.ts";
import { cleanForYaml, type ComponentGetCache } from "./cache.ts";
import {
  displayAttributes,
  displayComponentInfo,
} from "./get.ts";
import { cachedGetComponent, cachedGetSchema } from "../cache.ts";
import { isSubscription } from "../template/attribute_diff.ts";
import { filterAttributes } from "./attribute_utils.ts";

/**
 * Options for the component search command
 */
export interface ComponentSearchOptions {
  /** Change set ID or name (defaults to HEAD) */
  changeSet?: string;
  /** Output format: "info" (default), "json", or "yaml" */
  output?: string;
  /** Attribute paths to include in output (can be specified multiple times) */
  attribute?: string[];
  /** Show full component details for each result */
  fullComponent?: boolean;
}

/**
 * Extracts attribute values from a component by path.
 *
 * @param component - The component search result
 * @param attributePaths - The attribute paths to extract
 * @param componentsApi - ComponentsApi instance for API calls
 * @param workspaceId - Workspace ID
 * @param changeSetId - Change set ID
 * @returns Map of attribute path to value
 */
async function extractAttributes(
  component: ComponentSearchResult,
  attributePaths: string[],
  componentsApi: ComponentsApi,
  workspaceId: string,
  changeSetId: string,
): Promise<Record<string, unknown>> {
  const ctx = Context.instance();
  const attributes: Record<string, unknown> = {};

  // Fetch full component to get attributes
  const componentResponse = await componentsApi.findComponent({
    workspaceId,
    changeSetId,
    componentId: component.id,
  });

  const fullComponent = componentResponse.data.component;

  // Extract requested attributes
  for (const path of attributePaths) {
    const value = fullComponent.attributes[path];
    if (value !== undefined) {
      attributes[path] = value;
    } else {
      ctx.logger.debug(
        `Attribute {path} not found on component {componentId}`,
        {
          path,
          componentId: component.id,
        },
      );
    }
  }

  return attributes;
}

/**
 * Recursively enriches $source objects in attributes with component name and schema name.
 *
 * @param attributes - The attributes object to enrich (mutates in place)
 * @param componentCache - Cache map for component lookups
 * @param schemaCache - Cache map for schema lookups
 * @param componentsApi - ComponentsApi instance for API calls
 * @param schemasApi - SchemasApi instance for API calls
 * @param workspaceId - Workspace ID
 * @param changeSetId - Change set ID
 */
async function enrichSubscriptions(
  attributes: Record<string, unknown>,
  componentCache: Map<string, GetComponentV1Response>,
  schemaCache: Map<string, GetSchemaV1Response>,
  componentsApi: ComponentsApi,
  schemasApi: SchemasApi,
  workspaceId: string,
  changeSetId: string,
): Promise<void> {
  const ctx = Context.instance();

  // Recursively walk the attributes object
  for (const [_key, value] of Object.entries(attributes)) {
    if (isSubscription(value)) {
      // This is a $source object
      const sourceObj = value as {
        $source: {
          component: string;
          path: string;
          func?: string;
          name?: string;
          schema?: string;
        };
      };

      const componentId = sourceObj.$source.component;

      try {
        // Fetch component details using cached lookup
        const componentData = await cachedGetComponent(
          componentCache,
          componentsApi,
          ctx.logger,
          workspaceId,
          changeSetId,
          componentId,
        );

        // Fetch schema details using cached lookup
        const schemaData = await cachedGetSchema(
          schemaCache,
          schemasApi,
          ctx.logger,
          workspaceId,
          changeSetId,
          componentData.component.schemaId,
        );

        // Enrich the $source object with name and schema
        sourceObj.$source.name = componentData.component.name;
        sourceObj.$source.schema = schemaData.name;

        ctx.logger.debug(
          `Enriched subscription: {componentId} -> {name} ({schema})`,
          {
            componentId,
            name: componentData.component.name,
            schema: schemaData.name,
          },
        );
      } catch (error) {
        // If lookup fails, log warning but continue
        ctx.logger.warn(
          `Failed to enrich subscription for component {componentId}: {error}`,
          {
            componentId,
            error: error instanceof Error ? error.message : String(error),
          },
        );
      }
    } else if (typeof value === "object" && value !== null) {
      // Recursively process nested objects
      await enrichSubscriptions(
        value as Record<string, unknown>,
        componentCache,
        schemaCache,
        componentsApi,
        schemasApi,
        workspaceId,
        changeSetId,
      );
    }
  }
}

/**
 * Extracts qualification information from component.
 *
 * @param component - The component view
 * @returns Array of qualification results
 */
function extractQualifications(component: {
  domainProps?: Array<{ path: string; value?: unknown }>;
  resourceProps?: Array<{ path: string; value?: unknown }>;
}): Array<{
  name: string;
  status: string;
  message?: string;
}> {
  const qualifications: Array<{
    name: string;
    status: string;
    message?: string;
  }> = [];

  // Check domain props for qualification results
  for (const prop of component.domainProps || []) {
    if (prop.path.includes("qualification") || prop.path.includes("valid")) {
      qualifications.push({
        name: prop.path,
        status: prop.value ? "success" : "failure",
        message: typeof prop.value === "string" ? prop.value : undefined,
      });
    }
  }

  // Check resource props for qualification results
  for (const prop of component.resourceProps || []) {
    if (prop.path.includes("qualification") || prop.path.includes("valid")) {
      qualifications.push({
        name: prop.path,
        status: prop.value ? "success" : "failure",
        message: typeof prop.value === "string" ? prop.value : undefined,
      });
    }
  }

  return qualifications;
}

/**
 * Determines if a component is qualified based on its attributes.
 *
 * @param component - The component view with attributes
 * @param qualifications - Extracted qualification results
 * @returns True if component is qualified
 */
function isQualified(
  component: { attributes: Record<string, unknown> },
  qualifications: Array<{ status: string }>,
): boolean {
  // Check if there's a qualification attribute
  const qualAttr = component.attributes["/si/qualification"] ||
    component.attributes["/domain/qualified"];

  if (qualAttr !== undefined) {
    return Boolean(qualAttr);
  }

  // If no explicit qualification, assume qualified if no errors
  if (qualifications.length === 0) {
    return true; // No qualifications means qualified by default
  }

  // All qualifications must pass
  return qualifications.every((q) => q.status === "success");
}

/**
 * Filters actions to only those belonging to a specific component.
 *
 * @param actions - All actions in the change set
 * @param componentId - The component ID to filter by
 * @returns Array of actions for this component
 */
function filterActionsByComponent(
  actions: ActionViewV1[],
  componentId: string,
): Array<{
  id: string;
  name: string;
  state: string;
}> {
  return actions
    .filter((action) => action.componentId === componentId)
    .map((action) => ({
      id: action.id || "",
      name: action.name || "",
      state: action.state || "unknown",
    }));
}

/**
 * Interface for search result output
 */
interface SearchResultOutput {
  count: number;
  components: Array<{
    id: string;
    name: string;
    schema: {
      name: string;
    };
    attributes?: Record<string, unknown>;
    fullComponent?: ComponentGetCache;
  }>;
}

/**
 * Searches for components and displays results.
 *
 * @param query - Search query string
 * @param options - Command options
 */
export async function componentSearch(
  query: string,
  options: ComponentSearchOptions,
): Promise<void> {
  const ctx = Context.instance();

  if (!apiConfig || !WORKSPACE_ID) {
    throw new Error("API configuration not initialized");
  }

  // Determine output format early to control logging
  const outputFormat = options.output || "info";

  // Only log when in info mode to keep json/yaml output clean
  if (outputFormat === "info") {
    ctx.logger.info(`Searching for components: {query}`, { query });
  }

  // Resolve change set
  const changeSetId = options.changeSet
    ? await resolveChangeSet(WORKSPACE_ID, options.changeSet)
    : await getHeadChangeSetId(apiConfig, WORKSPACE_ID);

  ctx.logger.debug(`Using change set: {changeSetId}`, { changeSetId });

  // Execute search
  const searchApi = new SearchApi(apiConfig);
  const searchResponse = await searchApi.search({
    workspaceId: WORKSPACE_ID,
    changeSetId,
    q: query,
  });

  const components = searchResponse.data.components;
  const count = components.length;

  ctx.logger.debug(`Found {count} components`, { count });

  // Handle empty results
  if (count === 0) {
    if (outputFormat === "info") {
      ctx.logger.warn("No components found matching query: {query}", { query });
    } else if (outputFormat === "json") {
      console.log(JSON.stringify({ count: 0, components: [] }, null, 2));
    } else if (outputFormat === "yaml") {
      console.log(stringifyYaml({ count: 0, components: [] }));
    }
    return;
  }

  // Build output structure
  const output: SearchResultOutput = {
    count,
    components: [],
  };

  const componentsApi = new ComponentsApi(apiConfig);

  // Process each component
  for (let i = 0; i < components.length; i++) {
    const component = components[i];
    const componentData: SearchResultOutput["components"][0] = {
      id: component.id,
      name: component.name,
      schema: {
        name: component.schema.name,
      },
    };

    // Fetch attributes if requested
    if (options.attribute && options.attribute.length > 0) {
      componentData.attributes = await extractAttributes(
        component,
        options.attribute,
        componentsApi,
        WORKSPACE_ID,
        changeSetId,
      );
    }

    // Fetch full component if requested
    if (options.fullComponent) {
      // Fetch the full component data
      const fullComponentResponse = await componentsApi.findComponent({
        workspaceId: WORKSPACE_ID,
        changeSetId,
        componentId: component.id,
      });
      const fullComponent = fullComponentResponse.data.component;

      // Fetch schema name
      const schemasApi = new SchemasApi(apiConfig);
      const schemaResponse = await schemasApi.getSchema({
        workspaceId: WORKSPACE_ID,
        changeSetId,
        schemaId: fullComponent.schemaId,
      });
      const schemaName = schemaResponse.data.name;

      // Fetch actions
      const actionsApi = new ActionsApi(apiConfig);
      const actionsResponse = await actionsApi.getActions({
        workspaceId: WORKSPACE_ID,
        changeSetId,
      });
      const actions = actionsResponse.data.actions as ActionViewV1[];
      const componentActions = filterActionsByComponent(actions, fullComponent.id);

      // Filter attributes to relevant paths
      const filteredAttrs = filterAttributes(fullComponent.attributes);

      // Extract qualifications
      const qualifications = extractQualifications(fullComponent);

      // Determine qualified status
      const qualified = isQualified(fullComponent, qualifications);

      // Extract resource data
      const resourceData = fullComponent.attributes["/resource_value"];

      // Extract resource payload (as separate field)
      const resourcePayload = fullComponent.attributes["/resource/payload"];

      // Build cache data structure (same as component get)
      const cacheData: ComponentGetCache = {
        componentId: fullComponent.id,
        schemaId: fullComponent.schemaId,
        schemaName: schemaName,
        resourceId: fullComponent.resourceId,
        toDelete: fullComponent.toDelete,
        canBeUpgraded: fullComponent.canBeUpgraded,
        qualified: qualified,
        attributes: filteredAttrs,
        resourceData: resourceData as Record<string, unknown> | undefined,
        resource: resourcePayload as Record<string, unknown> | undefined,
        qualifications: qualifications,
        actions: componentActions,
      };

      // Enrich subscriptions with component name and schema name
      const componentCache = new Map<string, GetComponentV1Response>();
      const schemaCache = new Map<string, GetSchemaV1Response>();

      await enrichSubscriptions(
        cacheData.attributes,
        componentCache,
        schemaCache,
        componentsApi,
        schemasApi,
        WORKSPACE_ID,
        changeSetId,
      );

      componentData.fullComponent = cacheData;
    }

    output.components.push(componentData);
  }

  // Handle output based on format
  if (outputFormat === "json") {
    console.log(JSON.stringify(output, null, 2));
  } else if (outputFormat === "yaml") {
    console.log(stringifyYaml(cleanForYaml(output)));
  } else {
    // Info mode: display results
    ctx.logger.info("Found {count} components", { count });

    // Create API instances and caches for full component display
    const schemasApi = new SchemasApi(apiConfig);
    const actionsApi = new ActionsApi(apiConfig);
    let allActions: ActionViewV1[] | undefined;

    // Fetch all actions once if full component display is requested
    if (options.fullComponent) {
      const actionsResponse = await actionsApi.getActions({
        workspaceId: WORKSPACE_ID,
        changeSetId,
      });
      allActions = actionsResponse.data.actions as ActionViewV1[];
    }

    for (let i = 0; i < output.components.length; i++) {
      const componentData = output.components[i];
      const index = i + 1;

      // Display component header: "Component: AWS EC2 Instance component-name 1/10"
      ctx.logger.info("Component: {schema} {name} {index}/{total}", {
        schema: componentData.schema.name,
        name: componentData.name,
        index,
        total: count,
      });

      // Display attributes if requested (without full component)
      if (componentData.attributes && !options.fullComponent) {
        displayAttributes(componentData.attributes);
      }

      // Display full component if requested
      if (componentData.fullComponent && options.fullComponent) {
        // componentData.fullComponent is already a ComponentGetCache
        // that was correctly built earlier with proper qualifications
        // and enriched subscriptions, so just use it directly
        displayComponentInfo(componentData.fullComponent);
      }
    }
  }
}
