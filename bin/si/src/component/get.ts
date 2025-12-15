/**
 * Component Get Module - Fetch and display component data
 *
 * This module provides functionality to retrieve detailed component information
 * from the System Initiative API, including attributes, qualifications, and
 * enqueued actions.
 *
 * @module
 */

import {
  ActionsApi,
  type ActionViewV1,
  ComponentsApi,
  type ComponentViewV1,
  type GetComponentV1Response,
  type GetSchemaV1Response,
  SchemasApi,
} from "@systeminit/api-client";
import { stringify as stringifyYaml } from "@std/yaml";
import { Context } from "../context.ts";
import {
  cacheComponentData,
  cleanForYaml,
  type ComponentGetCache,
} from "./cache.ts";
import { cachedGetComponent, cachedGetSchema } from "./cache_api.ts";
import { isSubscription } from "../template/attribute_diff.ts";
import { resolveChangeSet } from "./change_set.ts";
import { filterAttributes } from "./attribute_utils.ts";
import { getHeadChangeSetId } from "../cli/helpers.ts";

/**
 * Options for the component get command
 */
export interface ComponentGetOptions {
  /** Change set ID or name (defaults to HEAD) */
  changeSet?: string;
  /** Output format: "info" (default), "json", or "yaml" */
  output?: string;
  /** Cache file path (format determined by extension) */
  cache?: string;
  /** Output raw API response as JSON and exit */
  raw?: boolean;
}

/**
 * Extracts qualification information from component props.
 *
 * @param component - The component view
 * @returns Array of qualification results
 */
function extractQualifications(component: ComponentViewV1): Array<{
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
 * Determines if a component is qualified based on its attributes and props.
 *
 * @param component - The component view
 * @returns True if component is qualified
 */
function isQualified(component: ComponentViewV1): boolean {
  // Check if there's a qualification attribute
  const qualAttr =
    component.attributes["/si/qualification"] ||
    component.attributes["/domain/qualified"];

  if (qualAttr !== undefined) {
    return Boolean(qualAttr);
  }

  // If no explicit qualification, assume qualified if no errors
  const qualifications = extractQualifications(component);
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
 * Recursively enriches $source objects in attributes with component name and schema name.
 *
 * This function walks through all attributes looking for subscription ($source) objects
 * and augments them with human-readable component name and schema name.
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
 * Formats and displays JSON with proper alignment for interactive mode.
 * In interactive mode, LogTape pretty formatter adds prefix before message.
 * Format: [emoji(2)] [space] [level(~4)] [spaces(~4)] [category(15)] = ~27 chars
 *
 * @param ctx - The context instance
 * @param json - The JSON string to display
 */
function formatJsonWithAlignment(ctx: Context, json: string): void {
  if (ctx.isInteractive) {
    const lines = json.split("\n");
    for (const line of lines) {
      console.log(" ".repeat(27) + line);
    }
  } else {
    console.log(json);
  }
}

/**
 * Displays just the attributes section in info mode format.
 * This function is used by both get and search commands for consistent formatting.
 *
 * @param attributes - The attributes to display
 */
export function displayAttributes(attributes: Record<string, unknown>): void {
  const ctx = Context.instance();
  ctx.logger.info("Attributes:");
  formatJsonWithAlignment(ctx, JSON.stringify(attributes, null, 2));
}

/**
 * Displays full component information in info mode format.
 * This function is used by both get and search commands for consistent formatting.
 *
 * @param cacheData - The component data to display
 */
export function displayComponentInfo(cacheData: ComponentGetCache): void {
  const ctx = Context.instance();

  ctx.logger.info("Component ID: {componentId}", {
    componentId: cacheData.componentId,
  });
  ctx.logger.info("Schema: {schemaName} ({schemaId})", {
    schemaName: cacheData.schemaName,
    schemaId: cacheData.schemaId,
  });
  if (cacheData.resourceId) {
    ctx.logger.info("Resource ID: {resourceId}", {
      resourceId: cacheData.resourceId,
    });
  }
  ctx.logger.info("To Delete: {toDelete}", {
    toDelete: cacheData.toDelete,
  });
  ctx.logger.info("Can Be Upgraded: {canBeUpgraded}", {
    canBeUpgraded: cacheData.canBeUpgraded,
  });
  ctx.logger.info("Qualified: {qualified}", {
    qualified: cacheData.qualified,
  });

  ctx.logger.info("Attributes:");
  formatJsonWithAlignment(ctx, JSON.stringify(cacheData.attributes, null, 2));

  if (cacheData.resourceData) {
    ctx.logger.info("Resource Data:");
    formatJsonWithAlignment(
      ctx,
      JSON.stringify(cacheData.resourceData, null, 2),
    );
  }

  if (cacheData.resource) {
    ctx.logger.info("Resource:");
    formatJsonWithAlignment(ctx, JSON.stringify(cacheData.resource, null, 2));
  }

  if (cacheData.qualifications.length > 0) {
    ctx.logger.info("Qualifications:");
    for (const qual of cacheData.qualifications) {
      ctx.logger.info("  {name}: {status}", {
        name: qual.name,
        status: qual.status,
      });
      if (qual.message) {
        ctx.logger.info("    Message: {message}", {
          message: qual.message,
        });
      }
    }
  }

  if (cacheData.actions.length > 0) {
    ctx.logger.info("Enqueued Actions:");
    for (const action of cacheData.actions) {
      ctx.logger.info("  {name} ({id}): {state}", {
        name: action.name,
        id: action.id,
        state: action.state,
      });
    }
  }
}

/**
 * Fetches and displays component data.
 *
 * @param componentIdOrName - Component ID or name
 * @param options - Command options
 */
export async function callComponentGet(
  componentIdOrName: string,
  options: ComponentGetOptions,
): Promise<void> {
  const ctx = Context.instance();
  const apiConfig = Context.apiConfig();
  const workspaceId = Context.workspaceId();

  // Determine output format early to control logging
  const outputFormat = options.output || "info";

  // Only log when in info mode to keep json/yaml output clean
  if (outputFormat === "info") {
    ctx.logger.info(`Fetching component: {component}`, {
      component: componentIdOrName,
    });
  }

  // Resolve change set
  const changeSetId = options.changeSet
    ? await resolveChangeSet(workspaceId, options.changeSet)
    : await getHeadChangeSetId();

  ctx.logger.debug(`Using change set: {changeSetId}`, { changeSetId });

  // Fetch component data
  const componentsApi = new ComponentsApi(apiConfig);

  // Detect if it's a ULID (26 alphanumeric characters)
  // ULIDs use Crockford's base32: 0-9 and A-Z excluding I, L, O, U
  const isUlid = /^[0-9A-HJKMNP-TV-Z]{26}$/i.test(componentIdOrName);

  const componentResponse = await componentsApi.findComponent({
    workspaceId,
    changeSetId: changeSetId,
    // Use componentId parameter for ULIDs, component parameter for names
    ...(isUlid
      ? { componentId: componentIdOrName }
      : { component: componentIdOrName }),
  });

  const component = componentResponse.data.component;

  ctx.logger.debug(`Found component: {id} ({name})`, {
    id: component.id,
    name: component.name,
  });

  // If raw mode, output the component data as JSON and exit
  if (options.raw) {
    console.log(JSON.stringify(component, null, 2));
    Deno.exit(0);
  }

  // Fetch schema name
  const schemasApi = new SchemasApi(apiConfig);
  const schemaResponse = await schemasApi.getSchema({
    workspaceId,
    changeSetId: changeSetId,
    schemaId: component.schemaId,
  });

  const schemaName = schemaResponse.data.name;

  // Fetch actions
  const actionsApi = new ActionsApi(apiConfig);
  const actionsResponse = await actionsApi.getActions({
    workspaceId,
    changeSetId: changeSetId,
  });

  const actions = actionsResponse.data.actions as ActionViewV1[];
  const componentActions = filterActionsByComponent(actions, component.id);

  // Filter attributes to relevant paths
  const filteredAttrs = filterAttributes(component.attributes);

  // Extract qualifications
  const qualifications = extractQualifications(component);

  // Determine qualified status
  const qualified = isQualified(component);

  // Extract resource data
  const resourceData = component.attributes["/resource_value"];

  // Extract resource payload (as separate field)
  const resourcePayload = component.attributes["/resource/payload"];

  // Build cache data structure
  const cacheData: ComponentGetCache = {
    componentId: component.id,
    schemaId: component.schemaId,
    schemaName: schemaName,
    resourceId: component.resourceId,
    toDelete: component.toDelete,
    canBeUpgraded: component.canBeUpgraded,
    qualified: qualified,
    attributes: filteredAttrs,
    resourceData: resourceData,
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
    workspaceId,
    changeSetId,
  );

  // Handle output based on format (outputFormat already determined at start)
  if (outputFormat === "json") {
    // Suppress logs and output JSON to stdout
    console.log(JSON.stringify(cacheData, null, 2));
  } else if (outputFormat === "yaml") {
    // Suppress logs and output YAML to stdout
    // Clean undefined values as YAML stringify cannot handle them
    console.log(stringifyYaml(cleanForYaml(cacheData)));
  } else {
    // Default: pretty print with logger using shared display function
    displayComponentInfo(cacheData);
  }

  // Cache if requested
  if (options.cache) {
    await cacheComponentData(cacheData, options.cache, ctx.logger);
  }

  ctx.analytics.trackEvent("component get", {
    schemaName,
    outputFormat,
    setToCache: options.cache ?? false,
  });
}
