/**
 * Change Set Apply Module - Apply change sets
 *
 * This module provides functionality to apply change sets in the
 * System Initiative workspace. The change set will be merged into HEAD.
 * Note that HEAD change sets cannot be applied.
 *
 * @module
 */

import { ActionsApi, ChangeSetsApi, type ActionViewV1 } from "@systeminit/api-client";
import { Context } from "../context.ts";
import type { ChangeSetApplyOptions } from "./types.ts";
import { resolveChangeSet } from "./utils.ts";
import { sleep } from "../helpers.ts";

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
    do {
      await sleep(1000 * applyAttempts)
      applyAttempts++;

      const actionsResponse = await actionsApi.getActions({
        workspaceId,
        changeSetId,
      });

      actions = actionsResponse.data.actions.filter((a) => a.state === "Queued");

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

    // TODO(victor) - get component and asset names
    // TODO(victor) - block terminal and keep pooling action statuses until they're done, unless detach arg is passed in (see docker run)
    // Get enqueued actions from HEAD
    if (actions.length > 0) {
      ctx.logger.info(
        `Found ${actions.length} enqueued action(s):`,
      );
      for (const action of actions) {
        ctx.logger.info("  - {name}({id}) of kind {kind}, for component {componentId}", {
          id: action.id,
          name: action.name,
          kind: action.kind,
          componentId: action.componentId,
        });
      }

    }


  } catch (error) {
    ctx.logger.error(`Failed to apply change set: ${error}`);
    Deno.exit(1);
  }
}
