/**
 * Stage 2: Source Data Collection
 * Uses System Initiative API to collect component data based on queries
 */

import {
  ComponentsApi,
  SearchApi,
} from "@systeminit/api-client";
import { Context } from "../context.ts";

export interface ComponentData {
  componentId: string;
  schema: string;
  "si/name": string;
  // deno-lint-ignore no-explicit-any
  [key: string]: any; // Additional attributes
}

export interface SourceDataCollection {
  [queryName: string]: ComponentData[];
}

/**
 * Source Data Collector using System Initiative API
 */
export class SourceDataCollector {
  private componentsApi: ComponentsApi;
  private searchApi: SearchApi;
  private workspaceId: string;

  constructor() {
    const apiConfig = Context.apiConfig();
    this.workspaceId = Context.workspaceId();

    // Initialize API clients
    this.componentsApi = new ComponentsApi(apiConfig);
    this.searchApi = new SearchApi(apiConfig);
  }

  /**
   * Search for components matching a query
   */
  async searchComponents(
    changeSetId: string,
    query: string,
  ): Promise<ComponentData[]> {
    const ctx = Context.instance();
    ctx.logger.debug(`Search string sent to API: {query}`, { query });

    const response = await this.searchApi.search({
      workspaceId: this.workspaceId,
      changeSetId,
      q: query,
    });

    const components = response.data.components || [];
    ctx.logger.debug(`Search returned: {count} component(s)`, {
      count: components.length,
    });

    // Extract component data
    const componentData: ComponentData[] = [];

    for (const component of components) {
      // Get full component details to access all attributes
      const detailResponse = await this.componentsApi.findComponent({
        workspaceId: this.workspaceId,
        changeSetId,
        componentId: component.id,
      });

      const fullComponent = detailResponse.data.component;
      const attributes = fullComponent.attributes || {};

      // Extract schema name from the nested schema object
      const schemaName = component.schema?.name || "unknown";

      componentData.push({
        componentId: component.id,
        schema: schemaName,
        "si/name": attributes["/root/si/name"] ||
          attributes["/domain/name"] || component.name || "unknown",
        // Include other useful attributes
        ...attributes,
      });
    }

    return componentData;
  }

  /**
   * Collect source data for all queries
   */
  async collect(
    changeSetId: string,
    queries: Record<string, string>,
  ): Promise<SourceDataCollection> {
    const ctx = Context.instance();
    ctx.logger.info("Stage 2: Collecting source data...");
    ctx.logger.debug("Using change set: {changeSetId}", { changeSetId });

    // Collect data for each query
    const results: SourceDataCollection = {};

    for (const [queryName, queryString] of Object.entries(queries)) {
      ctx.logger.info("Collecting data for: {queryName}", { queryName });
      ctx.logger.debug("Query from policy: {query}", { query: queryString });
      const components = await this.searchComponents(
        changeSetId,
        queryString,
      );
      results[queryName] = components;
    }

    return results;
  }
}

/**
 * Main function to collect source data and write to file
 */
export async function collectSourceData(
  changeSetId: string,
  queries: Record<string, string>,
  outputPath: string,
): Promise<SourceDataCollection> {
  const ctx = Context.instance();
  const collector = new SourceDataCollector();
  const data = await collector.collect(changeSetId, queries);

  // Write to file using Deno
  await Deno.writeTextFile(outputPath, JSON.stringify(data, null, 2));
  ctx.logger.info("Source data collection complete");
  ctx.logger.debug("Output: {path}", { path: outputPath });

  return data;
}
