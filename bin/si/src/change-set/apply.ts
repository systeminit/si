/**
 * Change Set Apply Module - Apply change sets
 *
 * This module provides functionality to apply change sets in the
 * System Initiative workspace. The change set will be merged into HEAD.
 * Note that HEAD change sets cannot be applied.
 *
 * @module
 */

import { ActionsApi, ChangeSetsApi, type ActionViewV1, ComponentsApi, SchemasApi } from "@systeminit/api-client";
import { Listr } from "listr2";
import { Context } from "../context.ts";
import type { ChangeSetApplyOptions } from "./types.ts";
import { resolveChangeSet } from "./utils.ts";
import { sleep } from "../helpers.ts";
import { getHeadChangeSetId } from "../cli/helpers.ts";

/**
 * Component metadata cache entry
 */
interface ComponentMetadata {
  componentName: string;
  schemaName: string;
}

export type { ChangeSetApplyOptions };

/**
 * Main entry point for the change-set apply command
 */
export async function callChangeSetApply(
  options: ChangeSetApplyOptions,
): Promise<void> {
  // Get context
  const ctx = Context.instance();

  try {
    const apiConfig = Context.apiConfig();
    const workspaceId = Context.workspaceId();

    const changeSetsApi = new ChangeSetsApi(apiConfig);
    const actionsApi = new ActionsApi(apiConfig);
    const componentsApi = new ComponentsApi(apiConfig);
    const schemasApi = new SchemasApi(apiConfig);

    ctx.logger.info("Gathering change set data...");

    // Resolve the change set ID from the provided ID or name
    const changeSetId = await resolveChangeSet(
      workspaceId,
      options.changeSetIdOrName,
    );

    // Check if the change set is HEAD - cannot apply HEAD
    const getResponse = await changeSetsApi.getChangeSet({
      workspaceId,
      changeSetId,
    });

    if (getResponse.data.changeSet.isHead) {
      throw new Error(
        "Cannot apply HEAD change set. HEAD is immutable and cannot be applied.",
      );
    }

    ctx.logger.info("Applying change set...");
    // Apply the change set with progressive backoff if we get a 'busy' response
    let status;
    let applyAttempts = 0;
    const MAX_APPLY_ATTEMPTS = 4;
    let actions = [] as ActionViewV1[];

    // Cache for component metadata (componentId -> {componentName, schemaName})
    const componentMetadataCache = new Map<string, ComponentMetadata>();

    do {
      await sleep(1000 * applyAttempts)
      applyAttempts++;

      const actionsResponse = await actionsApi.getActions({
        workspaceId,
        changeSetId,
      });

      actions = actionsResponse.data.actions.filter((a) => a.state === "Queued");

      // Fetch component metadata for each action if not already cached
      for (const action of actions) {
        if (action.componentId && !componentMetadataCache.has(action.componentId)) {
          try {
            const componentResponse = await componentsApi.getComponent({
              workspaceId,
              changeSetId,
              componentId: action.componentId,
            });

            const component = componentResponse.data.component;
            let schemaName = "unknown";

            // Fetch schema name using schemaId
            if (component.schemaId) {
              try {
                const schemaResponse = await schemasApi.getSchema({
                  workspaceId,
                  changeSetId,
                  schemaId: component.schemaId,
                });
                schemaName = schemaResponse.data.name || "unknown";
              } catch (schemaError) {
                ctx.logger.warn(`Failed to fetch schema for component ${action.componentId}: ${schemaError}`);
              }
            }

            componentMetadataCache.set(action.componentId, {
              componentName: component.name || "unknown",
              schemaName,
            });
          } catch (error) {
            ctx.logger.warn(`Failed to fetch component metadata for ${action.componentId}: ${error}`);
            // Set fallback metadata
            componentMetadataCache.set(action.componentId, {
              componentName: "unknown",
              schemaName: "unknown",
            });
          }
        }
      }

      // Apply the change set (request approval)
      const response = await changeSetsApi.requestApproval({
        workspaceId,
        changeSetId,
      }, {
        validateStatus: (status) =>
          // 428 is blocked because of DVU. we can treat this without throwing
          (status >= 200 && status < 300) || status == 428,
      });

      status = response.status;
    } while (status === 428 && applyAttempts < MAX_APPLY_ATTEMPTS);

    if (status !== 200) {
      ctx.logger.error(`Failed to apply change set after ${MAX_APPLY_ATTEMPTS} attempts. Change set is busy, try again later.`);
      return;
    }

    // Check if the change set needs approval
    const updatedResponse = await changeSetsApi.getChangeSet({
      workspaceId,
      changeSetId,
    });

    if (updatedResponse.data.changeSet.status === "NeedsApproval") {
      ctx.logger.info(
        "Change set apply requested and is awaiting approval: {*}",
        {
          id: changeSetId,
          name: getResponse.data.changeSet.name,
          status: updatedResponse.data.changeSet.status,
        },
      );
      ctx.logger.info(
        "Please review and approve the change set in the System Initiative web application.",
      );
      return;
    }

    ctx.logger.info("Change set applied successfully: {*}", {
      id: changeSetId,
      name: getResponse.data.changeSet.name,
      status: updatedResponse.data.changeSet.status,
    });

    ctx.analytics.trackEvent("change set apply", {
      detach: options.detach ?? false,
      actionCount: actions.length,
    });

    if (actions.length === 0) {
      ctx.logger.info("No enqueued actions found.")
      return;
    }


    // Detached mode: list actions and return without waiting
    if (options.detach) {
      ctx.logger.info(
        `Found ${actions.length} enqueued action(s):`,
      );
      for (const action of actions) {
        const metadata = action.componentId ? componentMetadataCache.get(action.componentId) : undefined;
        const componentName = metadata?.componentName || "unknown";
        const schemaName = metadata?.schemaName || "unknown";
        ctx.logger.info(
          "  - [{schemaName}] {componentName} - {actionName}",
          {
            schemaName,
            componentName,
            actionName: action.name || "unknown",
          }
        );
      }

      ctx.logger.info("Actions are running in the background. Use the System Initiative web application to monitor progress.");

      return;
    }

    // Get HEAD changeset ID for polling actions after apply
    const headChangeSetId = await getHeadChangeSetId();

    // Helper function to format action title with component metadata
    const formatActionTitle = (action: ActionViewV1, state: string): string => {
      const metadata = action.componentId ? componentMetadataCache.get(action.componentId) : undefined;
      const componentName = metadata?.componentName || "unknown";
      const schemaName = metadata?.schemaName || "unknown";
      return `[${schemaName}] ${componentName} - ${action.name || "unknown"} - ${state}`;
    };

    // Create a map to track action states and their task references
    const actionStates = new Map<string, string>();
    const actionTaskRefs = new Map<string, { title: string }>();
    let pollingComplete = false; // Signal when polling has finished

    // Initialize action states
    for (const action of actions) {
      actionStates.set(action.id, action.state);
    }

    // Background polling promise that updates action states periodically
    const pollingPromise = (async () => {
      while (true) {
        try {
          // Fetch all action states in a single API call
          const actionsResponse = await actionsApi.getActions({
            workspaceId,
            changeSetId: headChangeSetId,
          });

          let runningActionFound = false;
          let hasFailedActions = false;
          let hasQueuedActions = false;

          // Update states for all actions we're tracking
          for (const action of actions) {
            const currentAction = actionsResponse.data.actions.find((a) => a.id === action.id);
            const taskRef = actionTaskRefs.get(action.id);

            // Handle action not found on HEAD (completed or blocked)
            if (!currentAction) {
              const wasQueued = actionStates.get(action.id) === "Queued";
              const newState = wasQueued ? "OnHold" : "Completed";
              const displayMsg = wasQueued ? "Blocked (dependency failed)" : "Success ✓";

              actionStates.set(action.id, newState);
              if (taskRef) taskRef.title = formatActionTitle(action, displayMsg);
              continue;
            }

            // Handle completed action
            if (currentAction.state === "Completed") {
              actionStates.set(action.id, "Completed");
              if (taskRef) taskRef.title = formatActionTitle(action, "Success ✓");
              continue;
            }

            // Update current state and track flags
            const currentState = currentAction.state;
            actionStates.set(action.id, currentState);

            let stateMsg = currentState;
            if (currentState === "Failed") {
              stateMsg = "Failed ❌";
              hasFailedActions = true;
            } else if (currentState === "OnHold") {
              stateMsg = "On Hold ⏸️";
            } else if (currentState === "Queued") {
              hasQueuedActions = true;
            } else if (currentState === "Running" || currentState === "Dispatched") {
              runningActionFound = true;
            }

            if (taskRef) taskRef.title = formatActionTitle(action, stateMsg);
          }

          // Exit polling if:
          // 1. No actions are actively running, AND
          // 2. Either there are no queued actions OR there are failed actions (blocked dependencies)
          if (!runningActionFound && (!hasQueuedActions || hasFailedActions)) {
            pollingComplete = true; // Signal to Listr tasks that polling is done
            return;
          }

          // Sleep at end of loop instead of beginning
          await sleep(1000);
        } catch (error) {
          ctx.logger.warn(`Error polling action states: ${error}`);
          await sleep(1000); // Also sleep on error before retrying
        }
      }
    })();

    // Main task list that will contain all action tasks
    const mainActionsTask = new Listr([
      {
        title: `Executing ${actions.length} action(s):`,
        task: (_ctx: unknown, task) => {
          // Create subtasks for each action using task.newListr()
          return task.newListr(
            actions.map((action) => ({
              title: formatActionTitle(action, action.state),
              task: async (_ctx: unknown, subtask: { title: string }) => {
                // Store reference to this task so polling can update it
                actionTaskRefs.set(action.id, subtask);

                // Wait for action to complete
                let completed = false;
                while (!completed) {
                  await sleep(500); // Check local state more frequently than we poll API

                  // If polling has completed, exit task
                  if (pollingComplete) {
                    const finalState = actionStates.get(action.id);
                    // If action is still queued when polling stops, it was blocked
                    if (finalState === "Queued") {
                      // Throw an error to mark this task as failed in Listr
                      throw new Error(formatActionTitle(action, "Blocked (dependency failed)"));
                    }
                    completed = true;
                    break;
                  }

                  const currentState = actionStates.get(action.id);

                  // Check if action is in a terminal state
                  if (currentState === undefined || currentState === "Completed") {
                    completed = true;
                  } else if (["Failed", "OnHold"].includes(currentState)) {
                    throw new Error(subtask.title);
                  }
                }
              },
            })),
            {
              concurrent: true,
              exitOnError: false,
            }
          );
        },
      },
    ])


    try {
      if (ctx.isInteractive) {
        // Interactive mode: show live progress with Listr
        await Promise.all([
          mainActionsTask.run(),
          pollingPromise,
        ]);
      } else {
        // Non-interactive mode: just poll and log final results
        ctx.logger.info(`Waiting for ${actions.length} action(s) to complete...`);

        // Wait for actions to finish
        await pollingPromise;

        // Log final results
        ctx.logger.info("Action execution complete:");
        for (const action of actions) {
          const state = actionStates.get(action.id) || "unknown";

          if (state === "Completed") {
            ctx.logger.info(`  ✓ ${formatActionTitle(action, 'Success')}`);
          } else if (state === "Failed") {
            ctx.logger.error(`  ✗ ${formatActionTitle(action, 'Failed')}`);
          } else if (state === "OnHold") {
            ctx.logger.warn(`  ⏸ ${formatActionTitle(action, 'On Hold')}`);
          } else {
            ctx.logger.info(`  - ${formatActionTitle(action, state)}`);
          }
        }
      }

      // Check if all actions completed successfully
      const failedActions = Array.from(actionStates.entries())
        .filter(([_, state]) => state === "Failed");

      if (failedActions.length > 0) {
        ctx.logger.error(
          `Change set applied successfully, but ${failedActions.length} action(s) failed.`
        );
        Deno.exit(1);
      }

      ctx.logger.info("All actions completed successfully!");
    } catch (error) {
      ctx.logger.error(`Error executing actions: ${error}`);

      // Show final state of all actions
      ctx.logger.info("Final action states:");
      for (const action of actions) {
        const state = actionStates.get(action.id);
        if (state) {
          const metadata = action.componentId ? componentMetadataCache.get(action.componentId) : undefined;
          const componentName = metadata?.componentName || "unknown";
          const schemaName = metadata?.schemaName || "unknown";
          ctx.logger.info(`  - [${schemaName}] ${componentName} - ${action.name || "unknown"}: ${state}`);
        }
      }

      Deno.exit(1);
    }


  } catch (error) {
    ctx.logger.error(`Failed to apply change set: ${error}`);
    Deno.exit(1);
  }
}
