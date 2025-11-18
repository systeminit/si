import type { TemplateContext } from "./context.ts";
import type {
  ComponentChange,
  CreateChange,
  DeleteChange,
  ExistingSetComponent,
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
 * @param existingSet - Existing components with template tags
 * @param dryRun - If true, shows plan but doesn't execute changes
 */
export async function executeChanges(
  ctx: TemplateContext,
  changes: ComponentChange[],
  changeSetId: string,
  existingSet: ExistingSetComponent[],
  dryRun: boolean,
): Promise<void> {
  const apiConfig = ctx.apiConfig();
  const workspaceId = ctx.workspaceId();

  if (!apiConfig || !workspaceId) {
    throw new Error("API configuration not available");
  }

  const componentsApi = new ComponentsApi(apiConfig);

  // Track workingSet ID → created SI ID mapping
  // Initialize with existing components to resolve references to already-existing components
  const createdIds = new Map<string, string>();
  for (const existing of existingSet) {
    createdIds.set(existing.templateWorkingSetId, existing.id);
    ctx.logger.debug(
      "Mapped existing component: workingSet ID {wsId} → SI ID {siId}",
      { wsId: existing.templateWorkingSetId, siId: existing.id },
    );
  }

  // Build registry of all working set IDs from the baseline
  // This helps us distinguish between working set IDs (that need resolution)
  // and external component IDs (that should be used as-is)
  const workingSetIds = new Set<string>();
  const workingSet = ctx.workingSet() || null;
  if (workingSet) {
    for (const component of workingSet) {
      workingSetIds.add(component.id);
      ctx.logger.trace("Registered working set ID: {id} ({name})", {
        id: component.id,
        name: component.name,
      });
    }
  }

  // Track components that failed to create (by working set ID)
  const failedComponentIds = new Set<string>();

  // Count different types of changes
  const createCount = changes.filter((c) => c.type === "create").length;
  const updateCount = changes.filter((c) => c.type === "update").length;
  const deleteCount = changes.filter((c) => c.type === "delete").length;

  // Execute each change in order (or show dry-run plan)
  let successCount = 0;
  let failCount = 0;
  let skipCount = 0;
  let currentIndex = 0;

  for (const change of changes) {
    currentIndex++;

    try {
      if (change.type === "create") {
        // Check if any dependencies failed
        const depCheck = checkDependenciesAvailable(
          change.attributes,
          failedComponentIds,
          workingSetIds,
          workingSet,
        );

        if (!depCheck.canCreate) {
          // Skip this component because a dependency failed
          const schemaName = await ctx.getSchemaName(
            workspaceId,
            changeSetId,
            change.workingSetComponent.schemaId,
          );
          ctx.logger.info(
            "Skipping {schemaName} {name} ({current}/{total}) - dependency {dep} failed to create",
            {
              schemaName,
              name: change.workingSetComponent.name,
              current: currentIndex,
              total: changes.length,
              dep: depCheck.failedDependencies[0],
            },
          );
          // Add this component to failed set so its dependents also skip
          failedComponentIds.add(change.workingSetComponent.id);
          skipCount++;
          continue;
        }

        if (dryRun) {
          dryRunCreate(change, currentIndex, changes.length, ctx);
        } else {
          await executeCreate(
            componentsApi,
            workspaceId,
            changeSetId,
            change,
            createdIds,
            workingSetIds,
            workingSet,
            ctx,
            apiConfig,
            currentIndex,
            changes.length,
          );
        }
        successCount++;
      } else if (change.type === "update") {
        // For updates, convert the diff back to attributes format for dependency checking
        const updateAttrs: { [key: string]: unknown } = {};
        for (const [path, value] of change.attributeDiff.set.entries()) {
          updateAttrs[path] = value;
        }

        // Check if any dependencies failed
        const depCheck = checkDependenciesAvailable(
          updateAttrs,
          failedComponentIds,
          workingSetIds,
          workingSet,
        );

        if (!depCheck.canCreate) {
          // Skip this component because a dependency failed
          const schemaName = await ctx.getSchemaName(
            workspaceId,
            changeSetId,
            change.existingComponent.schemaId,
          );
          ctx.logger.info(
            "Skipping update of {schemaName} {name} ({current}/{total}) - dependency {dep} failed",
            {
              schemaName,
              name: change.existingComponent.name,
              current: currentIndex,
              total: changes.length,
              dep: depCheck.failedDependencies[0],
            },
          );
          // Add this component to failed set so its dependents also skip
          const wsId = change.existingComponent.attributes?.[
            "/si/tags/templateWorkingSetId"
          ];
          if (wsId && typeof wsId === "string") {
            failedComponentIds.add(wsId);
          }
          skipCount++;
          continue;
        }

        if (dryRun) {
          dryRunUpdate(change, currentIndex, changes.length, ctx);
        } else {
          await executeUpdate(
            componentsApi,
            workspaceId,
            changeSetId,
            change,
            createdIds,
            workingSetIds,
            workingSet,
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

      // Track which component failed so we can skip dependents
      if (change.type === "create") {
        failedComponentIds.add(change.workingSetComponent.id);
      } else if (change.type === "update") {
        // For updates, track by the templateWorkingSetId if available
        const wsId = change.existingComponent.attributes?.[
          "/si/tags/templateWorkingSetId"
        ];
        if (wsId && typeof wsId === "string") {
          failedComponentIds.add(wsId);
        }
      }

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
    if (skipCount > 0) {
      ctx.logger.info(
        "Execution complete: {succeeded} succeeded, {failed} failed, {skipped} skipped",
        {
          succeeded: successCount,
          failed: failCount,
          skipped: skipCount,
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
 * @param workingSetIds - Set of all working set IDs from the baseline
 * @param workingSet - Array of working set components
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
  workingSetIds: Set<string>,
  workingSet: { id: string; name: string }[] | null,
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
    workingSetIds,
    workingSet,
    comp.name,
    ctx,
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
 * @param workingSetIds - Set of all working set IDs from the baseline
 * @param workingSet - Array of working set components
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
  workingSetIds: Set<string>,
  workingSet: { id: string; name: string }[] | null,
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
    workingSetIds,
    workingSet,
    comp.name,
    ctx,
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
 * Checks if a component's dependencies are available for creation.
 * Returns false if any dependency has failed to create.
 *
 * @param attributes - Attributes object potentially containing subscriptions
 * @param failedComponentIds - Set of working set IDs that failed to create
 * @param workingSetIds - Set of all working set IDs from the baseline
 * @param workingSet - Array of working set components (for name lookup)
 * @returns Object with canCreate flag and list of failed dependency names
 */
function checkDependenciesAvailable(
  // deno-lint-ignore no-explicit-any
  attributes: { [key: string]: any },
  failedComponentIds: Set<string>,
  workingSetIds: Set<string>,
  workingSet: { id: string; name: string }[] | null,
): { canCreate: boolean; failedDependencies: string[] } {
  const failedDeps: string[] = [];

  for (const value of Object.values(attributes)) {
    if (isSubscription(value)) {
      const componentRef = value.$source.component;

      // Only check working set IDs - external components are assumed to exist
      if (workingSetIds.has(componentRef)) {
        if (failedComponentIds.has(componentRef)) {
          // This dependency failed - get its name for error message
          const depName = workingSet?.find((c) =>
            c.id === componentRef
          )?.name ||
            componentRef;
          failedDeps.push(depName);
        }
      }
    }
  }

  return {
    canCreate: failedDeps.length === 0,
    failedDependencies: failedDeps,
  };
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
 * @param workingSetIds - Set of all working set IDs from the baseline
 * @param workingSet - Array of working set components (for name lookup)
 * @param componentName - Name of the component being processed (for error messages)
 * @param ctx - Template context for logging
 * @returns Resolved attributes with updated subscription references
 */
function resolveSubscriptionPlaceholders(
  // deno-lint-ignore no-explicit-any
  attributes: { [key: string]: any },
  createdIds: Map<string, string>,
  workingSetIds: Set<string>,
  workingSet: { id: string; name: string }[] | null,
  componentName: string,
  ctx: TemplateContext,
  // deno-lint-ignore no-explicit-any
): { [key: string]: any } {
  // deno-lint-ignore no-explicit-any
  const resolved: { [key: string]: any } = {};

  for (const [path, value] of Object.entries(attributes)) {
    if (isSubscription(value)) {
      const componentRef = value.$source.component;
      const subscriptionPath = value.$source.path;

      // Check if this is a working set ID that needs resolution
      if (workingSetIds.has(componentRef)) {
        // This is a working set ID - check if it's been resolved
        const resolvedId = createdIds.get(componentRef);

        if (!resolvedId) {
          // Working set ID hasn't been resolved yet - this is a dependency problem
          const refComponentName = workingSet?.find((c) =>
            c.id === componentRef
          )
            ?.name || componentRef;

          throw new Error(
            `Subscription resolution failed for component "${componentName}":\n` +
              `  Attribute path: ${path}\n` +
              `  References component: "${refComponentName}" (working set ID: ${componentRef})\n` +
              `  Subscription path: ${subscriptionPath}\n\n` +
              `This component has not been created yet. This usually indicates a dependency ordering issue.\n` +
              `The topological sort should have ensured dependencies are created first.\n\n` +
              `Possible causes:\n` +
              `  - The referenced component failed to create\n` +
              `  - There's a circular dependency\n` +
              `  - The referenced component is not in the working set\n\n` +
              `Debug info:\n` +
              `  - Total working set components: ${workingSetIds.size}\n` +
              `  - Components created so far: ${createdIds.size}\n` +
              `  - Available created IDs: ${
                Array.from(createdIds.keys()).join(", ")
              }`,
          );
        }

        ctx.logger.trace(
          "Resolved subscription: {componentRef} → {resolvedId} for {componentName}",
          { componentRef, resolvedId, componentName },
        );

        resolved[path] = {
          $source: {
            component: resolvedId,
            path: subscriptionPath,
            ...(value.$source.func && { func: value.$source.func }),
          },
        };
      } else {
        // Not a working set ID - assume it's an external component ID (e.g., credentials, regions)
        // Use it as-is
        ctx.logger.trace(
          "Using external component reference: {componentRef} for {componentName}",
          { componentRef, componentName },
        );

        resolved[path] = {
          $source: {
            component: componentRef,
            path: subscriptionPath,
            ...(value.$source.func && { func: value.$source.func }),
          },
        };
      }
    } else {
      resolved[path] = value;
    }
  }

  return resolved;
}
