/**
 * Change Set Abandon Module - Abandon (delete) change sets
 *
 * This module provides functionality to abandon change sets in the
 * System Initiative workspace. Note that HEAD change sets cannot be abandoned.
 *
 * @module
 */

import { ChangeSetsApi } from "@systeminit/api-client";
import { Context } from "../context.ts";
import type { ChangeSetAbandonOptions } from "./types.ts";
import { resolveChangeSet } from "./utils.ts";

export type { ChangeSetAbandonOptions };

/**
 * Main entry point for the change-set abandon command
 */
export async function callChangeSetAbandon(
  options: ChangeSetAbandonOptions,
): Promise<void> {
  // Get context
  const ctx = Context.instance();

  try {
    const apiConfig = Context.apiConfig();
    const workspaceId = Context.workspaceId();

    const changeSetsApi = new ChangeSetsApi(apiConfig);

    // Resolve the change set ID from the provided ID or name
    const changeSetId = await resolveChangeSet(
      workspaceId,
      options.changeSetIdOrName,
    );

    // Check if the change set is HEAD - cannot abandon HEAD
    const getResponse = await changeSetsApi.getChangeSet({
      workspaceId,
      changeSetId,
    });

    if (getResponse.data.changeSet.isHead) {
      throw new Error(
        "Cannot abandon HEAD change set. HEAD is immutable and cannot be deleted.",
      );
    }

    // Abandon the change set
    const response = await changeSetsApi.abandonChangeSet({
      workspaceId,
      changeSetId,
    });

    if (response.data.success) {
      ctx.logger.info("Change set abandoned: {*}", {
        id: changeSetId,
        name: getResponse.data.changeSet.name,
      });
    }
    
    ctx.analytics.trackEvent("change set abandon", {});
    
  } catch (error) {
    ctx.logger.error(`Failed to abandon change set: ${error}`);
    Deno.exit(1);
  }
}
