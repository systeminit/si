import type { TemplateContext } from "./context.ts";
import type {
  ComponentChange,
  CreateChange,
  DeleteChange,
  UpdateChange,
} from "./converge_types.ts";
import { attributeDiffToUpdatePayload } from "./attribute_diff.ts";
import {
  ChangeSetsApi,
  type ChangeSetViewV1,
  ComponentsApi,
  type Configuration,
  type UpdateComponentV1Request,
} from "@systeminit/api-client";
import { extractErrorDetails, logComponentWithSchema } from "../helpers.ts";

/**
 * Executes all pending changes (creates, updates, deletes) in the provided order.
 * Creates or reuses a change set for the operations.
 *
 * @param ctx - Template context
 * @param changes - Ordered array of changes to execute
 * @param changeSetId - Change set ID to use
 * @param dryRun - If true, shows plan but doesn't execute changes
 */
export async function executeChanges(
  ctx: TemplateContext,
  changes: ComponentChange[],
  changeSetId: string,
  dryRun: boolean,
): Promise<void> {
  const apiConfig = ctx.apiConfig();
  const workspaceId = ctx.workspaceId();

  if (!apiConfig || !workspaceId) {
    throw new Error("API configuration not available");
  }

  const componentsApi = new ComponentsApi(apiConfig);

  // Track workingSet ID → created SI ID mapping
  const createdIds = new Map<string, string>();

  // Count different types of changes
  const createCount = changes.filter((c) => c.type === "create").length;
  const updateCount = changes.filter((c) => c.type === "update").length;
  const deleteCount = changes.filter((c) => c.type === "delete").length;

  // Execute each change in order (or show dry-run plan)
  let successCount = 0;
  let failCount = 0;
  let currentIndex = 0;

  for (const change of changes) {
    currentIndex++;

    try {
      if (change.type === "create") {
        if (dryRun) {
          dryRunCreate(change, currentIndex, changes.length, ctx);
        } else {
          await executeCreate(
            componentsApi,
            workspaceId,
            changeSetId,
            change,
            createdIds,
            ctx,
            apiConfig,
            currentIndex,
            changes.length,
          );
        }
        successCount++;
      } else if (change.type === "update") {
        if (dryRun) {
          dryRunUpdate(change, currentIndex, changes.length, ctx);
        } else {
          await executeUpdate(
            componentsApi,
            workspaceId,
            changeSetId,
            change,
            createdIds,
            ctx,
            currentIndex,
            changes.length,
          );
        }
        successCount++;
      } else if (change.type === "delete") {
        if (dryRun) {
          dryRunDelete(change, currentIndex, changes.length, ctx);
        } else {
          await executeDelete(
            componentsApi,
            workspaceId,
            changeSetId,
            change,
            ctx,
            currentIndex,
            changes.length,
          );
        }
        successCount++;
      }
    } catch (error) {
      failCount++;
      const errorDetails = extractErrorDetails(error, false);
      ctx.logger.error("Failed to execute change: {error}", {
        error: errorDetails,
      });
      // Log stack trace separately for proper formatting
      if (error instanceof Error && error.stack) {
        console.error(error.stack);
      }
      // Continue with remaining changes
    }
  }

  if (dryRun) {
    ctx.logger.info(
      "Dry run complete: {creates} creates, {updates} updates, {deletes} deletes",
      {
        creates: createCount,
        updates: updateCount,
        deletes: deleteCount,
      },
    );
  } else {
    ctx.logger.info(
      "Execution complete: {succeeded} succeeded, {failed} failed",
      {
        succeeded: successCount,
        failed: failCount,
      },
    );
  }
}

/**
 * Gets an existing change set by name, or creates a new one if it doesn't exist.
 *
 * @param apiConfig - API configuration
 * @param workspaceId - Workspace ID
 * @param changeSetName - Name of the change set
 * @param ctx - Template context for logging
 * @returns The change set ID
 */
export async function getOrCreateChangeSet(
  apiConfig: Configuration,
  workspaceId: string,
  changeSetName: string,
  ctx: TemplateContext,
): Promise<string> {
  const changeSetsApi = new ChangeSetsApi(apiConfig);

  // List existing change sets
  const response = await changeSetsApi.listChangeSets({ workspaceId });
  const changeSets = response.data.changeSets as ChangeSetViewV1[];
  const existing = changeSets.find(
    (cs) => cs.name === changeSetName,
  );

  if (existing) {
    ctx.logger.debug("Found existing change set: {name}", {
      name: changeSetName,
    });
    return existing.id;
  }

  // Create new change set
  ctx.logger.debug("Creating change set: {name}", { name: changeSetName });
  const createResponse = await changeSetsApi.createChangeSet({
    workspaceId,
    createChangeSetV1Request: { changeSetName },
  });

  return createResponse.data.changeSet.id;
}

/**
 * Gets the schema name from the schema ID using the schema cache.
 *
 * @param ctx - Template context
 * @param apiConfig - API configuration
 * @param workspaceId - Workspace ID
 * @param changeSetId - Change set ID
 * @param schemaId - Schema ID to look up
 * @returns The schema name
 */

/**
 * Executes a create change operation.
 *
 * @param api - Components API instance
 * @param workspaceId - Workspace ID
 * @param changeSetId - Change set ID
 * @param change - Create change to execute
 * @param createdIds - Map to track workingSet ID → SI ID mappings
 * @param ctx - Template context for logging
 * @param apiConfig - API configuration for schema lookups
 * @param current - Current change index
 * @param total - Total number of changes
 */
async function executeCreate(
  api: ComponentsApi,
  workspaceId: string,
  changeSetId: string,
  change: CreateChange,
  createdIds: Map<string, string>,
  ctx: TemplateContext,
  _apiConfig: Configuration,
  current: number,
  total: number,
): Promise<void> {
  const comp = change.workingSetComponent;

  // Get schema name for API call and logging
  const schemaName = await ctx.getSchemaName(
    workspaceId,
    changeSetId,
    comp.schemaId,
  );

  // Log component creation
  await logComponentWithSchema(
    ctx,
    comp,
    workspaceId,
    changeSetId,
    "Creating {schemaName} {siName} ({current}/{total})",
    current,
    total,
  );

  // Resolve any pending subscription dependencies
  const resolvedAttrs = resolveSubscriptionPlaceholders(
    change.attributes,
    createdIds,
  );

  // Add template tags
  const templateTag = `${ctx.name()}-${ctx.invocationKey()}`;
  resolvedAttrs["/si/tags/templateFrom"] = templateTag;
  resolvedAttrs["/si/tags/templateWorkingSetId"] = comp.id;

  ctx.logger.debug("Setting attributes on {name}: {attributes}", {
    name: comp.name,
    attributes: resolvedAttrs,
  });

  const response = await api.createComponent({
    workspaceId,
    changeSetId,
    createComponentV1Request: {
      schemaName,
      name: comp.name,
      attributes: resolvedAttrs,
    },
  });

  const newId = response.data.component.id;
  createdIds.set(comp.id, newId);

  ctx.logger.debug("Created with ID: {id}", { id: newId });
}

/**
 * Shows what a create operation would do in dry-run mode.
 */
function dryRunCreate(
  change: CreateChange,
  current: number,
  total: number,
  ctx: TemplateContext,
): void {
  const comp = change.workingSetComponent;

  // Get schema name from cache
  const schemaName = ctx.schemaCache().get(comp.schemaId)?.name ||
    comp.schemaId;

  ctx.logger.info("Dry Run: Creating {schemaName} {name} ({current}/{total})", {
    schemaName,
    name: comp.name,
    current,
    total,
  });

  // In dry-run, always show attributes at INFO level
  ctx.logger.info("Dry Run: Setting attributes on {name}: {attributes}", {
    name: comp.name,
    attributes: change.attributes,
  });
}

/**
 * Executes an update change operation.
 *
 * @param api - Components API instance
 * @param workspaceId - Workspace ID
 * @param changeSetId - Change set ID
 * @param change - Update change to execute
 * @param createdIds - Map to track workingSet ID → SI ID mappings
 * @param ctx - Template context for logging
 * @param current - Current change index
 * @param total - Total number of changes
 */
async function executeUpdate(
  api: ComponentsApi,
  workspaceId: string,
  changeSetId: string,
  change: UpdateChange,
  createdIds: Map<string, string>,
  ctx: TemplateContext,
  current: number,
  total: number,
): Promise<void> {
  const comp = change.existingComponent;

  // Log component update with schema name
  await logComponentWithSchema(
    ctx,
    comp,
    workspaceId,
    changeSetId,
    "Updating {schemaName} {siName} ({current}/{total})",
    current,
    total,
  );

  // Convert diff to update payload
  const payload = attributeDiffToUpdatePayload(change.attributeDiff);

  // Resolve subscription placeholders
  const resolvedPayload = resolveSubscriptionPlaceholders(
    payload,
    createdIds,
  );

  const updateRequest: UpdateComponentV1Request = {
    attributes: resolvedPayload,
  };

  if (change.nameChange) {
    updateRequest.name = change.nameChange.to;
  }

  ctx.logger.debug("Updating attributes on {name}: {request}", {
    name: comp.name,
    request: updateRequest,
  });

  // Log the complete update payload at trace level for debugging
  ctx.logger.trace("Complete update payload: {request}", {
    request: updateRequest,
  });

  await api.updateComponent({
    workspaceId,
    changeSetId,
    componentId: comp.id,
    updateComponentV1Request: updateRequest,
  });
}

/**
 * Shows what an update operation would do in dry-run mode.
 */
function dryRunUpdate(
  change: UpdateChange,
  current: number,
  total: number,
  ctx: TemplateContext,
): void {
  const comp = change.existingComponent;

  // Get schema name from cache
  const schemaName = ctx.schemaCache().get(comp.schemaId)?.name ||
    comp.schemaId;

  ctx.logger.info("Dry Run: Updating {schemaName} {name} ({current}/{total})", {
    schemaName,
    name: comp.name,
    current,
    total,
  });

  // Convert diff to update payload
  const payload = attributeDiffToUpdatePayload(change.attributeDiff);

  const updateRequest: UpdateComponentV1Request = {
    attributes: payload,
  };

  if (change.nameChange) {
    updateRequest.name = change.nameChange.to;
  }

  // In dry-run, always show attributes at INFO level
  ctx.logger.info("Updating attributes on {name}: {request}", {
    name: comp.name,
    request: updateRequest,
  });
}

/**
 * Executes a delete change operation.
 *
 * @param api - Components API instance
 * @param workspaceId - Workspace ID
 * @param changeSetId - Change set ID
 * @param change - Delete change to execute
 * @param ctx - Template context for logging
 * @param current - Current change index
 * @param total - Total number of changes
 */
async function executeDelete(
  api: ComponentsApi,
  workspaceId: string,
  changeSetId: string,
  change: DeleteChange,
  ctx: TemplateContext,
  current: number,
  total: number,
): Promise<void> {
  const comp = change.existingComponent;

  // Log component deletion with schema name
  await logComponentWithSchema(
    ctx,
    comp,
    workspaceId,
    changeSetId,
    "Deleting {schemaName} {siName} ({current}/{total})",
    current,
    total,
  );

  await api.deleteComponent({
    workspaceId,
    changeSetId,
    componentId: comp.id,
  });
}

/**
 * Shows what a delete operation would do in dry-run mode.
 */
function dryRunDelete(
  change: DeleteChange,
  current: number,
  total: number,
  ctx: TemplateContext,
): void {
  const comp = change.existingComponent;

  // Get schema name from cache
  const schemaName = ctx.schemaCache().get(comp.schemaId)?.name ||
    comp.schemaId;

  ctx.logger.info("Dry Run: Deleting {schemaName} {name} ({current}/{total})", {
    schemaName,
    name: comp.name,
    current,
    total,
  });
}

/**
 * Checks if a value represents a subscription to another component's attribute.
 *
 * @param value - Value to check
 * @returns True if the value is a subscription
 */
// deno-lint-ignore no-explicit-any
function isSubscription(value: any): boolean {
  return (
    typeof value === "object" &&
    value !== null &&
    "$source" in value &&
    typeof value.$source === "object" &&
    value.$source !== null &&
    "component" in value.$source &&
    "path" in value.$source
  );
}

/**
 * Resolves subscription placeholders by replacing workingSet IDs with actual SI component IDs.
 *
 * @param attributes - Attributes object potentially containing subscriptions
 * @param createdIds - Map of workingSet ID → SI component ID
 * @returns Resolved attributes with updated subscription references
 */
function resolveSubscriptionPlaceholders(
  // deno-lint-ignore no-explicit-any
  attributes: { [key: string]: any },
  createdIds: Map<string, string>,
  // deno-lint-ignore no-explicit-any
): { [key: string]: any } {
  // deno-lint-ignore no-explicit-any
  const resolved: { [key: string]: any } = {};

  for (const [path, value] of Object.entries(attributes)) {
    if (isSubscription(value)) {
      const componentRef = value.$source.component;
      const resolvedId = createdIds.get(componentRef) || componentRef;

      resolved[path] = {
        $source: {
          component: resolvedId,
          path: value.$source.path,
          ...(value.$source.func && { func: value.$source.func }),
        },
      };
    } else {
      resolved[path] = value;
    }
  }

  return resolved;
}
