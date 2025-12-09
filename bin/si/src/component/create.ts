/**
 * Component Create Module - Create component
 * 
 * @module
 */

import {
    ComponentsApi,
} from "@systeminit/api-client";
import { Context } from "../context.ts";
import { resolveChangeSet } from "./change_set.ts";
import { stringify as stringifyYaml } from "@std/yaml";
import { loadComponentFile } from "./update.ts";
import { getHeadChangeSetId } from "../cli/helpers.ts";


export interface ComponentCreateOptions {
  /** Change set ID or name */
  changeSet: string;
  /** Output format: "info" (default), "json", or "yaml" */
  output?: string;
  /** Output raw API response as JSON and exit */
  raw?: boolean;
}

/**
 * Fetches and displays component data.
 *
 * @param componentIdOrName - Component ID or name
 * @param options - Command options
 */
export async function callComponentCreate(
  inputFile: string,
  options: ComponentCreateOptions,
): Promise<void> {
  const ctx = Context.instance();
  const apiConfig = Context.apiConfig();
  const workspaceId = Context.workspaceId();

  ctx.logger.info(`Loading component data from {file}`, { file: inputFile });
  const inputData = await loadComponentFile(inputFile);

  // Determine output format early to control logging
  const outputFormat = options.output || "info";

  const attributes = inputData.attributes || {};
  const componentName = inputData.attributes["/si/name"] as string;
  const schemaName = inputData.schemaName;
  if (outputFormat === "info") {
    ctx.logger.info(`Creating component: {schemaName} {componentName}`, {
      componentName,
      schemaName,
    });
  }

  const head = await getHeadChangeSetId();
  const changeSetId = await resolveChangeSet(workspaceId, options.changeSet)

  if (head === changeSetId) {
    ctx.logger.error("Please specify a Change Set ID that is not the HEAD Change Set.")
    Deno.exit(0);
  }

  const componentsApi = new ComponentsApi(apiConfig);

  const resp = await componentsApi.createComponent({
    workspaceId,
    changeSetId,
    createComponentV1Request: {
      name: componentName,
      schemaName,
      attributes,
    }
  });

  const data = {id: resp.data.component.id};
  if (options.raw) {
    console.log(resp.data.component.id);
  } else 
  switch (outputFormat) {
    case "info":
      console.log(`Component ID: ${resp.data.component.id}`);
      break;
    case "json":
      console.log(JSON.stringify(data, null, 2));
      break;
    case "yaml":
      console.log(stringifyYaml(data));
      break;
  }
}