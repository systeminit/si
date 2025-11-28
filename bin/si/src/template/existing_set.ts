import { ComponentsApi, SearchApi } from "@systeminit/api-client";
import type { TemplateContext } from "./context.ts";
import { filterComponentAttributes } from "./context.ts";
import type { ExistingSetComponent } from "./converge_types.ts";
import { logComponentWithSchema } from "./helpers.ts";

/**
 * Query SI for components created by this template using tag search.
 * Returns components that have the templateFrom tag matching this template's invocation.
 *
 * @param ctx - Template context
 * @param changeSetId - Change set ID to query components from
 * @returns Array of existing components with template tags
 */
export async function queryExistingSet(
  ctx: TemplateContext,
  changeSetId: string,
): Promise<ExistingSetComponent[]> {
  const apiConfig = ctx.apiConfig();
  const workspaceId = ctx.workspaceId();

  if (!apiConfig || !workspaceId) {
    ctx.logger.warn("Cannot query existing set: API not configured");
    return [];
  }

  // Search for components with templateFrom tag
  const templateTag = `${ctx.name()}-${ctx.invocationKey()}`;
  const query = `/si/tags/templateFrom:"${templateTag}"`;
  ctx.logger.debug("Querying existing set: {query}", { query });

  const searchApi = new SearchApi(apiConfig);
  const componentsApi = new ComponentsApi(apiConfig);

  const searchResult = await searchApi.search({
    workspaceId,
    changeSetId,
    q: query,
  });

  ctx.logger.info("Found {count} existing components", {
    count: searchResult.data.components.length,
  });

  // Fetch full component data for each
  const existing: ExistingSetComponent[] = [];
  const total = searchResult.data.components.length;

  for (let i = 0; i < searchResult.data.components.length; i++) {
    const component = searchResult.data.components[i];
    const current = i + 1;

    const fullComponent = await componentsApi.getComponent({
      workspaceId,
      changeSetId,
      componentId: component.id,
    });

    const comp = fullComponent.data.component;
    const workingSetId = comp.attributes?.["/si/tags/templateWorkingSetId"];

    if (!workingSetId) {
      ctx.logger.warn(`Component {id} missing templateWorkingSetId tag`, {
        id: comp.id,
      });
      continue;
    }

    // Fetch schema name from cache
    const schemaName = await ctx.getSchemaName(
      workspaceId,
      changeSetId,
      comp.schemaId,
    );

    // Log component loading with schema name
    await logComponentWithSchema(
      ctx,
      comp,
      workspaceId,
      changeSetId,
      `Loaded existing component {schemaName} {siName} ({current}/{total})`,
      current,
      total,
    );

    existing.push({
      id: comp.id,
      schemaId: comp.schemaId,
      schemaName: schemaName,
      name: comp.name,
      resourceId: comp.resourceId,
      attributes: filterComponentAttributes(comp.attributes || {}),
      templateWorkingSetId: workingSetId,
    });
  }

  return existing;
}
