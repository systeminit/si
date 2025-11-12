/**
 * Component Update Module - Update component attributes from file
 *
 * This module provides functionality to update a component's attributes
 * from a JSON or YAML file, using the same idempotent convergence logic
 * as template execution.
 *
 * @module
 */

import {
  ComponentsApi,
  type GetComponentV1Response,
  SearchApi,
  SchemasApi,
} from "@systeminit/api-client";
import { parse as parseYaml } from "@std/yaml";
import { extname } from "@std/path";
import { Context } from "../context.ts";
import { apiConfig, WORKSPACE_ID } from "../si_client.ts";
import { resolveChangeSet } from "../change_set_utils.ts";
import { filterAttributes } from "./attribute_utils.ts";
import { updateComponent } from "./update_utils.ts";
import { computeAttributeDiff } from "../template/attribute_diff.ts";
import { resolveSubscriptionsInAttributes } from "./subscription_resolver.ts";
import type { ComponentGetCache } from "./cache.ts";

/**
 * Options for the component update command
 */
export interface ComponentUpdateOptions {
  /** Override component ID or name from file */
  component?: string;
  /** Change set ID or name (required) */
  changeSet: string;
  /** Show diff without applying changes */
  dryRun?: boolean;
}

/**
 * Loads component data from a JSON or YAML file.
 *
 * @param filePath - Path to the input file
 * @returns Parsed component data
 * @throws Error if file cannot be read or parsed
 */
async function loadComponentFile(
  filePath: string,
): Promise<ComponentGetCache> {
  const ctx = Context.instance();
  ctx.logger.debug(`Loading component file: {filePath}`, { filePath });

  try {
    const content = await Deno.readTextFile(filePath);
    const ext = extname(filePath).toLowerCase();

    let data: unknown;
    if (ext === ".json") {
      data = JSON.parse(content);
    } else if (ext === ".yaml" || ext === ".yml") {
      data = parseYaml(content);
    } else {
      throw new Error(
        `Unsupported file extension: ${ext}. Use .json, .yaml, or .yml`,
      );
    }

    // Validate that required fields exist
    if (
      typeof data !== "object" || data === null || !("attributes" in data)
    ) {
      throw new Error(
        "Invalid component file format: missing 'attributes' field",
      );
    }

    return data as ComponentGetCache;
  } catch (error) {
    if (error instanceof Deno.errors.NotFound) {
      throw new Error(`File not found: ${filePath}`);
    }
    throw error;
  }
}

/**
 * Fetches the current component from the API.
 *
 * @param api - ComponentsApi instance
 * @param workspaceId - Workspace ID
 * @param componentIdOrName - Component ID or name
 * @param changeSetId - The change set ID
 * @returns The component data
 * @throws Error if component not found
 */
async function fetchCurrentComponent(
  api: ComponentsApi,
  workspaceId: string,
  componentIdOrName: string,
  changeSetId: string,
): Promise<GetComponentV1Response> {
  const ctx = Context.instance();
  ctx.logger.debug(`Fetching component: {component}`, {
    component: componentIdOrName,
  });

  // Detect if it's a ULID (26 alphanumeric characters)
  // ULIDs use Crockford's base32: 0-9 and A-Z excluding I, L, O, U
  const isUlid = /^[0-9A-HJKMNP-TV-Z]{26}$/i.test(componentIdOrName);

  const response = await api.findComponent({
    workspaceId,
    changeSetId,
    // Use componentId parameter for ULIDs, component parameter for names
    ...(isUlid
      ? { componentId: componentIdOrName }
      : { component: componentIdOrName }),
  });

  if (!response.data) {
    throw new Error(`Component not found: ${componentIdOrName}`);
  }

  return response.data as GetComponentV1Response;
}

/**
 * Executes the component update command.
 *
 * @param inputFile - Path to the input JSON/YAML file
 * @param options - Update command options
 */
export async function componentUpdate(
  inputFile: string,
  options: ComponentUpdateOptions,
): Promise<void> {
  const ctx = Context.instance();

  // Verify workspace ID is available
  if (!WORKSPACE_ID) {
    throw new Error("Workspace ID not available from API token");
  }

  // Load input file
  ctx.logger.info(`Loading component data from {file}`, { file: inputFile });
  const inputData = await loadComponentFile(inputFile);

  // Determine component ID (CLI overrides file)
  const componentIdOrName = options.component || inputData.componentId;
  if (!componentIdOrName) {
    throw new Error(
      "Component ID or name must be provided via --component flag or in the input file (componentId field)",
    );
  }

  // Resolve change set
  ctx.logger.info(`Resolving change set: {changeSet}`, {
    changeSet: options.changeSet,
  });
  const changeSetId = await resolveChangeSet(WORKSPACE_ID, options.changeSet);

  // Initialize API client
  const api = new ComponentsApi(apiConfig);

  // Fetch current component
  ctx.logger.info(`Fetching current component: {component}`, {
    component: componentIdOrName,
  });
  const currentComponent = await fetchCurrentComponent(
    api,
    WORKSPACE_ID,
    componentIdOrName,
    changeSetId,
  );

  // Fetch schema information for display
  const schemasApi = new SchemasApi(apiConfig);
  const schemaResponse = await schemasApi.getSchema({
    workspaceId: WORKSPACE_ID,
    changeSetId: changeSetId,
    schemaId: currentComponent.component?.schemaId,
  });

  ctx.logger.info("Schema: {schemaName} ({schemaId})", {
    schemaName: schemaResponse.data.name,
    schemaId: currentComponent.component?.schemaId,
  });
  ctx.logger.info("Component: {name} ({id})", {
    name: currentComponent.component?.name,
    id: currentComponent.component?.id,
  });

  // Extract and filter current attributes
  const currentAttributes = filterAttributes(
    currentComponent.component?.attributes || {},
  );

  // Input attributes are already filtered (from component get)
  const desiredAttributes = inputData.attributes;

  // Resolve any component names or search queries in subscriptions to IDs
  const searchApi = new SearchApi(apiConfig);
  ctx.logger.debug("Resolving subscription references");
  await resolveSubscriptionsInAttributes(
    desiredAttributes,
    searchApi,
    WORKSPACE_ID,
    changeSetId,
    ctx.logger,
  );

  // Compute diff
  ctx.logger.debug("Computing attribute diff");
  const diff = computeAttributeDiff(desiredAttributes, currentAttributes);

  // Check for name change
  const currentName = currentComponent.component?.name;
  const desiredName = inputData.attributes["/si/name"] as string | undefined;
  let nameChange: { from: string; to: string } | undefined;

  if (desiredName && currentName && desiredName !== currentName) {
    nameChange = { from: currentName, to: desiredName };
  }

  // Check if there are any changes
  const hasChanges = diff.set.size > 0 || diff.unset.length > 0 ||
    diff.subscriptions.size > 0 || nameChange !== undefined;

  if (!hasChanges) {
    ctx.logger.info("No changes to apply");
    return;
  }

  // Show what will be changed
  if (diff.set.size > 0) {
    ctx.logger.info("Attributes to set or update: {attributes}", {
      attributes: Object.fromEntries(diff.set),
    });
  }

  if (diff.subscriptions.size > 0) {
    ctx.logger.info("Subscriptions to update: {subscriptions}", {
      subscriptions: Object.fromEntries(diff.subscriptions),
    });
  }

  if (diff.unset.length > 0) {
    ctx.logger.info("Attributes to remove: {attributes}", {
      attributes: diff.unset,
    });
  }

  if (nameChange) {
    ctx.logger.info("Name change: {from} -> {to}", {
      from: nameChange.from,
      to: nameChange.to,
    });
  }

  // Dry run mode - exit without applying
  if (options.dryRun) {
    ctx.logger.info("Dry run mode - no changes applied");
    return;
  }

  // Apply update
  const componentId = currentComponent.component?.id;
  if (!componentId) {
    throw new Error("Component ID not found in response");
  }

  await updateComponent(
    api,
    WORKSPACE_ID,
    changeSetId,
    componentId,
    diff,
    nameChange,
  );

  ctx.logger.info(
    `Component {component} updated successfully in change set {changeSet}`,
    {
      component: currentName,
      changeSet: options.changeSet,
    },
  );
}
