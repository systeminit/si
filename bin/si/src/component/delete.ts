/**
 * Component Delete Module - Delete components in a change set
 *
 * This module provides functionality to delete components from the System Initiative
 * API. It supports deletion by component ID or name, and includes a dry-run mode
 * to preview what would be deleted. The command gracefully handles components that
 * are already marked for deletion.
 *
 * @module
 */

import { ComponentsApi, SchemasApi } from "@systeminit/api-client";
import { Context } from "../context.ts";
import { resolveChangeSet } from "./change_set.ts";

/**
 * Options for the component delete command
 */
export interface ComponentDeleteOptions {
  /** Change set ID or name (required) */
  changeSet: string;
  /** Dry-run mode - preview deletion without applying */
  dryRun?: boolean;
}

/**
 * Deletes a component by ID or name.
 *
 * This function:
 * 1. Resolves the component by ID or name
 * 2. Checks if the component is already marked for deletion
 * 3. In dry-run mode, logs what would be deleted and exits
 * 4. Otherwise, deletes the component via the API
 *
 * @param componentIdOrName - Component ID (ULID) or name
 * @param options - Command options including changeSet and dryRun
 * @throws Error if API configuration is not initialized or component not found
 */
export async function callComponentDelete(
  componentIdOrName: string,
  options: ComponentDeleteOptions,
): Promise<void> {
  const ctx = Context.instance();
  const apiConfig = Context.apiConfig();
  const workspaceId = Context.workspaceId();

  ctx.logger.info(`Deleting component: {component}`, {
    component: componentIdOrName,
  });

  // Resolve change set
  const changeSetId = await resolveChangeSet(workspaceId, options.changeSet);
  ctx.logger.debug(`Using change set: {changeSetId}`, { changeSetId });

  // Fetch component to verify it exists and get its ID
  const componentsApi = new ComponentsApi(apiConfig);

  // Detect if it's a ULID (26 alphanumeric characters)
  // ULIDs use Crockford's base32: 0-9 and A-Z excluding I, L, O, U
  const isUlid = /^[0-9A-HJKMNP-TV-Z]{26}$/i.test(componentIdOrName);

  let component;
  try {
    const componentResponse = await componentsApi.findComponent({
      workspaceId,
      changeSetId: changeSetId,
      // Use componentId parameter for ULIDs, component parameter for names
      ...(isUlid
        ? { componentId: componentIdOrName }
        : { component: componentIdOrName }),
    });

    component = componentResponse.data.component;

    ctx.logger.debug(`Found component: {id} ({name})`, {
      id: component.id,
      name: component.name,
    });
  } catch (error) {
    // Check if it's a 404 (component not found)
    const statusCode =
      // deno-lint-ignore no-explicit-any
      (error as any)?.response?.status ||
      // deno-lint-ignore no-explicit-any
      (error as any)?.status;

    if (statusCode === 404) {
      // Component doesn't exist - treat as success (idempotent)
      ctx.logger.warn(
        `Component '{component}' not found - already deleted or never existed`,
        { component: componentIdOrName },
      );
      ctx.logger.info("No action needed");
      return;
    }

    // Other errors should propagate
    throw error;
  }

  // Check if already marked for deletion
  if (component.toDelete) {
    ctx.logger.info(`Component {name} ({id}) is already marked for deletion`, {
      name: component.name,
      id: component.id,
    });

    // In dry-run mode, still exit successfully
    if (options.dryRun) {
      ctx.logger.info("Dry-run mode - no changes applied");
    } else {
      ctx.logger.info("No action needed");
    }

    return;
  }

  // Dry-run mode: log what would be deleted
  if (options.dryRun) {
    ctx.logger.info(`Would delete component: {name} ({id})`, {
      name: component.name,
      id: component.id,
    });
    ctx.logger.info("Dry-run mode - no changes applied");
    return;
  }

  // Delete the component
  await componentsApi.deleteComponent({
    workspaceId,
    changeSetId: changeSetId,
    componentId: component.id,
  });

  ctx.logger.info(`Successfully deleted component: {name} ({id})`, {
    name: component.name,
    id: component.id,
  });

  // Track component delete - fetch schema name first
  const schemasApi = new SchemasApi(apiConfig);
  const schemaResponse = await schemasApi.getSchema({
    workspaceId,
    changeSetId,
    schemaId: component.schemaId,
  });

  ctx.analytics.trackEvent("component delete", {
    schemaName: schemaResponse.data.name,
    dryRun: options.dryRun ?? false,
  });
}
