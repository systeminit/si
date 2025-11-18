import { Context } from "../context.ts";
import { ComponentsApi, SearchApi } from "@systeminit/api-client";
import type { TemplateComponent, TemplateContext } from "./context.ts";
import { componentViewToTemplateComponent } from "./context.ts";
import { loadDataFromFile } from "./input.ts";
import type { BaselineCache } from "./cache.ts";
import { getHeadChangeSetId } from "../si_client.ts";
import { logComponentWithSchema } from "../helpers.ts";

/**
 * Set baseline data by searching for components using the context's search strings.
 * If baseline is already set, this function will skip the search.
 *
 * @param ctx - The template context
 */
export async function setBaseline(ctx: TemplateContext): Promise<void> {
  // If baseline is already set, skip search
  if (ctx.baseline() !== undefined) {
    ctx.logger.debug("Using supplied baseline data");
    return;
  }

  const apiConfig = ctx.apiConfig();
  const workspaceId = ctx.workspaceId();

  if (!apiConfig || !workspaceId) {
    ctx.logger.warn("Cannot set baseline: API configuration not available");
    return;
  }

  // Get the HEAD changeset ID (will throw if not found)
  const changeSetId = await getHeadChangeSetId(apiConfig, workspaceId);

  const searchStrings = ctx.search();
  if (!searchStrings || searchStrings.length === 0) {
    ctx.logger.warn("No search strings configured, baseline will be empty");
    ctx.baseline([]);
    return;
  }

  ctx.logger.info(`Building baseline with search strings: {searchStrings}`, {
    searchStrings,
  });

  const searchApi = new SearchApi(apiConfig);
  const componentsApi = new ComponentsApi(apiConfig);

  // Search for components using each search string
  const componentIds = new Set<string>();
  for (const searchString of searchStrings) {
    const searchResult = await searchApi.search({
      workspaceId,
      changeSetId,
      q: searchString,
    });

    for (const component of searchResult.data.components) {
      componentIds.add(component.id);
    }
  }

  ctx.logger.info(`Found {count} unique components from search`, {
    count: componentIds.size,
  });

  // Fetch full component data for each ID and convert to TemplateComponent
  const components: TemplateComponent[] = [];
  const componentIdsArray = Array.from(componentIds);
  const total = componentIdsArray.length;

  for (let i = 0; i < componentIdsArray.length; i++) {
    const componentId = componentIdsArray[i];
    const current = i + 1;

    ctx.logger.debug(`Fetching data for {componentId}`, { componentId });
    const componentResult = await componentsApi.getComponent({
      workspaceId,
      changeSetId,
      componentId,
    });

    const component = componentResult.data.component;

    // Fetch schema name from cache
    const schemaName = await ctx.getSchemaName(
      workspaceId,
      changeSetId,
      component.schemaId,
    );

    // Log component loading with schema name
    await logComponentWithSchema(
      ctx,
      component,
      workspaceId,
      changeSetId,
      `Loaded baseline component {schemaName} {siName} ({current}/{total})`,
      current,
      total,
    );

    // Convert to TemplateComponent with filtered attributes and schema name
    components.push(componentViewToTemplateComponent(component, schemaName));
  }

  ctx.logger.info(`Built baseline with {count} components from search`, {
    count: components.length,
  });
  ctx.baseline(components);
}

/**
 * Load baseline data and schema cache from a JSON or YAML file.
 * Sets both baseline and schema cache on the provided context.
 *
 * @param tctx - Template context to populate
 * @param filePath - Path to the baseline cache file
 */
export async function loadBaselineFromFile(
  tctx: TemplateContext,
  filePath: string,
): Promise<void> {
  const ctx = Context.instance();

  ctx.logger.info(`Loading baseline data from {filePath}`, { filePath });

  const data = await loadDataFromFile(filePath);

  // Validate it's an object with components and schemas
  if (typeof data !== "object" || data === null) {
    throw new Error(
      `Baseline file must contain an object with components and schemas`,
    );
  }

  const cacheData = data as BaselineCache;

  if (!Array.isArray(cacheData.components)) {
    throw new Error(`Baseline file must have a 'components' array`);
  }

  if (typeof cacheData.schemas !== "object" || cacheData.schemas === null) {
    throw new Error(`Baseline file must have a 'schemas' object`);
  }

  ctx.logger.debug(
    `Loaded {componentCount} components and {schemaCount} schemas from baseline file`,
    {
      componentCount: cacheData.components.length,
      schemaCount: Object.keys(cacheData.schemas).length,
    },
  );

  // Set baseline
  tctx.baseline(cacheData.components);

  // Populate schema cache from object
  const schemaCache = tctx.schemaCache();
  for (const [schemaId, schema] of Object.entries(cacheData.schemas)) {
    schemaCache.set(schemaId, schema);
  }
}
